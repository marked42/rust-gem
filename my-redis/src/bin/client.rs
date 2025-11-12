use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get { key: String, resp: Responder<Option<Bytes>> },
    Set { key: String, value: Bytes, resp: Responder<()> },
}

#[tokio::main]
async fn main() {
    let (sender1, mut receiver) = mpsc::channel::<Command>(32);
    let sender2 = sender1.clone();

    let t1 = tokio::spawn(async move {
        let (resp_sender, resp_receiver) = oneshot::channel();
        let command = Command::Get {
            key: "foo".to_string(),
            resp: resp_sender,
        };

        if sender1.send(command).await.is_err() {
            return;
        }

        let res = resp_receiver.await;
        println!("GOT (GET) = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_sender, resp_receiver) = oneshot::channel();
        let command = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_sender,
        };

        if sender2.send(command).await.is_err() {
            return;
        }

        let res = resp_receiver.await;
        println!("GOT (Set) = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("localhost:6379").await.unwrap();

        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, value, resp } => {
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
