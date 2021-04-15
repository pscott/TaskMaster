pub mod client;
pub mod command;
pub mod config;
pub mod server;
mod threadpool;

/// Default address and port of the taskmaster daemon.
pub const DEFAULT_ADDR: &str = "127.0.0.1:2121";
