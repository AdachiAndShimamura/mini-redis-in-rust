use tokio::sync::oneshot;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
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
        let listener=TcpListener::bind(addr).unwrap();
        loop {
            let (stream,_)=listener.accept().unwrap();
            tokio::spawn(
                async move{
                    RedisServer::handle(stream)
                }
            )
        }
    }

    pub fn init_data_base(&mut self){

    }

    pub async fn handle(mut stream:TcpStream){
        let mut connection = Connection::new(stream);
        while let Ok(frame) = connection.read_frame().await {
            if let request = Command::from_frame(frame).unwrap() {
                match request {
                    Get(data) => {
                        let res = get_data_base()
                            .read()
                            .unwrap()
                            .get(data.key.as_str())
                            .cloned();
                        match res {
                            None => {
                                connection
                                    .write_frame(Frame::ErrorResult("Result NULL!".parse().unwrap()))
                                    .await
                                    .expect("error");
                            }
                            Some(data) => {
                                connection
                                    .write_frame(Frame::Bulk(data))
                                    .await
                                    .expect("error");
                            }
                        }
                    }
                    // Command::Publish(_) => {}
                    Set(set) => {
                        let (key, value) = (set.key, set.value);
                        get_data_base().write().unwrap().insert(key, value);
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