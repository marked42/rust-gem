use my_redis::delay::Delay;
use my_redis::interval::Interval;
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(1);
    let mut future = Interval {
        rem: 3,
        delay: Delay { when },
    };

    while let Some(val) = future.next().await {
        println!("{:?}", val)
    }
    println!("done");
}
