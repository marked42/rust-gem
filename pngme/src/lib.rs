pub mod chunk;
pub mod chunk_type;
pub mod png;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
