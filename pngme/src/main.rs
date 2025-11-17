mod args;
mod commands;

pub use args::{Cli, Commands};
pub use clap::Parser;
pub use commands::{decode, encode, print_chunks, remove};
use pngme::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.commands {
        Commands::Encode(args) => encode(args),
        Commands::Decode(args) => decode(args),
        Commands::Remove(args) => remove(args),
        Commands::Print(args) => print_chunks(args),
    }
}
