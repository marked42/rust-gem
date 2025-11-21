use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Result, Write},
    ops::Range,
};

use clap::{Arg, ArgAction, ArgMatches, Command};
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
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
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

    // pattern is required, safe to unwrap
    let pattern = args.get_one::<String>("pattern").unwrap();
    let pattern = Regex::new(pattern).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    // input is required, safe to unwrap
    let input = args.get_one::<String>("input").unwrap();
    let reader = prepare_input(input)?;
    let words = MatchedWords::new(reader, pattern);

    let output_file = args.get_one::<String>("output");
    let (output_file, is_terminal) = prepare_output(output_file)?;
    let output_format = get_output_format(&args, is_terminal);

    output_matched_words(words, output_file, output_format)?;

    Ok(())
}

fn get_output_format(args: &ArgMatches, is_terminal: bool) -> OutputFormat {
    let line_number = args.get_flag("line_number");
    let color = args.get_flag("color");

    OutputFormat {
        color: color && is_terminal,
        line_number,
    }
}

struct OutputFormat {
    color: bool,
    line_number: bool,
}

fn prepare_input(input: &str) -> Result<Box<dyn BufRead>> {
    if input == STDIN_MARKER {
        return Ok(Box::new(io::stdin().lock()));
    }

    let file = File::open(input).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot open file '{}': {}", input, e),
        )
    })?;
    Ok(Box::new(BufReader::new(file)))
}

fn prepare_output(output_file: Option<&String>) -> Result<(Box<dyn Write>, bool)> {
    match output_file {
        Some(f) if !f.is_empty() => Ok((Box::new(File::create(f)?), false)),
        _ => Ok((Box::new(io::stdout()), true)),
    }
}

struct MatchedWords {
    iter: Lines<Box<dyn BufRead>>,
    pattern: Regex,
    line: Option<String>,
    line_no: usize,
    start: usize,
}

impl MatchedWords {
    fn new(input: Box<dyn BufRead>, pattern: Regex) -> Self {
        Self {
            iter: input.lines(),
            pattern,
            line: None,
            line_no: 0,
            start: 0,
        }
    }
}

#[derive(Debug)]
struct MatchedWord {
    line: String,
    line_no: usize,
    range: Range<usize>,
}

impl Iterator for MatchedWords {
    type Item = Result<MatchedWord>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = &self.line
                && !line.is_empty()
                && self.start < line.len() - 1
            {
                let matched = self.pattern.find_at(line, self.start);
                if let Some(matched) = matched {
                    self.start = matched.end();

                    return Some(Ok(MatchedWord {
                        line: line.clone(),
                        line_no: self.line_no,
                        range: matched.range(),
                    }));
                } else {
                    self.start = 0;
                    self.line = None;
                }
            }

            match self.iter.next() {
                Some(line) => match line {
                    Ok(line) => {
                        self.line = Some(line);
                        self.start = 0;
                        self.line_no += 1;
                    }
                    Err(e) => {
                        return Some(Err(e));
                    }
                },
                None => {
                    return None;
                }
            }
        }
    }
}

fn output_matched_words(
    matched_words: MatchedWords,
    mut output_file: Box<dyn Write>,
    output_format: OutputFormat,
) -> Result<()> {
    let mut last_line = String::new();
    let mut current_line_no: Option<usize> = None;
    let mut prev_word_end = 0usize;

    // TODO: more declarative way ?
    for word in matched_words {
        let MatchedWord {
            line,
            line_no,
            range,
        } = word?;

        match current_line_no {
            None => {
                current_line_no = Some(line_no);

                last_line = line.clone();

                // newline
                if output_format.line_number {
                    write!(output_file, "[{line_no}]")?;
                }
            }
            Some(no) => {
                if no != line_no {
                    if last_line.len() > prev_word_end {
                        write!(output_file, "{}", &last_line[prev_word_end..])?;
                    }
                    writeln!(output_file, "")?;

                    last_line = line.clone();
                    current_line_no = Some(line_no);
                    prev_word_end = 0;

                    // newline
                    if output_format.line_number {
                        write!(output_file, "[{line_no}]")?;
                    }
                }
            }
        }

        if range.start > prev_word_end {
            write!(output_file, "{}", &line[prev_word_end..range.start])?;
        }

        prev_word_end = range.end;
        if output_format.color {
            write!(output_file, "{}", line[range].red())?;
        } else {
            write!(output_file, "{}", &line[range])?;
        }
    }
    if last_line.len() > prev_word_end {
        write!(output_file, "{}", &last_line[prev_word_end..])?;
    }
    writeln!(output_file, "")?;

    Ok(())
}
