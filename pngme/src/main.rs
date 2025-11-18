mod args;
mod commands;

pub use args::{Cli, Command, Parser, Result};
pub use commands::{decode, encode, print_chunks, remove};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Encode(args) => encode(args),
        Command::Decode(args) => decode(args),
        Command::Remove(args) => remove(args),
        Command::Print(args) => print_chunks(args),
    }
}
