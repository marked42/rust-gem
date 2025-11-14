use my_redis::delay::Delay;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(3);
    let future = Delay { when };

    let out = future.await;

    assert_eq!(out, "done");
}
