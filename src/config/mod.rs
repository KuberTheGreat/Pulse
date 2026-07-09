//! Configuration system for Pulse.
//!
//! Loads settings from `~/.pulse/config.toml`, falling back to sensible
//! defaults for every field.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Top-level configuration for both the daemon and CLI.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PulseConfig {
    /// Collection interval in seconds (daemon only).
    pub interval: u64,
    /// Number of days of history to retain before cleanup.
    pub history_days: u64,
    /// CPU usage percentage that triggers a critical alert.
    pub cpu_threshold: f64,
    /// Memory usage percentage that triggers a critical alert.
    pub memory_threshold: f64,
    /// Number of recent samples used by the prediction engine.
    pub prediction_window: usize,
    /// Whether desktop notifications are enabled.
    pub enable_notifications: bool,
    /// Minimum seconds between repeated notifications of the same category.
    pub notification_cooldown: u64,
    /// Custom path for the SQLite database file.
    pub db_path: Option<String>,
    /// Log level: trace, debug, info, warn, error.
    pub log_level: String,
    /// Number of top processes to collect per cycle.
    pub top_n: usize,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            interval: 5,
            history_days: 30,
            cpu_threshold: 90.0,
            memory_threshold: 85.0,
            prediction_window: 120,
            enable_notifications: true,
            notification_cooldown: 300,
            db_path: None,
            log_level: "info".into(),
            top_n: 15,
        }
    }
}

impl PulseConfig {
    // ── Paths ────────────────────────────────────────────────────────

    /// Returns `~/.pulse/`, creating it if it doesn't exist.
    pub fn data_dir() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not determine home directory");
        path.push(".pulse");
        fs::create_dir_all(&path).ok();
        path
    }

    /// Resolved database file path.
    pub fn db_path(&self) -> PathBuf {
        self.db_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::data_dir().join("pulse.db"))
    }

    /// Path to `config.toml`.
    pub fn config_path() -> PathBuf {
        Self::data_dir().join("config.toml")
    }

    /// Path to the daemon PID file.
    pub fn pid_path() -> PathBuf {
        Self::data_dir().join("daemon.pid")
    }

    // ── Loading ──────────────────────────────────────────────────────

    /// Load configuration from `~/.pulse/config.toml`.
    /// Returns defaults if the file does not exist or cannot be parsed.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    // ── Validation ───────────────────────────────────────────────────

    /// Validates config values, returning an error message on failure.
    pub fn validate(&self) -> Result<(), String> {
        if self.interval < 1 {
            return Err("interval must be >= 1 second".into());
        }
        if self.cpu_threshold < 0.0 || self.cpu_threshold > 100.0 {
            return Err("cpu_threshold must be between 0 and 100".into());
        }
        if self.memory_threshold < 0.0 || self.memory_threshold > 100.0 {
            return Err("memory_threshold must be between 0 and 100".into());
        }
        if self.top_n < 1 {
            return Err("top_n must be >= 1".into());
        }
        if self.prediction_window < 5 {
            return Err("prediction_window must be >= 5".into());
        }
        Ok(())
    }

    // ── Template generation ──────────────────────────────────────────

    /// Writes a commented default config template if none exists.
    pub fn generate_default_config() {
        let path = Self::config_path();
        if !path.exists() {
            let template = r#"# ──────────────────────────────────────────────
# Pulse Configuration
# ──────────────────────────────────────────────

# Collection interval in seconds (daemon)
# interval = 5

# Days of history to retain
# history_days = 30

# CPU usage alert threshold (%)
# cpu_threshold = 90

# Memory usage alert threshold (%)
# memory_threshold = 85

# Samples used by prediction engine
# prediction_window = 120

# Enable desktop notifications
# enable_notifications = true

# Seconds between repeated alerts
# notification_cooldown = 300

# Database path (default: ~/.pulse/pulse.db)
# db_path = ""

# Log level: trace | debug | info | warn | error
# log_level = "info"

# Top-N processes to track per cycle
# top_n = 15
"#;
            fs::write(&path, template).ok();
        }
    }
}
