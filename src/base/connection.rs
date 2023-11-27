use crate::base::event::{Event, EventType};
use crate::base::frame::Frame;
use anyhow::Result;
use bytes::BytesMut;
use log::info;
use std::collections::HashMap;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync;

pub struct Connection {
    pub(crate) stream: TcpStream,
    buffer: BytesMut,
    subscribe: HashMap<EventType, sync::watch::Receiver<Event>>,
}

pub const STREAM_MAX_BYTES: usize = 65536;
impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        return Connection {
            stream,
            buffer: BytesMut::with_capacity(STREAM_MAX_BYTES),
            subscribe: HashMap::new(),
        };
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        let bytes = Frame::parse_frame_to_bytes(frame)?;
        info!("write data to frame:{:?}", bytes);
        self.stream.write(&*bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }
    pub async fn read_frame(&mut self) -> Result<Frame> {
        loop {
            if let Ok(data) = self.try_parse_frame().await {
                info!("success to parse a frame{:?}", data);
                return Ok(data);
            }
            let data_len = self.stream.read_buf(&mut self.buffer).await?;
            info!("read data:{},stream:{:?}", data_len, self.buffer);
        }
    }

    pub async fn get_state(&self) -> bool {
        return match self.stream.readable().await {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    pub async fn try_parse_frame(&mut self) -> Result<Frame> {
        let mut buffer_wrap = Cursor::new(&self.buffer[..]);
        Frame::check(&mut buffer_wrap)?;
        info!("success to parse a frame");
        buffer_wrap.set_position(0);
        let res = Frame::parse(&mut buffer_wrap);
        let position = buffer_wrap.position();
        let _ = self.buffer.split_to(position as usize);
        res
    }
}
