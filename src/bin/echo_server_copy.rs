#![feature(future_join)]

use std::future::join;
use Rust_Redis::base::redis_server::RedisServer;

#[tokio::main]
async fn main() {
    let fut = RedisServer::connect_and_start("127.0.0.1:6379".parse().unwrap());
    join!(fut);
}
