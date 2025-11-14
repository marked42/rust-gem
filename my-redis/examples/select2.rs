use tokio::sync::{mpsc};

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = mpsc::channel(128);

    let mut done = false;
    let operation = action(None);
    tokio::pin!(operation);

    tokio::spawn(async move {
        let _ = tx.send(1).await;
        let _ = tx.send(2).await;
        let _ = tx.send(3).await;
    });

    loop {
        tokio::select! {
            res = &mut operation, if !done => {
                done = true;

                if let Some(v) = res {
                    println!("GOT = {}", v);
                    return;
                }
            }
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    operation.set(action(Some(v)));
                    done = false;
                }
            }
        }
    }
}

async fn action(input: Option<i32>) -> Option<String> {
    let i = input?;

    Some(i.to_string())
}