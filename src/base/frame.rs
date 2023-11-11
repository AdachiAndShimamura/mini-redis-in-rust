use crate::base::connection::STREAM_MAX_BYTES;
use crate::base::frame::Frame::{Array, Bulk, ErrorResult, Integer, Null, Simple};
use anyhow::Result;
use anyhow::{anyhow, Error};
use atoi::atoi;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::Write as fmt_writer;
use std::io::{Cursor, Write};

pub const PARSE_ERROR: &str = "Parse Error";
pub const WRITE_ERROR: &str = "Write Error";
pub enum Frame {
    Simple(String),
    Integer(i64),
    Null,
    ErrorResult(String),
    Bulk(Bytes),
    Array(Vec<Frame>),
}
impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<()> {
        let data = get_u8(src)?;
        return match data {
            //字符串
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            //整数
            b':' => {
                get_u8(src)?;
                get_i64_char(src)?;
                Ok(())
            }
            //空
            b'_' => {
                get_u64_char(src)?;
                Ok(())
            }
            //错误信息
            b'-' => {
                get_u64_char(src)?;
                get_line(src)?;
                Ok(())
            }
            //多行字符串
            b'$' => {
                get_u64_char(src)?;
                get_line(src)?;
                Ok(())
            }
            //数组
            b'*' => {
                // let frame_vec=Vec::new();
                let len = get_u64_char(src)?;
                for _ in 0..len {
                    Frame::check(src)?;
                }
                Ok(())
            }
            _ => Err(Error::msg("failed to check data")),
        };
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame> {
        let data = get_u8(src)?;
        return match data {
            //字符串
            b'+' => {
                let data = get_line(src)?.to_vec();
                Ok(Simple(String::from_utf8(data)?))
            }
            //整数
            b':' => Ok(Integer(get_i64_char(src)?)),
            //空
            b'_' => {
                skip(src);
                Ok(Null)
            }
            //错误信息
            b'-' => Ok(ErrorResult(String::from_utf8(get_line(src)?.to_vec())?)),
            //多行字符串
            b'$' => {
                let len = get_u64_char(src)?;
                let position = src.position();
                let data = &src.get_ref()[position as usize..position as usize + len as usize];
                src.set_position(position + len);
                skip(src);
                Ok(Bulk(Bytes::copy_from_slice(data)))
            }
            //数组
            b'*' => {
                let len = get_u64_char(src)?;
                let mut frame_vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let frame = Frame::parse(src)?;
                    frame_vec.push(frame);
                }
                Ok(Array(frame_vec))
            }
            _ => Err(anyhow!("failed to parse frame : frame head error!")),
        };
    }

    pub fn parse_to_bytes(frame: Frame) -> Result<BytesMut> {
        let mut bytes = BytesMut::with_capacity(STREAM_MAX_BYTES);
        match frame {
            Simple(value) => {
                bytes.put_u8(b'+');
                bytes.write_str(value.as_str())?;
                write_rn_to_bytes(&mut bytes)?;
            }
            Integer(value) => {
                bytes.put_u8(b':');
                bytes.put_i64(value);
                write_rn_to_bytes(&mut bytes)?;
            }
            Null => {
                bytes.put_u8(b'_');
                write_rn_to_bytes(&mut bytes)?;
            }
            ErrorResult(value) => {
                bytes.put_u8(b'-');
                bytes.write_str(value.as_str())?;
                write_rn_to_bytes(&mut bytes)?;
            }
            Bulk(value) => {
                bytes.put_u8(b'$');
                bytes.put_u32(value.len() as u32);
                write_rn_to_bytes(&mut bytes)?;
                bytes.extend(value);
                write_rn_to_bytes(&mut bytes)?;
            }
            Array(vec) => {
                let mut byte_vec: Vec<u8> = Vec::new();
                byte_vec.put_u8(b'*');
                let len = vec.len();
                byte_vec.put_u32(len as u32);
                write_rn(&mut byte_vec)?;
                for single_frame in vec {
                    let frame_bytes = Frame::parse_to_bytes(single_frame)?;
                    bytes.extend(frame_bytes);
                }
            }
        }
        Ok(bytes)
    }
}
pub fn write_line(src: &[u8], target: &mut Vec<u8>) -> Result<()> {
    target.write(src)?;
    target.write("\r\n".as_ref())?;
    Ok(())
}
pub fn write_rn(target: &mut Vec<u8>) -> Result<()> {
    target.write("\r\n".as_ref())?;
    Ok(())
}
pub fn write_rn_to_bytes(target: &mut BytesMut) -> Result<()> {
    target.writer().write("\r\n".as_bytes())?;
    Ok(())
}

pub fn skip(src: &mut Cursor<&[u8]>) {
    let index = src.position();
    src.set_position(index + 2);
}
pub fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8> {
    if !src.has_remaining() {
        return Err(anyhow!("src has no data"));
    };
    Ok(src.get_u8())
}

pub fn get_u64_char(src: &mut Cursor<&[u8]>) -> Result<u64> {
    use atoi::atoi;
    let line = get_line(src)?;
    atoi(line).ok_or_else(|| anyhow!("failed to parse line to u64"))
}

pub fn get_i64_char(src: &mut Cursor<&[u8]>) -> Result<i64> {
    use atoi::atoi;
    let line = get_line(src)?;
    atoi(line).ok_or_else(|| anyhow!("failed to parse line to u64"))
}

pub fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    let start = src.position();
    let end = (src.get_ref().len() - 1) as u64;
    for index in start..end - 1 {
        if src.get_ref()[index as usize] == b'\r' && src.get_ref()[index as usize + 1] == b'\n' {
            src.set_position(index + 2);
            return Ok(&src.get_ref()[start as usize..index as usize - 1]);
        }
    }
    Err(anyhow!("buffer has no line"))
}
