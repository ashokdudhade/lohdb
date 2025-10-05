//! LohDB - A simple, fast, embeddable key-value store
//! 
//! Features:
//! - Durability via Write-Ahead Log (WAL)
//! - In-memory indexing for fast reads
//! - Change subscriptions/notifications
//! - Pluggable storage backends
//! - Crash recovery

pub mod db;
pub mod cli;

pub use db::{Database, DatabaseConfig, StorageEngine, Operation, ChangeEvent};
pub use cli::run_cli;

/// Result type used throughout the library
pub type Result<T> = anyhow::Result<T>;