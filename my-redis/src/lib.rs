pub mod frame;

pub mod delay;

pub mod connection;

pub mod interval;

pub type Error = Box<dyn std::error::Error + Send + Sync>;