use clap::{Arg, ArgAction, ArgMatches, Command};
use regex::Regex;

use crate::Result;
use crate::error::GrepError;

pub struct AppConfig<'a> {
    pub input: &'a str,
    pub regex: Regex,
    pub output: &'a str,
    pub format: OutputFormat,
}

pub const STD_IN_OUT_MARKER: &str = "-";

pub fn create_grep_command() -> Command {
    Command::new("grep")
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
}

fn parse_output(args: &ArgMatches) -> &str {
    args.get_one::<String>("output")
        .expect("output has default value '-', safe to unwrap")
}

impl AppConfig<'_> {
    pub fn from_args(args: &ArgMatches) -> Result<AppConfig<'_>> {
        let pattern = args
            .get_one::<String>("pattern")
            .expect("pattern is required, should not be empty");
        let regex = Regex::new(pattern).map_err(GrepError::from)?;

        let input = args
            .get_one::<String>("input")
            .expect("input to has stdin as default value, safe to unwrap");
        let output = parse_output(&args);
        let format = OutputFormat::from_args(&args);

        Ok(AppConfig {
            input,
            regex,
            output,
            format,
        })
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct OutputFormat {
    pub color: bool,
    pub line_number: bool,
    pub summary: bool,
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
        let color = args.get_flag("color");

        Self::new().with_color(color).with_line_number(line_number).with_report(report)
    }
}
