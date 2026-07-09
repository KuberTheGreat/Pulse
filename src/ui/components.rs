//! Reusable terminal UI components — preserves the existing visual
//! identity while adding new display widgets.

use super::color;
use super::format;

// ── Display-level severity (different from analytics Severity) ───────

pub enum BadgeKind {
    Ok,
    Warn,
    Critical,
    Info,
    Learning,
}

// ── Banner ───────────────────────────────────────────────────────────

pub fn banner() {
    println!();
    println!(
        "  {}{}⚡ Pulse{} {}— Intelligent System Behavior Analysis{}",
        color::BOLD, color::CYAN, color::RESET, color::DIM, color::RESET,
    );
    println!("  {}{}{}", color::DIM, "━".repeat(50), color::RESET);
    println!();
}

// ── Section Header ───────────────────────────────────────────────────

pub fn section(title: &str) {
    println!();
    println!(
        "{}{}  {}  {}",
        color::BOLD, color::CYAN, title, color::RESET,
    );
    println!(
        "{}{}{}",
        color::DIM,
        "─".repeat(title.len() + 4),
        color::RESET,
    );
}

// ── Key–Value ────────────────────────────────────────────────────────

pub fn kv(key: &str, value: &str) {
    println!(
        "  {}{}{:<14}{} {}",
        color::BOLD,
        color::WHITE,
        std::format!("{}:", key),
        color::RESET,
        value,
    );
}

// ── Progress Bar ─────────────────────────────────────────────────────

pub fn bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);

    let bar_color = if percent <= 50.0 {
        color::GREEN
    } else if percent <= 80.0 {
        color::YELLOW
    } else {
        color::RED
    };

    let bar_str = std::format!(
        "{}{}{}{}",
        bar_color,
        "█".repeat(filled),
        color::DIM,
        "░".repeat(width - filled),
    );
    std::format!(
        "{}{} {}{:>5.1}%{}",
        bar_str, color::RESET, color::BOLD, percent, color::RESET,
    )
}

// ── Sparkline ────────────────────────────────────────────────────────

pub fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let ticks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < f64::EPSILON {
        return std::format!(
            "{}{}{}",
            color::GREEN,
            ticks[0].to_string().repeat(values.len()),
            color::RESET,
        );
    }

    values
        .iter()
        .map(|v| {
            let norm = (v - min) / (max - min);
            let idx = (norm * (ticks.len() as f64 - 1.0)).round() as usize;
            let c = if norm <= 0.4 {
                color::GREEN
            } else if norm <= 0.7 {
                color::YELLOW
            } else {
                color::RED
            };
            std::format!("{}{}{}", c, ticks[idx], color::RESET)
        })
        .collect()
}

// ── Labeled Sparkline (graph + numeric stats) ────────────────────────

pub fn labeled_sparkline<F>(label: &str, values: &[f64], current: f64, format_value: F)
where
    F: Fn(f64) -> String,
{
    if values.is_empty() {
        kv(label, &std::format!("{}no data{}", color::DIM, color::RESET));
        return;
    }

    let graph = sparkline(values);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = values.iter().sum::<f64>() / values.len() as f64;

    println!(
        "  {}{}{:<14}{} {}",
        color::BOLD,
        color::WHITE,
        std::format!("{}:", label),
        color::RESET,
        graph,
    );
    println!(
        "  {:>14}  {}min{} {}  {}avg{} {}  {}max{} {}  {}now{} {}",
        "",
        color::DIM, color::RESET, format_value(min),
        color::DIM, color::RESET, format_value(avg),
        color::DIM, color::RESET, format_value(max),
        color::BOLD, color::RESET, format_value(current),
    );
}

// ── Status Badge ─────────────────────────────────────────────────────

pub fn status_badge(kind: BadgeKind) -> String {
    match kind {
        BadgeKind::Ok => std::format!(
            "{}{}  ✓ OK  {}",
            color::BG_GREEN, color::BOLD, color::RESET,
        ),
        BadgeKind::Warn => std::format!(
            "{}{}  ⚠ WARN  {}",
            color::BG_YELLOW, color::BOLD, color::RESET,
        ),
        BadgeKind::Critical => std::format!(
            "{}{}  ✗ CRITICAL  {}",
            color::BG_RED, color::BOLD, color::RESET,
        ),
        BadgeKind::Info => std::format!(
            "{}{}ℹ  INFO{}",
            color::CYAN, color::BOLD, color::RESET,
        ),
        BadgeKind::Learning => std::format!(
            "{}{}⏳ LEARNING{}",
            color::BLUE, color::BOLD, color::RESET,
        ),
    }
}

// ── Prediction Card ──────────────────────────────────────────────────

pub fn prediction_card(
    eta_minutes: f64,
    confidence: f64,
    trend_label: &str,
    reason: &str,
) {
    section("Prediction");
    kv("ETA", &std::format!(
        "{}{}{:.0} minutes{}",
        color::BOLD, color::YELLOW, eta_minutes, color::RESET,
    ));
    kv("Confidence", &std::format!("{}", bar(confidence * 100.0, 15)));
    kv("Trend", trend_label);
    kv("Reason", &std::format!("{}{}{}", color::DIM, reason, color::RESET));
}

// ── Anomaly Table ────────────────────────────────────────────────────

pub fn anomaly_table(rows: &[crate::storage::queries::AnomalyRow]) {
    if rows.is_empty() {
        println!("  {}No anomalies recorded.{}", color::DIM, color::RESET);
        return;
    }

    // Header
    println!(
        "  {}{}{:<20} {:<16} {:<10} {}{}",
        color::BOLD, color::WHITE,
        "Timestamp", "Process", "Severity", "Reason",
        color::RESET,
    );
    println!("  {}{}{}", color::DIM, "─".repeat(76), color::RESET);

    for row in rows {
        let ts = format::format_timestamp(row.timestamp);
        let sev = format::format_severity(&row.severity);
        let reason_short = if row.reason.len() > 30 {
            std::format!("{}…", &row.reason[..29])
        } else {
            row.reason.clone()
        };
        println!("  {:<20} {:<16} {:<10} {}", ts, row.process_name, sev, reason_short);
    }
}

// ── History Table ────────────────────────────────────────────────────

pub fn process_history_table(rows: &[crate::storage::queries::ProcessMetricRow]) {
    if rows.is_empty() {
        println!("  {}No history data available.{}", color::DIM, color::RESET);
        return;
    }

    println!(
        "  {}{}{:<20} {:<8} {:<10} {:<10}{}",
        color::BOLD, color::WHITE,
        "Timestamp", "PID", "Memory", "CPU",
        color::RESET,
    );
    println!("  {}{}{}", color::DIM, "─".repeat(52), color::RESET);

    for row in rows {
        let ts = format::format_timestamp(row.timestamp);
        let mem = format::format_memory_bytes(row.memory);
        let cpu = format::format_cpu(row.cpu as f32);
        println!("  {:<20} {:<8} {:<10} {:<10}", ts, row.pid, mem, cpu);
    }
}

pub fn system_history_table(rows: &[crate::storage::queries::SystemMetricRow]) {
    if rows.is_empty() {
        println!("  {}No system history available.{}", color::DIM, color::RESET);
        return;
    }

    println!(
        "  {}{}{:<20} {:<10} {:<12} {:<12}{}",
        color::BOLD, color::WHITE,
        "Timestamp", "CPU", "Memory", "Swap",
        color::RESET,
    );
    println!("  {}{}{}", color::DIM, "─".repeat(58), color::RESET);

    for row in rows {
        let ts = format::format_timestamp(row.timestamp);
        let cpu = std::format!("{:.1}%", row.cpu);
        let mem = format::format_memory_bytes(row.memory);
        let swap = format::format_memory_bytes(row.swap);
        println!("  {:<20} {:<10} {:<12} {:<12}", ts, cpu, mem, swap);
    }
}

// ── Doctor Report ────────────────────────────────────────────────────

pub fn doctor_report(
    config_path: &str,
    db_path: &str,
    db_size: u64,
    daemon_pid: Option<u32>,
    sys_metrics: i64,
    proc_metrics: i64,
    anomaly_count: i64,
    tracked_processes: usize,
) {
    section("Health Check");

    kv("Config", config_path);
    kv("Database", db_path);
    kv("DB Size", &format::format_db_size(db_size));

    let daemon_status = match daemon_pid {
        Some(pid) => std::format!(
            "{} running (PID {})",
            status_badge(BadgeKind::Ok), pid,
        ),
        None => std::format!(
            "{} not running",
            status_badge(BadgeKind::Warn),
        ),
    };
    kv("Daemon", &daemon_status);

    section("Data");
    kv("System Samples", &sys_metrics.to_string());
    kv("Process Samples", &proc_metrics.to_string());
    kv("Anomalies", &anomaly_count.to_string());
    kv("Tracked Procs", &tracked_processes.to_string());
}

// ── Footer ───────────────────────────────────────────────────────────

pub fn footer() {
    println!();
    println!(
        "  {}{}Tip:{} Run {}pulse doctor{} for a system health check. \
         Use {}pulse daemon start{} to begin background collection.",
        color::DIM, color::CYAN, color::RESET,
        color::BOLD, color::RESET,
        color::BOLD, color::RESET,
    );
    println!();
}
