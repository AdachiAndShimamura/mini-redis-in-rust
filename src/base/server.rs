use crate::base::cmd::command::Command;
use crate::base::cmd::command::Command::{Get, Set};
use crate::base::connection::Connection;
use crate::base::db::DB;
use crate::base::event::{Event, Message};
use crate::base::frame::Frame;
use async_stream::stream;
use std::iter::Iterator;
use tokio_stream::StreamExt;
use log::{ info};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_stream::StreamMap;

#[derive(Clone)]
pub struct RedisServer {
    db: DB,
}

impl RedisServer {
    // pub fn default()->RedisServer{
    //
    // }
    pub async fn connect_and_start(&self, addr: SocketAddr) {
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            let db=self.db.clone();
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut server = SingleServer::new(db, stream);
                server.start().await;
            });
        }
    }

    pub fn init_data_base(&mut self) {}
}

pub struct SingleServer {
    db: DB,
    connection: Connection,
    subscribe: StreamMap<String, Message>,
    // shutdown:tokio::sync::watch::Receiver<String>
}

impl SingleServer {
    pub fn new(db: DB, connect: TcpStream) -> SingleServer {
        return SingleServer {
            db,
            connection: Connection::new(connect),
            subscribe: StreamMap::new(),
        };
    }

    pub async fn start(&mut self) {
        while self.connection.get_state().await {
            select! {
                Ok(command)=self.connection.read_frame()=>{
                    self.handle_command_from_connection(command);
                }
                Some((channel_name,message))=self.subscribe.next()=>{
                    let event=Event::Change(channel_name,message);
                    let frame=event.parse_ro_frame();
                    self.connection.write_frame(frame).await;
                }
            }
        }
        info!("Connection Closed!");
    }

    pub async fn handle_command_from_connection(&mut self, frame: Frame) {
        if let request = Command::from_frame(frame).unwrap() {
            match request {
                Get(data) => match self.db.get(data.key).await {
                    None => {
                        self.connection
                            .write_frame(Frame::ErrorResult("Result NULL!".parse().unwrap()))
                            .await
                            .expect("error");
                    }
                    Some(data) => {
                        info!("get:value:{:?}", data);
                        self.connection
                            .write_frame(Frame::Bulk(data.clone()))
                            .await
                            .expect("error");
                    }
                },
                Set(set) => {
                    let (key, value) = (set.key, set.value);

                    info!("command:SET  key:{}  value:{:?}", key, value);

                    self.db.set(key, value).await;
                    self.connection
                        .write_frame(Frame::Simple("OK".parse().unwrap()))
                        .await
                        .expect("error");
                }
                Command::Subscribe(subscribe) => {
                    let channels = subscribe.channels;
                    for channel in &channels {
                        let mut receiver = self.db.subscribe(channel).await;
                        let channel_future = Box::pin(stream! {
                            loop{
                                match receiver.recv().await{
                            Ok(res) => {
                                yield res
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {

                            }
                            Err(_)=>{
                                break
                            }
                        }
                            }
                        });
                        self.subscribe.insert(channel.clone(), channel_future);
                    }
                }
                Command::UnSubscribe(unsubscribe) => {
                    let channels = unsubscribe.channels;
                    for channel in channels {
                        self.subscribe.remove(&channel);
                    }
                }
                Command::Publish(publish) => {
                    let (key, value) = (publish.key, publish.value);
                    self.db.publish(key, value).await.unwrap();
                }
                Command::Unknown => {
                    info!("unknown command!");
                }
            }
        };
    }
}
