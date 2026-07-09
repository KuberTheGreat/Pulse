//! Pulse — Intelligent System Behavior Analysis Platform
//!
//! Library crate exposing all modules for use by the `pulse` CLI and
//! `pulse-daemon` binaries.

pub mod config;
pub mod logging;
pub mod storage;
pub mod collector;
pub mod analytics;
pub mod alerts;
pub mod ui;
pub mod daemon;

// ── Crate-wide error type ────────────────────────────────────────────

/// Unified error type used throughout the Pulse crate.
#[derive(Debug)]
pub enum PulseError {
    Io(std::io::Error),
    Database(rusqlite::Error),
    Config(String),
    Process(String),
}

impl std::fmt::Display for PulseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseError::Io(e) => write!(f, "IO error: {}", e),
            PulseError::Database(e) => write!(f, "Database error: {}", e),
            PulseError::Config(msg) => write!(f, "Config error: {}", msg),
            PulseError::Process(msg) => write!(f, "Process error: {}", msg),
        }
    }
}

impl std::error::Error for PulseError {}

impl From<std::io::Error> for PulseError {
    fn from(e: std::io::Error) -> Self {
        PulseError::Io(e)
    }
}

impl From<rusqlite::Error> for PulseError {
    fn from(e: rusqlite::Error) -> Self {
        PulseError::Database(e)
    }
}
