use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::ops::Range;

use regex::Regex;

use crate::Result;
use crate::command::STD_IN_OUT_MARKER;
use crate::error::GrepError;

pub type InputReader = Box<dyn BufRead>;

pub struct PatternSearcher {
    reader: InputReader,
}

impl PatternSearcher {
    pub fn try_new(input: &str) -> Result<Self> {
        let reader: InputReader = if input == STD_IN_OUT_MARKER {
            Box::new(io::stdin().lock())
        } else {
            let file = File::open(input).map_err(GrepError::from)?;
            Box::new(BufReader::new(file))
        };

        Ok(Self { reader })
    }

    pub fn find_matches<'a>(self, regex: &'a Regex) -> Result<MatchIterator<'a>> {
        Ok(MatchIterator::new(self.reader, regex))
    }
}

#[derive(Debug)]
pub struct Match {
    pub line: String,
    pub line_no: usize,
    pub range: Range<usize>,
}

pub struct MatchIterator<'a> {
    pub lines: Lines<InputReader>,
    pub pattern: &'a Regex,
    pub current_line: Option<String>,
    pub line_no: usize,
    pub search_start: usize,
}

impl<'a> MatchIterator<'a> {
    fn new(input: InputReader, pattern: &'a Regex) -> Self {
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

impl Iterator for MatchIterator<'_> {
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
