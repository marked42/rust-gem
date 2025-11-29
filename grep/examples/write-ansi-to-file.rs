use colored::Colorize;
use std::io;
use std::io::Write;
use std::io::stdout;

fn main() -> io::Result<()> {
    let text = "-";

    let mut out = stdout().lock();
    writeln!(out, "{}", text.red())?;

    // colored crate will format ansi code in string when detecting output is terminal
    // no color ansi code when run by 'cargo run | out.txt', stdout is piped to a file
    let s = text.red().to_string();
    writeln!(out, "{}", s)?;

    writeln!(out, "\n\x1b[31mx\x1b[0m")?;

    Ok(())
}
