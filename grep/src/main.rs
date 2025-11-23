use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Write},
    ops::Range,
    time::{Duration, Instant},
};

use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use humantime::format_duration;
use regex::Regex;
use thiserror::Error;

const STD_IN_OUT_MARKER: &str = "-";

type InputReader = Box<dyn BufRead>;
type OutputWriter = Box<dyn Write>;
type Result<T> = std::result::Result<T, GrepError>;

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

fn main() -> Result<()> {
    let (input, regex, output, format) = parse_args()?;

    let searcher = PatternSearcher::try_new(&input)?;
    let matches = searcher.find_matches(regex)?;

    let mut reporter = MatchReporter::try_new(output, format)?;
    reporter.report(matches)?;

    Ok(())
}

// TODO: use derive macro
fn parse_args() -> Result<(String, Regex, String, OutputFormat)> {
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
                    "File to search (use '{STD_IN_OUT_MARKER}' for standard input))",
                ))
                .required(true)
                .default_value(STD_IN_OUT_MARKER)
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
                .default_value(STD_IN_OUT_MARKER)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("summary")
                .help("output summary of found matches count and used time")
                .long("summary")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // pattern is required, safe to unwrap
    let pattern = args.get_one::<String>("pattern").unwrap();
    let regex = Regex::new(pattern).map_err(GrepError::from)?;

    // input is required, safe to unwrap
    let input = args.get_one::<String>("input").unwrap();
    let output = parse_output(&args);
    let format = OutputFormat::from_args(&args);

    // TODO: remove clone
    Ok((input.clone(), regex, output.clone(), format))
}

struct PatternSearcher {
    reader: InputReader,
}

impl PatternSearcher {
    fn try_new(input: &str) -> Result<Self> {
        let reader: InputReader = if input == STD_IN_OUT_MARKER {
            Box::new(io::stdin().lock())
        } else {
            let file = File::open(input).map_err(GrepError::from)?;
            Box::new(BufReader::new(file))
        };

        Ok(Self { reader })
    }

    fn find_matches(self, regex: Regex) -> Result<MatchIterator> {
        Ok(MatchIterator::new(self.reader, regex))
    }
}

struct MatchIterator {
    lines: Lines<InputReader>,
    pattern: Regex,
    current_line: Option<String>,
    line_no: usize,
    search_start: usize,
}

impl MatchIterator {
    fn new(input: InputReader, pattern: Regex) -> Self {
        Self {
            lines: input.lines(),
            pattern,
            current_line: None,
            line_no: 0,
            search_start: 0,
        }
    }

    fn next_match_from_current_line(&mut self) -> Option<Match> {
        let line = self.current_line.as_ref()?;

        if self.search_start >= line.len() {
            self.search_start = 0;
            self.current_line = None;
            return None;
        }

        let matched = self.pattern.find_at(line, self.search_start)?;
        self.search_start = matched.end();

        Some(Match {
            line: line.clone(),
            line_no: self.line_no,
            range: matched.range(),
        })
    }

    fn read_next_line(&mut self) -> Option<Result<String>> {
        match self.lines.next() {
            Some(Ok(line)) => {
                self.current_line = Some(line.clone());
                self.search_start = 0;
                self.line_no += 1;

                Some(Ok(line))
            }
            Some(o) => Some(o.map_err(GrepError::from)),
            None => None,
        }
    }
}

#[derive(Debug)]
struct Match {
    line: String,
    line_no: usize,
    range: Range<usize>,
}

impl Iterator for MatchIterator {
    type Item = Result<Match>;

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

struct MatchReporter {
    writer: OutputWriter,
    format: OutputFormat,
    state: LineState,
}

impl MatchReporter {
    pub fn try_new(output: String, format: OutputFormat) -> Result<Self> {
        let writer: OutputWriter = if output == STD_IN_OUT_MARKER {
            Box::new(io::stdout())
        } else {
            Box::new(File::create(output).map_err(GrepError::from)?)
        };

        Ok(Self {
            writer,
            format,
            state: LineState::new(),
        })
    }

    pub fn report(&mut self, matches: MatchIterator) -> Result<()> {
        let (count, duration) = self.report_matches(matches)?;
        self.report_summary(count, duration);

        Ok(())
    }

    fn report_matches(&mut self, matches: MatchIterator) -> Result<(usize, Duration)> {
        let start = Instant::now();
        let mut count = 0;

        for match_word in matches {
            count += 1;
            self.report_match(&match_word?)?;
        }
        self.finish_line()?;

        // line_state is changed during report, reset it so that this method can be called multiple
        // times on different matched words
        self.state.reset();

        Ok((count, start.elapsed()))
    }

    fn report_match(&mut self, word: &Match) -> Result<()> {
        if self.state.in_new_line(word.line_no) {
            self.finish_line()?;
            self.start_new_line(word)?;
        }

        let Range { start, end } = word.range;

        self.write_unmatched_part(start)?;
        self.write_matched_part(&word)?;

        self.state.advance_unmatched(end);

        Ok(())
    }

    fn finish_line(&mut self) -> Result<()> {
        if let Some(line_no) = self.state.current_line_no {
            self.write_unmatched_part(self.state.current_line.len())?;
            self.write_new_line(line_no)?;
        }

        Ok(())
    }

    fn write_unmatched_part(&mut self, pos: usize) -> Result<()> {
        if pos > self.state.unmatched_start {
            write!(self.writer, "{}", self.state.get_unmatched(pos)).map_err(GrepError::from)?;
        }

        Ok(())
    }

    fn write_new_line(&mut self, line_no: usize) -> Result<()> {
        // output newline at the end of each line starting from first line
        if line_no > 0 {
            writeln!(self.writer).map_err(GrepError::from)?;
        }

        Ok(())
    }

    fn write_matched_part(&mut self, word: &Match) -> Result<()> {
        let matched_text = &self.state.current_line[word.range.clone()];

        if self.format.color {
            write!(self.writer, "{}", matched_text.red())
        } else {
            write!(self.writer, "{}", matched_text)
        }
        .map_err(GrepError::from)
    }

    fn start_new_line(&mut self, word: &Match) -> Result<()> {
        self.state.set_line(word);
        self.write_line_number(word.line_no)?;

        Ok(())
    }

    fn write_line_number(&mut self, line_no: usize) -> Result<()> {
        // TODO: line no width should be same
        if self.format.line_number {
            write!(self.writer, "[{line_no}]").map_err(GrepError::from)?;
        }

        Ok(())
    }

    fn report_summary(&self, count: usize, duration: Duration) {
        if self.format.summary {
            println!(
                "Found {} matches in {}",
                count.to_string().green(),
                format_duration(duration)
            );
        }
    }
}

#[derive(Debug, Default)]
struct OutputFormat {
    color: bool,
    line_number: bool,
    summary: bool,
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

    fn with_report(mut self, report: bool) -> Self {
        self.summary = report;
        self
    }

    fn from_args(args: &ArgMatches) -> Self {
        let line_number = args.get_flag("line_number");
        let report = args.get_flag("summary");

        // output has default value '-', safe to unwrap
        let output = parse_output(args);
        let is_terminal = output == STD_IN_OUT_MARKER;
        let color = args.get_flag("color") && is_terminal;

        Self::new().with_color(color).with_line_number(line_number).with_report(report)
    }
}

fn parse_output(args: &ArgMatches) -> String {
    // output has default value '-', safe to unwrap
    args.get_one::<String>("output").unwrap().clone()
}

struct LineState {
    current_line_no: Option<usize>,
    current_line: String,
    unmatched_start: usize,
}

impl Default for LineState {
    fn default() -> Self {
        Self {
            current_line_no: Default::default(),
            current_line: Default::default(),
            unmatched_start: Default::default(),
        }
    }
}

impl LineState {
    fn new() -> Self {
        Default::default()
    }

    fn reset(&mut self) {
        self.current_line_no = Default::default();
        // avoid reallocation
        self.current_line.clear();
        self.unmatched_start = Default::default();
    }

    fn set_line(&mut self, word: &Match) {
        self.current_line_no = Some(word.line_no);
        self.current_line = word.line.clone();
        self.unmatched_start = 0;
    }

    fn advance_unmatched(&mut self, pos: usize) {
        self.unmatched_start = pos;
    }

    fn get_unmatched(&self, end: usize) -> &str {
        &self.current_line[self.unmatched_start..end]
    }

    fn in_new_line(&self, line_no: usize) -> bool {
        self.current_line_no != Some(line_no)
    }
}
