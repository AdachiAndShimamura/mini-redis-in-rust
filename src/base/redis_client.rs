use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::net::{TcpSocket};
use anyhow::Result;
use bytes::Bytes;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::base::cmd::command::{Command, CommandWrap};
use crate::base::cmd::get::Get;
use crate::base::cmd::set::Set;
use crate::base::connection::Connection;
use crate::base::frame::Frame;

pub struct RedisClient{
    pub command_sender:mpsc::Sender<CommandWrap>,

}


pub struct ClientConnection{
    pub connection:Connection,
    pub command_receiver:mpsc::Receiver<CommandWrap>,
}

impl ClientConnection {
    pub fn from_connection(connection:Connection,command_receiver:mpsc::Receiver<CommandWrap>)->ClientConnection{
        ClientConnection{
            connection,
            command_receiver
        }
    }
    pub async fn start(&mut self){
        tokio::spawn(async move{
            loop {
                let command=self.command_receiver.recv().await.unwrap();
                let frame=command.command.parse_to_frame();
                self.write_frame_to_stream(frame).await.expect("failed to write frame to stream");
            }
        });
    }

    pub async fn write_frame_to_stream(&mut self,frame:Frame)->Result<()>{
        self.connection.write_frame(frame).await?;
        self.connection.stream.flush().await?;
        Ok(())
    }

}

impl RedisClient {
    pub fn new(command_sender:Sender<CommandWrap>)->Self{
        RedisClient{
            command_sender
        }
    }

    pub async fn connect(addr:SocketAddr) ->Result<RedisClient>{
        let socket=TcpSocket::new_v4().unwrap();
        let stream=socket.connect(addr).await?;
        let (command_sender,command_receiver)=mpsc::channel(100);
        let connection=ClientConnection::from_connection(Connection::new(stream),command_receiver);
        let mut redis_client =RedisClient{
            command_sender,
        };
        redis_client.start().await?;
        Ok(redis_client)
    }


    pub async fn request(&mut self,command:Command)->Result<Option<Bytes>>{
        let (sender,receiver)=oneshot_channel();
        let command_wrap=CommandWrap::new(command,sender);
        self.command_sender.send(command_wrap).await?;
        if let Ok(Some(response))=receiver.await.unwrap(){
            Ok(Some(response))
        }
        Ok(None)
    }

    pub async fn get(&mut self, key:&str)->Result<Option<Bytes>>{
        let command=Command::Get(Get::new(key));
        self.request(command).await
    }

    pub async fn set(&mut self,key:&str,value:Bytes)->Result<Option<Bytes>>{
        let command=Command::Set(
            Set::new(key,value)
        );
        self.request(command).await
    }
}