use std::thread;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() {
    tokio::join!(hello(1, 200), hello(2, 200), hello(3, 200),);
}

async fn hello(task: u64, time: u64) {
    println!("Task {task} started on {:?}.", thread::current().id());
    thread::sleep(Duration::from_millis(time));
    println!("Task {task} finished");
}
