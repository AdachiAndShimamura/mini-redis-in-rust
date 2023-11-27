use bytes::Bytes;
use futures::Stream;
use std::pin::{pin, Pin};
use crate::base::frame::Frame;
use crate::base::frame::Frame::{Bulk, Simple};

#[derive(Eq, PartialEq, Hash)]
pub enum EventType {
    Delete(String),
    Change(String),
}
pub enum Event {
    Delete(String),
    Change(String, Bytes),
}

impl Event{
    pub fn parse_ro_frame(&self)->Frame{
        match self {
            Event::Delete(key) => {
                let command = Simple("GET".to_string());
                let data = Simple(key.clone());
                let frame_vec = vec![command, data];
                let frame = Frame::Array(frame_vec);
                frame
            }
            Event::Change(key, value) => {
                let command = Simple("SET".to_string());
                let key = Simple(key.clone());
                let value = Bulk(value.clone());
                let frame_vec = vec![command, key, value];
                let frame = Frame::Array(frame_vec);
                frame
            }
        }
    }
}
pub(crate) type Message = Pin<Box<dyn Stream<Item = Bytes> + Send>>;
