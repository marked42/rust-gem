use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut file = File::create("foo.txt").await?;
    file.write_all(b"Hello, world!").await?;
    Ok(())
}
