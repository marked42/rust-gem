use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut file = File::create("foo.txt").await?;
    let n = file.write(b"some bytes").await?;

    println!("Wrote the first {} bytes of 'some bytes", n);
    Ok(())
}
