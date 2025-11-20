use std::{
    fs::File,
    io::{self, BufRead, BufReader, Result, Write},
};

use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use regex::Regex;

const STDIN_MARKER: &str = "-";

fn main() -> Result<()> {
    let args = Command::new("grep")
        .version("1.0")
        .about("search for patterns")
        .arg(
            Arg::new("pattern")
                .help("The pattern to search for")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("input")
                .help(format!(
                    "File to search (use '{STDIN_MARKER}' for standard input))",
                ))
                .required(true)
                .default_value(STDIN_MARKER)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("line_number")
                .help("output line number")
                .short('l')
                .long("line-number")
                .default_value("false")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .default_value("false")
                .action(ArgAction::SetTrue)
                .help("works only in terminal"),
        )
        .arg(
            Arg::new("output")
                .help("output file, default is stdout")
                .short('o')
                .long("output")
                .action(ArgAction::Set),
        )
        .get_matches();

    let pattern = args.get_one::<String>("pattern").expect("pattern is not provided");
    let pattern = Regex::new(pattern).unwrap();

    let input = args
        .get_one::<String>("input")
        .expect("input is not provided, '-' or a file path");

    let output_line_no = args.get_one::<bool>("line_number").unwrap_or(&false);
    let color = args.get_one::<bool>("color").unwrap_or(&false);

    let output_file = args.get_one::<String>("output");

    let reader = prepare_input(input)?;
    let (output_file, is_terminal) = prepare_output(output_file)?;
    let matched_lines = find_matched_lines(reader, &pattern)?;

    let color = *color && is_terminal;
    process_lines(matched_lines, color, *output_line_no, output_file)?;

    Ok(())
}

fn prepare_input(input: &str) -> Result<Box<dyn BufRead>> {
    let reader: Box<dyn BufRead> = if input == STDIN_MARKER {
        let stdin = io::stdin();
        let reader = stdin.lock();
        Box::new(reader)
    } else {
        let f = File::open(input)?;
        let reader = BufReader::new(f);
        Box::new(reader)
    };

    Ok(reader)
}

fn prepare_output(output_file: Option<&String>) -> Result<(Box<dyn Write>, bool)> {
    if let Some(f) = output_file.filter(|s| !s.is_empty()) {
        Ok((Box::new(File::create(f)?), false))
    } else {
        Ok((Box::new(io::stdout()), true))
    }
}

fn find_matched_lines(
    input: Box<dyn BufRead>,
    pattern: &Regex,
) -> Result<Vec<Vec<(String, bool)>>> {
    let mut lines = Vec::new();

    for line in input.lines() {
        let line = line?;
        if !pattern.is_match(&line) {
            continue;
        }
        let mut line_vec: Vec<(String, bool)> = Vec::new();

        let mut iter = pattern.find_iter(&line);
        let mut prev: usize = 0;
        while let Some(m) = iter.next() {
            if m.start() - prev > 0 {
                line_vec.push((line[prev..m.start()].to_string(), false));
            }
            line_vec.push((m.as_str().to_string(), true));
            prev = m.end();
        }

        if line.len() - prev > 0 {
            line_vec.push((line[prev..line.len()].to_string(), false));
        }

        lines.push(line_vec);
    }

    Ok(lines)
}

// TODO: iterator on matched line & word
fn process_lines(
    lines: Vec<Vec<(String, bool)>>,
    color: bool,
    output_line_no: bool,
    mut output_file: Box<dyn Write>,
) -> Result<()> {
    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;

        if output_line_no {
            write!(output_file, "[{line_no}]")?;
        }
        for (text, is_match) in line {
            if *is_match && color {
                write!(output_file, "{}", text.as_str().red())?;
            } else {
                write!(output_file, "{}", text)?;
            }
        }

        writeln!(output_file, "")?;
    }

    Ok(())
}
