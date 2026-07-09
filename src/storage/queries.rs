//! Prepared-statement query helpers for the Pulse database.

use rusqlite::{params, Connection};
use crate::PulseError;

// ── Row types ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SystemMetricRow {
    pub timestamp: i64,
    pub cpu: f64,
    pub memory: u64,
    pub swap: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessMetricRow {
    pub timestamp: i64,
    pub pid: i32,
    pub process_name: String,
    pub cpu: f64,
    pub memory: u64,
}

#[derive(Debug, Clone)]
pub struct AnomalyRow {
    pub id: i64,
    pub timestamp: i64,
    pub pid: Option<i32>,
    pub process_name: String,
    pub severity: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PredictionRow {
    pub timestamp: i64,
    pub pred_type: String,
    pub confidence: f64,
    pub eta_minutes: Option<f64>,
}

// ── Inserts ──────────────────────────────────────────────────────────

pub fn insert_system_metric(
    conn: &Connection,
    timestamp: i64,
    cpu: f64,
    memory: u64,
    swap: u64,
) -> Result<(), PulseError> {
    conn.execute(
        "INSERT INTO system_metrics (timestamp, cpu, memory, swap)
         VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, cpu, memory as i64, swap as i64],
    )?;
    Ok(())
}

/// Batch-insert process metrics inside a single transaction.
pub fn insert_process_metrics_batch(
    conn: &Connection,
    rows: &[(i64, i32, &str, f64, u64)],
) -> Result<(), PulseError> {
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO process_metrics (timestamp, pid, process_name, cpu, memory)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for &(ts, pid, name, cpu, mem) in rows {
            stmt.execute(params![ts, pid, name, cpu, mem as i64])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn insert_anomaly(
    conn: &Connection,
    timestamp: i64,
    pid: Option<i32>,
    process_name: &str,
    severity: &str,
    reason: &str,
) -> Result<(), PulseError> {
    conn.execute(
        "INSERT INTO anomalies (timestamp, pid, process_name, severity, reason)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![timestamp, pid, process_name, severity, reason],
    )?;
    Ok(())
}

pub fn insert_prediction(
    conn: &Connection,
    timestamp: i64,
    pred_type: &str,
    confidence: f64,
    eta_minutes: Option<f64>,
) -> Result<(), PulseError> {
    conn.execute(
        "INSERT INTO predictions (timestamp, pred_type, confidence, eta_minutes)
         VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, pred_type, confidence, eta_minutes],
    )?;
    Ok(())
}

// ── Queries ──────────────────────────────────────────────────────────

/// Most recent `limit` process metric rows for a given process name.
/// Results are returned in **ascending** timestamp order (oldest first).
pub fn query_process_history(
    conn: &Connection,
    name: &str,
    limit: usize,
) -> Result<Vec<ProcessMetricRow>, PulseError> {
    let mut stmt = conn.prepare_cached(
        "SELECT timestamp, pid, process_name, cpu, memory
           FROM process_metrics
          WHERE process_name = ?1
          ORDER BY timestamp DESC
          LIMIT ?2",
    )?;

    let rows: Vec<ProcessMetricRow> = stmt
        .query_map(params![name, limit as i64], |row| {
            Ok(ProcessMetricRow {
                timestamp: row.get(0)?,
                pid: row.get(1)?,
                process_name: row.get(2)?,
                cpu: row.get(3)?,
                memory: row.get::<_, i64>(4)? as u64,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Reverse so oldest-first (analytics expect chronological order).
    let mut sorted = rows;
    sorted.reverse();
    Ok(sorted)
}

/// Most recent system-level metrics.
pub fn query_system_history(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SystemMetricRow>, PulseError> {
    let mut stmt = conn.prepare_cached(
        "SELECT timestamp, cpu, memory, swap
           FROM system_metrics
          ORDER BY timestamp DESC
          LIMIT ?1",
    )?;

    let rows: Vec<SystemMetricRow> = stmt
        .query_map(params![limit as i64], |row| {
            Ok(SystemMetricRow {
                timestamp: row.get(0)?,
                cpu: row.get(1)?,
                memory: row.get::<_, i64>(2)? as u64,
                swap: row.get::<_, i64>(3)? as u64,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut sorted = rows;
    sorted.reverse();
    Ok(sorted)
}

/// Most recent anomaly records.
pub fn query_anomalies(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<AnomalyRow>, PulseError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, timestamp, pid, process_name, severity, reason
           FROM anomalies
          ORDER BY timestamp DESC
          LIMIT ?1",
    )?;

    let rows: Vec<AnomalyRow> = stmt
        .query_map(params![limit as i64], |row| {
            Ok(AnomalyRow {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                pid: row.get(2)?,
                process_name: row.get(3)?,
                severity: row.get(4)?,
                reason: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Distinct tracked process names.
pub fn query_process_names(conn: &Connection) -> Result<Vec<String>, PulseError> {
    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT process_name FROM process_metrics ORDER BY process_name",
    )?;
    let rows: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Row counts across all tables — (system, process, anomaly).
pub fn query_metric_counts(
    conn: &Connection,
) -> Result<(i64, i64, i64), PulseError> {
    let sys: i64 =
        conn.query_row("SELECT COUNT(*) FROM system_metrics", [], |r| r.get(0))?;
    let proc: i64 =
        conn.query_row("SELECT COUNT(*) FROM process_metrics", [], |r| r.get(0))?;
    let anom: i64 =
        conn.query_row("SELECT COUNT(*) FROM anomalies", [], |r| r.get(0))?;
    Ok((sys, proc, anom))
}
