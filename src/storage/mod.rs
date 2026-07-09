//! SQLite storage backend for Pulse.
//!
//! Replaces the old JSON-based persistence with an indexed, WAL-mode
//! SQLite database supporting months of history and fast queries.

pub mod queries;

use std::path::Path;
use rusqlite::Connection;
use log;

use crate::PulseError;

/// Handle to the Pulse SQLite database.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at `path`, enable WAL mode,
    /// and run all pending migrations.
    pub fn open(path: &Path) -> Result<Self, PulseError> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous  = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size   = -8000;",
        )?;

        let db = Database { conn };
        db.migrate()?;

        log::info!("Database opened: {}", path.display());
        Ok(db)
    }

    /// Open the database in read-only mode (for the CLI).
    pub fn open_readonly(path: &Path) -> Result<Self, PulseError> {
        if !path.exists() {
            return Err(PulseError::Database(rusqlite::Error::QueryReturnedNoRows));
        }
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
        Ok(Database { conn })
    }

    /// Create all tables and indices if they don't exist.
    fn migrate(&self) -> Result<(), PulseError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS system_metrics (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu       REAL    NOT NULL,
                memory    INTEGER NOT NULL,
                swap      INTEGER NOT NULL DEFAULT 0
             );
             CREATE INDEX IF NOT EXISTS idx_sys_ts
                ON system_metrics(timestamp);

             CREATE TABLE IF NOT EXISTS process_metrics (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    INTEGER NOT NULL,
                pid          INTEGER NOT NULL,
                process_name TEXT    NOT NULL,
                cpu          REAL    NOT NULL,
                memory       INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_proc_ts
                ON process_metrics(timestamp);
             CREATE INDEX IF NOT EXISTS idx_proc_name
                ON process_metrics(process_name);

             CREATE TABLE IF NOT EXISTS anomalies (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    INTEGER NOT NULL,
                pid          INTEGER,
                process_name TEXT    NOT NULL,
                severity     TEXT    NOT NULL,
                reason       TEXT    NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_anom_ts
                ON anomalies(timestamp);

             CREATE TABLE IF NOT EXISTS predictions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   INTEGER NOT NULL,
                pred_type   TEXT    NOT NULL,
                confidence  REAL    NOT NULL,
                eta_minutes REAL
             );",
        )?;

        log::debug!("Database migrations complete");
        Ok(())
    }

    /// Borrow the underlying connection (for query helpers).
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Delete rows older than `retain_days` from all tables.
    pub fn cleanup(&self, retain_days: u64) -> Result<usize, PulseError> {
        let cutoff = chrono::Utc::now().timestamp() - (retain_days as i64 * 86400);
        let mut total = 0usize;

        total += self.conn.execute(
            "DELETE FROM system_metrics WHERE timestamp < ?1",
            rusqlite::params![cutoff],
        )?;
        total += self.conn.execute(
            "DELETE FROM process_metrics WHERE timestamp < ?1",
            rusqlite::params![cutoff],
        )?;
        total += self.conn.execute(
            "DELETE FROM anomalies WHERE timestamp < ?1",
            rusqlite::params![cutoff],
        )?;
        total += self.conn.execute(
            "DELETE FROM predictions WHERE timestamp < ?1",
            rusqlite::params![cutoff],
        )?;

        if total > 0 {
            log::info!(
                "Cleaned up {} old records (older than {} days)",
                total,
                retain_days,
            );
        }
        Ok(total)
    }

    /// Compact the database file.
    pub fn vacuum(&self) -> Result<(), PulseError> {
        self.conn.execute_batch("VACUUM")?;
        log::info!("Database vacuumed");
        Ok(())
    }

    /// Get the size of the database file in bytes.
    pub fn db_size_bytes(path: &Path) -> u64 {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    }
}

// ── JSON → SQLite migration ─────────────────────────────────────────

/// If `~/.pulse/history.json` exists and the SQLite DB does not,
/// migrate all data into the new database and rename the old file.
pub fn migrate_from_json(config: &crate::config::PulseConfig) {
    let json_path = crate::config::PulseConfig::data_dir().join("history.json");
    let db_path = config.db_path();

    if !json_path.exists() || db_path.exists() {
        return;
    }

    log::info!("Migrating history.json → SQLite …");

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Could not read history.json: {}", e);
            return;
        }
    };

    // Minimal structs matching the old JSON schema.
    #[derive(serde::Deserialize)]
    struct OldHistory {
        processes: std::collections::HashMap<String, Vec<OldEntry>>,
    }
    #[derive(serde::Deserialize)]
    struct OldEntry {
        memory: u64,
        cpu_usage: f32,
        timestamp: i64,
    }

    let history: OldHistory = match serde_json::from_str(&content) {
        Ok(h) => h,
        Err(e) => {
            log::warn!("Could not parse history.json: {}", e);
            return;
        }
    };

    let db = match Database::open(&db_path) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Could not create database for migration: {}", e);
            return;
        }
    };

    let mut migrated = 0usize;
    for (name, entries) in &history.processes {
        let batch: Vec<(i64, i32, &str, f64, u64)> = entries
            .iter()
            .map(|e| (e.timestamp, 0i32, name.as_str(), e.cpu_usage as f64, e.memory))
            .collect();

        match queries::insert_process_metrics_batch(db.conn(), &batch) {
            Ok(()) => migrated += batch.len(),
            Err(e) => log::warn!("Failed to migrate process '{}': {}", name, e),
        }
    }

    log::info!("Migrated {} records from history.json", migrated);

    // Rename old file so it isn't re-processed.
    let backup = json_path.with_extension("json.migrated");
    std::fs::rename(&json_path, &backup).ok();
}
