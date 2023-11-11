use bytes::Bytes;
use tokio::sync::mpsc::channel;
use tokio::sync::oneshot::channel as oneshot_channel;
use Rust_Redis::base::cmd::command::{Command, CommandWrap};
use Rust_Redis::base::cmd::set::Set;

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = channel::<CommandWrap>(32);
    let manager = tokio::spawn(async move {
        let mut client = connect("127.0.0.1:6379").await.unwrap();
        while let Some(command_wrap) = rx.recv().await {
            let (command, channel) = (command_wrap.command, command_wrap.channel);
            match command {
                Command::Get(get) => {
                    let res = client.get(&key).await.expect("get error");
                    channel.send(Ok(res)).expect("TODO: panic message");
                }
                Command::Set(set) => {
                    let res = client.set(&key, value).await.expect("set error");
                    channel.send(Ok(None)).expect("TODO: panic message");
                }
            }
        }
    });
    let tx2 = tx.clone();
    let task1 = tokio::spawn(async move {
        let (sender, receiver) = oneshot_channel();
        tx.send(CommandWrap::new(
            Command::Set("key1".to_string(), Bytes::from("value1")),
            sender,
        ))
        .await
        .expect("send error");
        if let Ok(..) = receiver.await.unwrap() {
            println!("ok");
        }
    });
    let task2 = tokio::spawn(async move {
        let (sender, receiver) = oneshot_channel();
        tx2.send(CommandWrap::new(
            Command::Set(Set {
                key: "key2".to_string(),
                value: Bytes::from("value2"),
            }),
            sender,
        ))
        .await
        .expect("send error");
        if let Ok(Some(res)) = receiver.await.unwrap() {
            println!("{:?}", res);
        }
    });
    task1.await.expect("task1 error");
    task2.await.expect("task2 error");
}
