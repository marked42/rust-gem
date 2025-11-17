use std::str::FromStr;

use crate::Result;
use crate::args::{DecodeArgs, EncodeArgs, PrintArgs, RemoveArgs};
use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;

/// Encodes a message into a PNG file and saves the result
pub fn encode(args: EncodeArgs) -> Result<()> {
    let path = &args.path;
    let mut png = Png::from_file(path)?;

    let chunk_type = ChunkType::from_str(args.chunk_type.as_str())?;
    let chunk = Chunk::new(chunk_type, args.message.into_bytes())?;
    png.append_chunk(chunk);

    let output_file = args.output_file.unwrap_or(args.path);
    png.save_to_file(output_file)
}

/// Searches for a message hidden in a PNG file and prints the message if one is found
pub fn decode(args: DecodeArgs) -> Result<()> {
    let path = &args.path;
    let png = Png::from_file(path)?;

    let chunk = png.chunk_by_type(&args.chunk_type);

    if let Some(chunk) = chunk {
        println!("{}", chunk);
    } else {
        println!("Chunk \"{}\" not found", args.chunk_type);
    }

    Ok(())
}

/// Removes a chunk from a PNG file and saves the result
pub fn remove(args: RemoveArgs) -> Result<()> {
    let path = &args.path;
    let mut png = Png::from_file(path)?;

    let chunk = png.remove_first_chunk(&args.chunk_type);
    if chunk.is_ok() {
        println!("Chunk \"{}\" deleted", args.chunk_type);
    } else {
        println!("Chunk \"{}\" not found", args.chunk_type);
    }

    let output_file = args.output_file.unwrap_or(args.path);
    png.save_to_file(output_file)
}

/// Prints all of the chunks in a PNG file
pub fn print_chunks(args: PrintArgs) -> Result<()> {
    let path = &args.path;
    let png = Png::from_file(path)?;

    // 检查是否有 chunks
    if png.chunks().is_empty() {
        println!("No chunks found in {}", path.display());
        return Ok(());
    }

    for chunk in png.chunks() {
        println!("{}", chunk)
    }

    Ok(())
}
