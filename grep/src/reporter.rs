use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::Range;
use std::time::Duration;
use std::time::Instant;

use colored::Colorize;
use humantime::format_duration;

use crate::OutputFormat;
use crate::Result;
use crate::command::STD_IN_OUT_MARKER;
use crate::error::GrepError;
use crate::searcher::{Match, MatchIterator};

pub type OutputWriter = Box<dyn Write>;

pub struct MatchReporter {
    writer: OutputWriter,
    format: OutputFormat,
    state: LineState,
    total_line_count: usize,
}

impl MatchReporter {
    pub fn try_new(output: &str, format: OutputFormat) -> Result<Self> {
        let writer: OutputWriter = if output == STD_IN_OUT_MARKER {
            Box::new(io::stdout())
        } else {
            Box::new(File::create(output).map_err(GrepError::from)?)
        };

        Ok(Self {
            writer,
            format,
            state: LineState::new(),
            total_line_count: 0,
        })
    }

    pub fn report(&mut self, matches: MatchIterator) -> Result<()> {
        self.total_line_count = matches.total_line_count;

        let (count, duration) = self.report_matches(matches)?;
        self.report_summary(count, duration);

        self.total_line_count = Default::default();
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
        // borrow &self in an temporary expression and extract nested fields, avoid conflict
        // with mut self below
        let (line_no, line_len) = if let Some((line_no, line)) = &self.state.current {
            (*line_no, line.len())
        } else {
            return Ok(());
        };

        self.write_unmatched_part(line_len)?;
        self.write_new_line(line_no)?;

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
        let matched_text = &self.state.get_sub_str(word.range.clone());

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
        if self.format.line_number {
            write!(
                self.writer,
                "[{line_no:>width$}]",
                width = self.line_number_width()
            )
            .map_err(GrepError::from)?;
        }

        Ok(())
    }

    fn report_summary(&self, count: usize, duration: Duration) {
        if self.format.summary || count != 0 {
            println!(
                "Found {} matches in {}",
                count.to_string().green(),
                format_duration(duration)
            );
        }
    }

    fn line_number_width(&self) -> usize {
        self.total_line_count.checked_ilog10().unwrap_or(0) as usize + 1
    }
}

pub struct LineState {
    current: Option<(usize, String)>,
    unmatched_start: usize,
}

impl Default for LineState {
    fn default() -> Self {
        Self {
            current: Default::default(),
            unmatched_start: Default::default(),
        }
    }
}

impl LineState {
    fn new() -> Self {
        Default::default()
    }

    fn reset(&mut self) {
        self.current = Default::default();
        self.unmatched_start = Default::default();
    }

    fn set_line(&mut self, word: &Match) {
        self.current = Some((word.line_no, word.line.clone()));
        self.unmatched_start = 0;
    }

    fn advance_unmatched(&mut self, pos: usize) {
        self.unmatched_start = pos;
    }

    fn get_sub_str(&self, range: Range<usize>) -> &str {
        if let Some((_, current_line)) = &self.current {
            &current_line[range.clone()]
        } else {
            ""
        }
    }

    fn get_unmatched(&self, end: usize) -> &str {
        self.get_sub_str(self.unmatched_start..end)
    }

    fn in_new_line(&self, line_no: usize) -> bool {
        self.current.as_ref().map_or(true, |(l, _)| *l != line_no)
    }
}
