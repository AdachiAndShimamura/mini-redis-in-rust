use crate::base::cmd::get::Get;
use crate::base::cmd::set::Set;
use crate::base::frame::Frame;
use crate::base::frame::Frame::{Bulk, Simple};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use tokio::sync::oneshot;

pub type Responder<T> = oneshot::Sender<Result<T>>;

pub enum Command {
    Get(Get),
    Set(Set),
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
                    _ => Err(anyhow!("error command!")),
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
