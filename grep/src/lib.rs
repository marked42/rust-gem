mod command;
mod error;
mod reporter;
mod searcher;

pub use command::{AppConfig, OutputFormat, create_grep_command};
pub use error::GrepError;
pub use reporter::MatchReporter;
pub use searcher::PatternSearcher;

pub type Result<T> = std::result::Result<T, GrepError>;
