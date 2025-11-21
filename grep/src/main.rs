use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Result, Write},
    ops::Range,
};

use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use regex::Regex;

const STDIN_MARKER: &str = "-";

type InputReader = Box<dyn BufRead>;
type OutputWriter = Box<dyn Write>;

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
    let (mut output, is_terminal) = prepare_output(output_file)?;
    let output_format = OutputFormat::from_args(&args, is_terminal);

    output_matched_words(words, &mut output, &output_format)?;

    Ok(())
}

#[derive(Debug, Default)]
struct OutputFormat {
    color: bool,
    line_number: bool,
}

impl OutputFormat {
    fn new() -> Self {
        Self::default()
    }

    fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    fn with_line_number(mut self, line_number: bool) -> Self {
        self.line_number = line_number;
        self
    }

    fn from_args(args: &ArgMatches, is_terminal: bool) -> Self {
        let line_number = args.get_flag("line_number");
        let color = args.get_flag("color") && is_terminal;

        Self::new().with_color(color).with_line_number(line_number)
    }
}

fn prepare_input(input: &str) -> Result<InputReader> {
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

fn prepare_output(output_file: Option<&String>) -> Result<(OutputWriter, bool)> {
    match output_file {
        Some(f) if !f.is_empty() => Ok((Box::new(File::create(f)?), false)),
        _ => Ok((Box::new(io::stdout()), true)),
    }
}

struct MatchedWords {
    iter: Lines<InputReader>,
    pattern: Regex,
    current_line: Option<String>,
    line_no: usize,
    search_start: usize,
}

impl MatchedWords {
    fn new(input: InputReader, pattern: Regex) -> Self {
        Self {
            iter: input.lines(),
            pattern,
            current_line: None,
            line_no: 0,
            search_start: 0,
        }
    }

    fn next_match_from_current_line(&mut self) -> Option<MatchedWord> {
        let line = self.current_line.as_ref()?;

        if self.search_start >= line.len() {
            self.search_start = 0;
            self.current_line = None;
            return None;
        }

        let matched = self.pattern.find_at(line, self.search_start)?;
        self.search_start = matched.end();

        Some(MatchedWord {
            line: line.clone(),
            line_no: self.line_no,
            range: matched.range(),
        })
    }

    fn read_next_line(&mut self) -> Option<Result<String>> {
        match self.iter.next() {
            Some(Ok(line)) => {
                self.current_line = Some(line.clone());
                self.search_start = 0;
                self.line_no += 1;

                Some(Ok(line))
            }
            other => other,
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
            if let Some(matched) = self.next_match_from_current_line() {
                return Some(Ok(matched));
            }

            match self.read_next_line() {
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}

struct LineState {
    current_line_no: Option<usize>,
    current_line: String,
    last_word_end: usize,
}

impl LineState {
    fn new() -> Self {
        Self {
            current_line: String::new(),
            current_line_no: None,
            last_word_end: 0,
        }
    }

    fn reset(&mut self) {
        self.current_line_no = None;
        self.current_line.clear();
        self.last_word_end = 0;
    }

    fn process_word(
        &mut self,
        word: MatchedWord,
        // TODO: is &mut dyn Write better ?
        output: &mut OutputWriter,
        format: &OutputFormat,
    ) -> Result<()> {
        if self.current_line_no != Some(word.line_no) {
            self.finish_line(output)?;
            self.start_new_line(word.line_no, &word.line, output, format)?;
        }

        if word.range.start > self.last_word_end {
            write!(
                output,
                "{}",
                &self.current_line[self.last_word_end..word.range.start]
            )?;
        }

        let matched_text = &self.current_line[word.range.clone()];
        if format.color {
            write!(output, "{}", matched_text.red())?;
        } else {
            write!(output, "{}", matched_text)?;
        }

        self.last_word_end = word.range.end;
        Ok(())
    }

    fn start_new_line(
        &mut self,
        line_no: usize,
        line: &str,
        output: &mut OutputWriter,
        format: &OutputFormat,
    ) -> Result<()> {
        self.current_line_no = Some(line_no);
        self.current_line = line.to_string();
        self.last_word_end = 0;

        // TODO: line no width should be same
        if format.line_number {
            write!(output, "[{line_no}]")?
        }

        Ok(())
    }

    fn finish_line(&mut self, output: &mut OutputWriter) -> Result<()> {
        if let Some(line_no) = self.current_line_no {
            if self.last_word_end < self.current_line.len() {
                write!(output, "{}", &self.current_line[self.last_word_end..])?;
            }

            // output newline at the end of each line starting from first line
            if line_no > 0 {
                writeln!(output)?;
            }

            self.reset();
        }

        Ok(())
    }
}

fn output_matched_words(
    matched_words: MatchedWords,
    output: &mut OutputWriter,
    output_format: &OutputFormat,
) -> Result<()> {
    let mut current_line_state = LineState::new();

    for word_result in matched_words {
        let word = word_result?;
        current_line_state.process_word(word, output, output_format)?;
    }
    current_line_state.finish_line(output)?;

    Ok(())
}
