use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(vec![1, 2, 3]);

    while let Some(i) = stream.next().await {
        eprintln!("Got number: {}", i);
    }
}
