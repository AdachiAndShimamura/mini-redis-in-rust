use crate::base::cmd::get::Get;
use crate::base::cmd::publish::Publish;
use crate::base::cmd::set::Set;
use crate::base::cmd::subscribe::Subscribe;
use crate::base::cmd::unsubscribe::UnSubscribe;
use crate::base::frame::Frame;
use crate::base::frame::Frame::{Array, Bulk, Simple};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use tokio::sync::oneshot;

pub type Responder<T> = oneshot::Sender<Result<T>>;

pub enum Command {
    Get(Get),
    Set(Set),
    Subscribe(Subscribe),
    UnSubscribe(UnSubscribe),
    Publish(Publish),
    Unknown,
}

impl Command {
    pub fn parse_to_frame(&self) -> Frame {
        match self {
            Command::Get(get) => {
                let command = Simple("GET".to_string());
                let data = Simple(get.key.clone());
                let frame_vec = vec![command, data];
                let frame = Frame::Array(frame_vec);
                frame
            }
            Command::Set(set) => {
                let command = Simple("SET".to_string());
                let key = Simple(set.key.clone());
                let value = Bulk(set.value.clone());
                let frame_vec = vec![command, key, value];
                let frame = Frame::Array(frame_vec);
                frame
            }
            Command::Subscribe(subscribe) => {
                let command = Simple("SUBSCRIBE".to_string());
                let data = Array(
                    subscribe
                        .channels
                        .iter()
                        .map(|value| Frame::Simple(value.clone()))
                        .collect(),
                );
                let frame_vec = vec![command, data];
                Frame::Array(frame_vec)
            }
            Command::UnSubscribe(unsubscribe) => {
                let command = Simple("SUBSCRIBE".to_string());
                let data = Array(
                    unsubscribe
                        .channels
                        .iter()
                        .map(|value| Frame::Simple(value.clone()))
                        .collect(),
                );
                let frame_vec = vec![command, data];
                Frame::Array(frame_vec)
            }
            Command::Publish(publish) => {
                let command = Simple("SET".to_string());
                let channel = Simple(publish.key.clone());
                let value = Bulk(publish.value.clone());
                let frame_vec = vec![command, channel, value];
                let frame = Frame::Array(frame_vec);
                frame
            }
            Command::Unknown => Frame::Null,
        }
    }
    pub fn from_frame(frame: Frame) -> Result<Command> {
        return match frame {
            Frame::Array(value) => {
                let mut iter = value.iter();
                let mut command = String::new();
                match iter.next().unwrap() {
                    Frame::Simple(com) => {
                        command = com.clone();
                    }
                    _ => {
                        return Err(anyhow!("failed to get command from frame"));
                    }
                };
                match command.as_str() {
                    "GET" => {
                        let next = iter.next().unwrap();
                        let key = Frame::parse_to_string(next).unwrap();
                        Ok(Command::Get(Get { key }))
                    }
                    "SET" => {
                        let next = iter.next().unwrap();
                        let key = Frame::parse_to_string(next).unwrap();
                        let next = iter.next().unwrap();
                        let value = Frame::parse_to_bytes(next).unwrap().unwrap();
                        Ok(Command::Set(Set { key, value }))
                    }
                    "PUBLISH" => {
                        let key = Frame::parse_to_string(iter.next().unwrap()).unwrap();
                        let value = Frame::parse_to_bytes(iter.next().unwrap())
                            .unwrap()
                            .unwrap();
                        Ok(Command::Publish(Publish { key, value }))
                    }
                    "SUBSCRIBE" => {
                        let vec = Frame::parse_to_vec_string(iter.next().unwrap()).unwrap();
                        Ok(Command::Subscribe(Subscribe { channels: vec }))
                    }
                    "UNSUBSCRIBE" => {
                        let vec = Frame::parse_to_vec_string(iter.next().unwrap()).unwrap();
                        Ok(Command::UnSubscribe(UnSubscribe { channels: vec }))
                    }
                    _ => Ok(Command::Unknown),
                }
            }
            _ => Err(anyhow!("error frame!not a command!")),
        };
    }

    pub fn self_to_frame(&self) {}
}

pub struct CommandWrap {
    pub command: Command,
    pub channel: Responder<Option<Bytes>>,
}

impl CommandWrap {
    pub fn new(command: Command, channel: Responder<Option<Bytes>>) -> CommandWrap {
        CommandWrap { command, channel }
    }
}
