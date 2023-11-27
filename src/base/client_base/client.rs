use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::net::TcpSocket;
use crate::base::connection::Connection;
use tokio::sync::{broadcast, RwLock};
use crate::base::client_base::event::Message;
use anyhow::Result;
use crate::base::cmd::command::Command;

pub struct ClientConnection{
    pub connection:Connection,
    pub subscribers:HashMap<String,broadcast::Sender<Message>>
}

impl ClientConnection{
    pub async fn connect(port:u16, target:SocketAddr) ->Result<ClientConnection>{
        let socket=TcpSocket::new_v4().unwrap();
        socket.bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port).expect("error addr")).expect("failed to bind addr");
        let stream=socket.connect(target).await?;
        Ok(ClientConnection{
            connection:Connection::new(stream),
            subscribers:HashMap::new()
        })
    }

    pub fn get(key:&str){

    }

}
pub struct RedisClient{
    pub connection:Arc<RwLock<ClientConnection>>,
    pub subscribers:HashMap<String,broadcast::Receiver<Message>>
}

impl RedisClient {
    pub fn new(connection:Arc<RwLock<ClientConnection>>)->RedisClient{
        RedisClient{
            connection,
            subscribers:HashMap::new()
        }
    }

    pub fn get(key:&str){

    }
}