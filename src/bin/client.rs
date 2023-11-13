use anyhow::Result;
use bytes::Bytes;
use std::net::SocketAddr;
use std::thread::{sleep, Thread};
use std::time::Duration;
use Rust_Redis::base::redis_client::RedisClient;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let mut client = RedisClient::connect("127.0.0.1:6379".parse().unwrap())
        .await
        .unwrap();
    let re1=client.set("key1", Bytes::from("value1")).await?;
    let re2=client.set("key2", Bytes::from("value2")).await?;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let res1 = client.get("key1").await.unwrap();
    let res2 = client.get("key2").await.unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!("{:?}", re1);
    println!("{:?}", re2);
    println!("{:?}", res1);
    println!("{:?}", res2);
    Ok(())
}
