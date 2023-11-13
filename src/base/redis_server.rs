use std::net::SocketAddr;
use log::{debug, info};
use tokio::net::{TcpListener, TcpStream};
use crate::base::cmd::command::Command;
use crate::base::cmd::command::Command::{Get, Set};
use crate::base::connection::Connection;
use crate::base::data_base::get_data_base;
use crate::base::frame::Frame;

pub struct RedisServer{

}

impl RedisServer {
    // pub fn default()->RedisServer{
    //
    // }
    pub async fn connect_and_start(addr:SocketAddr){
        let listener=TcpListener::bind(addr).await.unwrap();
        loop {
            let (stream,_)=listener.accept().await.unwrap();
            tokio::spawn(
                async move{
                    RedisServer::handle(stream).await;
                }
            );
        }
    }

    pub fn init_data_base(&mut self){

    }

    pub async fn handle(stream:TcpStream){
        let mut connection = Connection::new(stream);
        while connection.get_state().await {
            while let Ok(frame) = connection.read_frame().await {
                if let request = Command::from_frame(frame).unwrap() {
                    match request {
                        Get(data) => {
                            let data_base = get_data_base().read().await;
                            let res = data_base.get(data.key.as_str());
                            match res {
                                None => {
                                    connection
                                        .write_frame(Frame::ErrorResult("Result NULL!".parse().unwrap()))
                                        .await
                                        .expect("error");
                                }
                                Some(data) => {
                                    info!("get:value:{:?}",data);
                                    connection
                                        .write_frame(Frame::Bulk(data.clone()))
                                        .await
                                        .expect("error");
                                }
                            }
                        }
                        // Command::Publish(_) => {}
                        Set(set) => {
                            let (key, value) = (set.key, set.value);

                            info!("command:SET  key:{}  value:{:?}",key,value);

                            get_data_base().write().await.insert(key, value);
                            connection
                                .write_frame(Frame::Simple("OK".parse().unwrap()))
                                .await
                                .expect("error");
                        } // Command::Subscribe(_) => {}
                        // Command::Unsubscribe(_) => {}
                        // Command::Unknown(_) => {}
                    }
                };
            }
        }
    }
}