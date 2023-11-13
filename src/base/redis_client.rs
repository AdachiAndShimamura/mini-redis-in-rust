use crate::base::cmd::command::{Command, CommandWrap};
use crate::base::cmd::get::Get;
use crate::base::cmd::set::Set;
use crate::base::connection::{Connection, STREAM_MAX_BYTES};
use crate::base::frame::Frame;
use anyhow::{anyhow, Result};
use bytes::{Bytes, BytesMut};
use std::io::Read;
use std::net::SocketAddr;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{ Sender};
use tokio::sync::oneshot::channel as oneshot_channel;

pub struct RedisClient {
    pub command_sender: mpsc::Sender<CommandWrap>,
}

pub struct ClientConnection {
    pub connection: Connection,
    pub command_receiver: mpsc::Receiver<CommandWrap>,
}

impl ClientConnection {
    pub fn from_connection(
        connection: Connection,
        command_receiver: mpsc::Receiver<CommandWrap>,
    ) -> ClientConnection {
        ClientConnection {
            connection,
            command_receiver,
        }
    }
    pub async fn start(mut connection: ClientConnection) {
        tokio::spawn(async move {
            loop {
                let command = connection.command_receiver.recv().await.unwrap();
                let frame = command.command.parse_to_frame();
                let sender = command.channel;
                let response = connection
                    .request(frame)
                    .await
                    .expect("failed to write frame to stream");
                info!("received response!");
                sender.send(Ok(response)).unwrap();
            }
        });
    }

    pub async fn request(&mut self, frame: Frame) -> Result<Option<Bytes>> {
        self.connection.write_frame(frame).await?;
        self.connection.stream.flush().await?;
        let mut res = BytesMut::with_capacity(STREAM_MAX_BYTES);
        Frame::parse_to_bytes(&self.connection.read_frame().await?)
        // info!("receive response,len:{}",res.len());
        // Ok(Some(Bytes::from(res)))
    }
}

impl RedisClient {
    pub fn new(command_sender: Sender<CommandWrap>) -> Self {
        RedisClient { command_sender }
    }

    pub async fn connect(addr: SocketAddr) -> Result<RedisClient> {
        let socket = TcpSocket::new_v4().unwrap();
        let stream = socket.connect(addr).await?;
        let (command_sender, command_receiver) = mpsc::channel(100);
        let connection =
            ClientConnection::from_connection(Connection::new(stream), command_receiver);
        ClientConnection::start(connection).await;
        Ok(RedisClient { command_sender })
    }

    pub async fn request(&mut self, command: Command) -> Result<Option<Bytes>> {
        let (sender, receiver) = oneshot_channel();
        let command_wrap = CommandWrap::new(command, sender);
        self.command_sender.send(command_wrap).await?;
        let response = receiver.await.unwrap().unwrap();
        Ok(response)
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        let command = Command::Get(Get::new(key));
        self.request(command).await
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<Option<Bytes>> {
        let command = Command::Set(Set::new(key, value));
        self.request(command).await
    }
}
