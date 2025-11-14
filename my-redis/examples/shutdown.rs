use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(1);

    for i in 0..10 {
        tokio::spawn(some_operation(i, sender.clone()));
    }

    // drop sender
    drop(sender);

    let r = receiver.recv().await;

    println!("app shutdown {:?}", r);
}

async fn some_operation(i: u64, _sender: mpsc::Sender<u32>) {
    time::sleep(Duration::from_millis(100 * i)).await;
    println!("Task {} shutting down", i);
}
