use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6113").await?;
    tokio::spawn(async {});
    loop {
        let (connect, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (mut reader, mut writer) = io::split(connect);
            if io::copy(&mut reader, &mut writer).await.is_err() {
                eprint!("io copy error!");
            }
        })
    }
}
