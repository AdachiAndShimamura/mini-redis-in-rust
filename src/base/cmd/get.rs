use crate::base::frame::Frame;
use crate::base::frame::Frame::Simple;
use anyhow::Result;

pub struct Get {
    pub key: String,
}

impl Get {
    pub fn new(key:&str)->Get{
        Get{
            key:key.to_string()
        }
    }
}