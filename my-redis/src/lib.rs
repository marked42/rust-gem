pub mod frame;

pub mod connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;