use tokio::net::{ TcpStream};
use Rust_Redis::base::redis_server::RedisServer;


#[tokio::main]
async fn main() {
    env_logger::init();

    let fut = RedisServer::connect_and_start("127.0.0.1:6379".parse().unwrap());
    tokio::join!(fut);
}

pub async fn tcp_connect_handle(stream: TcpStream) {}

#[test]
fn test() {}
