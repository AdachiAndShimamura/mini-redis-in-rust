use crate::base::connection::STREAM_MAX_BYTES;
use crate::base::frame::Frame::{Array, Bulk, ErrorResult, Integer, Null, Simple};
use anyhow::Result;
use anyhow::{anyhow, Error};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::{Display, Write as fmt_writer};
use std::i64;
use std::io::{Cursor, Write};
use log::{debug, info};

pub const PARSE_ERROR: &str = "Parse Error";
pub const WRITE_ERROR: &str = "Write Error";
#[derive(Clone,Debug)]
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
        debug!("{}",data);
        return match data {
            //字符串
            b'+' => {
                get_line(src)?;
                info!("{}",src.position());
                Ok(())
            }
            //整数
            b':' => {
                get_u8(src)?;
                get_message_i64(src)?;
                info!("{}",src.position());
                Ok(())
            }
            //空
            b'_' => {
                skip(src);
                info!("{}",src.position());
                Ok(())
            }
            //错误信息
            b'-' => {
                get_message_bytes(src)?;
                get_line(src)?;
                info!("{}",src.position());
                Ok(())
            }
            //多行字符串
            b'$' => {
                get_message_len(src)?;
                get_line(src)?;
                info!("{}",src.position());
                Ok(())
            }
            //数组
            b'*' => {
                debug!("frame:numbers");
                if let Ok(len) = get_message_len(src){
                    debug!("frame message len:{}",len);
                    for _ in 0..len {
                        Frame::check(src)?;
                        info!("{}",src.position());
                    }
                    Ok(())
                }else {
                    debug!("failed to get len!");
                    Err(Error::msg("failed to check data"))
                }

            }
            _ => {
                info!("error message head:{}",data);
                return Err(Error::msg("failed to check data"))},
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
            b':' => Ok(Integer(get_message_i64(src)?)),
            //空
            b'_' => {
                skip(src);
                Ok(Null)
            }
            //错误信息
            b'-' => Ok(ErrorResult(String::from_utf8(get_line(src)?.to_vec())?)),
            //多行字符串
            b'$' => {
                let len = get_message_len(src)?;
                let position = src.position();
                let data = &src.get_ref()[position as usize..position as usize + len as usize];
                src.set_position(position + len as u64);
                skip(src);
                Ok(Bulk(Bytes::copy_from_slice(data)))
            }
            //数组
            b'*' => {
                let len = get_message_len(src)?;
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

    pub fn parse_frame_to_bytes(frame: Frame) -> Result<BytesMut> {
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
                bytes.extend(byte_vec);
                for single_frame in vec {
                    let frame_bytes = Frame::parse_frame_to_bytes(single_frame)?;
                    bytes.extend(frame_bytes);
                }
            }
        }
        Ok(bytes)
    }

    pub fn parse_to_string(frame:&Frame)->Result<String>{
        match frame {
            Simple(val) => {
                Ok(val.clone())
            }
            Integer(_) => {
                Err(anyhow!("failed to parse to string!"))
            }
            Null => {
                Err(anyhow!("failed to parse to string!"))
            }
            ErrorResult(val) => {
                Err(anyhow!("{}",val))
            }
            Bulk(_) => {
                Err(anyhow!("failed to parse to string!"))
            }
            Array(_) => {
                Err(anyhow!("failed to parse to string!"))
            }
        }
    }

    pub fn parse_to_bytes(frame:&Frame)->Result<Option<Bytes>>{
        match frame.clone() {
            Frame::Simple(val) => {
                Ok(Some(Bytes::from(val)))
            }
            Frame::Integer(val) => {
                let data=val.to_be_bytes();
                let res=Bytes::from(data.to_vec());
                Ok(Some(res))
            }
            Frame::Null => {
                Ok(None)
            }
            Frame::ErrorResult(val) => {
                Ok(Some(Bytes::from(val)))
            }
            Frame::Bulk(val) => {
                Ok(Some(val))
            }
            Frame::Array(val) => {
                Err(anyhow!("error response!"))
            }
        }
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
pub fn get_message_bytes(src:&mut Cursor<&[u8]>)->Result<Bytes>{
    let line=get_line(src)?;
    Ok(Bytes::copy_from_slice(line))
}
pub fn get_message_i64(src:&mut Cursor<&[u8]>)->Result<i64>{
    let line = get_line(src)?;
    Ok(i64::from_be_bytes([line[0],line[1],line[2],line[3],line[4],line[5],line[6],line[7]]))
}
pub fn get_message_len(src:&mut Cursor<&[u8]>)->Result<u32>{
    let line = get_line(src)?;

    Ok(u32::from_be_bytes([line[0],line[1],line[2],line[3]]))
}

pub fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    let mut start = src.position() as usize;
    let end = src.get_ref().len();

    debug!("line::start:{}, end:{}", start, end);

    while start < end - 1 {
        if src.get_ref()[start] == b'\r' && src.get_ref()[start + 1] == b'\n' {
            let data = &src.get_ref()[src.position() as usize..start];
            debug!("success to read line: start: {},end:{}",src.position(), start);
            src.set_position((start + 2) as u64);
            return Ok(data);
        }
        start += 1;
    }

    Err(anyhow!("buffer has no line"))
}

#[test]
pub fn test(){
    use atoi::atoi;
    let line:[u8;4] = [50,50,50,50];
    let res=atoi::<u32>(line.as_ref()).ok_or_else(|| anyhow!("failed to parse line to u64"));
    match res {
        Ok(value) => {
            println!("{}", value);
        }
        Err(_) => {
            println!("error!")
        }
    }
}
