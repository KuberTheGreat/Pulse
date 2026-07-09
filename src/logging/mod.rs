//! Structured logging initialization for Pulse.

use log::LevelFilter;
use env_logger::Builder;
use std::io::Write;

/// Initialize the global logger.
///
/// Respects the `PULSE_LOG` environment variable as an override.
/// Format: `[TIMESTAMP] [LEVEL] [MODULE] message`
pub fn init_logging(level: &str) {
    let filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    Builder::new()
        .filter_level(filter)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("pulse"),
                record.args(),
            )
        })
        .parse_env("PULSE_LOG")
        .init();
}
