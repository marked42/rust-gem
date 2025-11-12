use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut str = String::new();
    f.read_to_string(&mut str).await?;
    println!("The string: {}", str);
    Ok(())
}
