use crate::base::frame::Frame;
use anyhow::Error;
use bytes::BytesMut;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Connection {
    pub(crate) stream: TcpStream,
    buffer: BytesMut,
}

pub const STREAM_MAX_BYTES: usize = 65536;
impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        return Connection {
            stream,
            buffer: BytesMut::with_capacity(STREAM_MAX_BYTES),
        };
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<(), Error> {
        let bytes = Frame::parse_to_bytes(frame)?;
        self.stream.write(&*bytes).await?;
        Ok(())
    }
    pub async fn read_frame(&mut self) -> Result<Frame, Error> {
        loop {
            if let data = self.parse_frame().await? {
                return Ok(data);
            }
            self.stream.read(&mut self.buffer).await?;
        }
    }

    pub async fn parse_frame(&mut self) -> Result<Frame, Error> {
        let mut buffer_wrap = Cursor::new(&self.buffer[..]);
        Frame::check(&mut buffer_wrap)?;
        buffer_wrap.set_position(0);
        Frame::parse(&mut buffer_wrap)
    }
}
