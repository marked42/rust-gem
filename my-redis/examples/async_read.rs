use std::io;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = [0; 10];

    let n = f.read(&mut buffer).await?;

    println!("The bytes: {:?}",  String::from_utf8_lossy(&buffer[..n]));
    Ok(())
}