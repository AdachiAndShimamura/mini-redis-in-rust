use bytes::Bytes;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use Rust_Redis::base::cmd::command::Command;
use Rust_Redis::base::cmd::command::Command::{Get, Set};
use Rust_Redis::base::connection::Connection;
use Rust_Redis::base::frame::Frame;

pub static DATABASE: OnceCell<Arc<RwLock<HashMap<String, Bytes>>>> = OnceCell::new();

pub fn get_data_base() -> Arc<RwLock<HashMap<String, Bytes>>> {
    DATABASE
        .get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
        .clone()
}

#[tokio::main]
async fn main() {
    let tcp_listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (stream, _) = tcp_listener.accept().await.unwrap();
        tokio::spawn(tcp_connect_handle(stream));
    }
}

pub async fn tcp_connect_handle(stream: TcpStream) {

}

#[test]
fn test() {}
