//! Value formatting helpers for memory, CPU, timestamps, etc.

use super::color;

/// Format raw bytes into a human-readable size string (KB / MB / GB).
pub fn format_memory_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        format!("{:.0} KB", kb)
    } else {
        let mb = kb / 1024.0;
        if mb < 1024.0 {
            format!("{:.1} MB", mb)
        } else {
            format!("{:.2} GB", mb / 1024.0)
        }
    }
}

/// Format a CPU usage percentage.
pub fn format_cpu(cpu: f32) -> String {
    format!("{:.1}%", cpu)
}

/// Format a duration in seconds into a human-readable string.
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

/// Format a Unix timestamp into a local datetime string.
pub fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Colorize a severity string.
pub fn format_severity(severity: &str) -> String {
    match severity {
        "CRITICAL" | "HIGH" => {
            format!("{}{}{}{}", color::BOLD, color::RED, severity, color::RESET)
        }
        "MEDIUM" => {
            format!("{}{}{}{}", color::BOLD, color::YELLOW, severity, color::RESET)
        }
        "LOW" => format!("{}{}{}", color::GREEN, severity, color::RESET),
        _ => severity.to_string(),
    }
}

/// Format a byte count for DB size display.
pub fn format_db_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else {
        format_memory_bytes(bytes)
    }
}
