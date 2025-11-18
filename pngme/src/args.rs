use std::path::PathBuf;

pub use clap::Parser;
use clap::Subcommand;
pub use pngme::Result;
use pngme::chunk_type::ChunkType;

#[derive(Parser, Debug)]
#[command(name = "pngme", version = "1.0", about = "PNG message encoder/decoder")]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Encode a message into a PNG file
    Encode(EncodeArgs),

    /// Decode a message from a PNG file
    Decode(DecodeArgs),

    /// Remove a message from a PNG file
    Remove(RemoveArgs),

    /// Print all messages in a PNG file
    Print(PrintArgs),
}

#[derive(Parser, Debug)]
pub struct EncodeArgs {
    #[arg(index = 1, value_parser = validate_png_file)]
    pub path: PathBuf,

    #[arg(index = 2, value_parser = validate_chunk_type)]
    pub chunk_type: String,

    #[arg(index = 3)]
    pub message: String,

    #[arg(index = 4, required = false)]
    pub output_file: Option<PathBuf>,
}

fn validate_png_file(path: &str) -> std::result::Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("File doesn't exist: {}", path.display()));
    }

    if path.extension().map(|ext| ext != "png").unwrap_or(true) {
        return Err("File must have .png extension".to_string());
    }

    Ok(path)
}

fn validate_chunk_type(chunk_type: &str) -> Result<String> {
    ChunkType::validate_str(chunk_type)
}

#[derive(Parser, Debug)]
pub struct DecodeArgs {
    #[arg(index = 1, value_parser = validate_png_file)]
    pub path: PathBuf,

    #[arg(index = 2, value_parser = validate_chunk_type)]
    pub chunk_type: String,
}

#[derive(Parser, Debug)]
pub struct RemoveArgs {
    #[arg(index = 1, value_parser = validate_png_file)]
    pub path: PathBuf,

    #[arg(index = 2, value_parser = validate_chunk_type)]
    pub chunk_type: String,

    #[arg(index = 3, required = false)]
    pub output_file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct PrintArgs {
    #[arg(index = 1, value_parser = validate_png_file)]
    pub path: PathBuf,
}
