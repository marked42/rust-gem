use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
