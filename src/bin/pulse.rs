//! `pulse` — CLI front-end for the Pulse observability platform.

use clap::{Parser, Subcommand};

use pulse::config::PulseConfig;
use pulse::storage::Database;
use pulse::storage::queries;
use pulse::collector::SystemCollector;
use pulse::analytics::{anomaly, trend, prediction, explain};
use pulse::ui::color;
use pulse::ui::format;
use pulse::ui::components;

// ── CLI argument definitions ─────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "pulse",
    version = env!("CARGO_PKG_VERSION"),
    about = "⚡ Pulse — Intelligent System Behavior Analysis",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// System overview (default when no subcommand is given)
    Status,
    /// Inspect a process by name or PID
    Process {
        /// Process name or numeric PID
        target: String,
    },
    /// Show recent metric history
    History {
        /// Filter by process name
        #[arg(long)]
        process: Option<String>,
        /// Number of entries to display
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// List detected anomalies
    Anomalies {
        /// Number of entries to display
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Process health summary
    Summary {
        /// Process name
        process: String,
    },
    /// System health check
    Doctor,
    /// Show active configuration
    Config,
    /// Daemon management
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Show version
    Version,
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the background daemon
    Start,
    /// Stop the running daemon
    Stop,
    /// Check daemon status
    Status,
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let config = PulseConfig::load();

    components::banner();

    match cli.command.unwrap_or(Commands::Status) {
        Commands::Status => cmd_status(&config),
        Commands::Process { target } => cmd_process(&config, &target),
        Commands::History { process, limit } => cmd_history(&config, process.as_deref(), limit),
        Commands::Anomalies { limit } => cmd_anomalies(&config, limit),
        Commands::Summary { process } => cmd_summary(&config, &process),
        Commands::Doctor => cmd_doctor(&config),
        Commands::Config => cmd_config(&config),
        Commands::Daemon { action } => cmd_daemon(action),
        Commands::Version => cmd_version(),
    }
}

// ── Subcommand implementations ───────────────────────────────────────

fn cmd_status(config: &PulseConfig) {
    let mut collector = SystemCollector::new();
    collector.refresh();
    let sys = collector.system_snapshot();

    components::section("System Status");

    let mem_pct = (sys.used_memory as f64 / sys.total_memory.max(1) as f64) * 100.0;
    components::kv(
        "Memory",
        &std::format!(
            "{} / {}  {}",
            format::format_memory_bytes(sys.used_memory),
            format::format_memory_bytes(sys.total_memory),
            components::bar(mem_pct, 20),
        ),
    );

    components::kv("CPU", &components::bar(sys.cpu_usage as f64, 20));

    if sys.swap_total > 0 {
        let swap_pct = (sys.swap_used as f64 / sys.swap_total.max(1) as f64) * 100.0;
        components::kv(
            "Swap",
            &std::format!(
                "{} / {}  {}",
                format::format_memory_bytes(sys.swap_used),
                format::format_memory_bytes(sys.swap_total),
                components::bar(swap_pct, 20),
            ),
        );
    }

    // If DB exists, show recent system history sparkline
    let db_path = config.db_path();
    if db_path.exists() {
        if let Ok(db) = Database::open_readonly(&db_path) {
            if let Ok(history) = queries::query_system_history(db.conn(), 20) {
                if !history.is_empty() {
                    let cpu_vals: Vec<f64> = history.iter().map(|r| r.cpu).collect();
                    let mem_vals: Vec<f64> = history.iter().map(|r| r.memory as f64).collect();

                    components::section("Recent Trends");
                    components::labeled_sparkline(
                        "CPU History",
                        &cpu_vals,
                        sys.cpu_usage as f64,
                        |v| std::format!("{:.1}%", v),
                    );
                    components::labeled_sparkline(
                        "Mem History",
                        &mem_vals,
                        sys.used_memory as f64,
                        |v| format::format_memory_bytes(v as u64),
                    );
                }
            }
        }
    }

    components::footer();
}

fn cmd_process(config: &PulseConfig, target: &str) {
    let mut collector = SystemCollector::new();
    collector.refresh();

    // Try as PID first, then as name
    let processes = if let Ok(pid) = target.parse::<i32>() {
        collector
            .find_process_by_pid(pid)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        collector.find_process(target)
    };

    if processes.is_empty() {
        println!(
            "  {}{}No process matching '{}' found.{}",
            color::RED, color::BOLD, target, color::RESET,
        );
        return;
    }

    // Open DB for history (read-write so we can record the sample)
    let db_path = config.db_path();
    let db = Database::open(&db_path).ok();

    for p in &processes {
        components::section(&std::format!("Process: {} (PID {})", p.name, p.pid));

        // Record this sample into the DB if available
        if let Some(ref db) = db {
            let now = chrono::Utc::now().timestamp();
            let _ = queries::insert_process_metrics_batch(
                db.conn(),
                &[(now, p.pid, p.name.as_str(), p.cpu_usage as f64, p.memory)],
            );
        }

        // Load history for sparklines + analytics
        let history = db
            .as_ref()
            .and_then(|d| {
                queries::query_process_history(d.conn(), &p.name, config.prediction_window)
                    .ok()
            })
            .unwrap_or_default();

        // ── Sparklines with stats ────────────────────────────────
        if !history.is_empty() {
            let mem_vals: Vec<f64> =
                history.iter().rev().take(20).map(|h| h.memory as f64).collect();
            let cpu_vals: Vec<f64> =
                history.iter().rev().take(20).map(|h| h.cpu).collect();

            components::labeled_sparkline(
                "Memory",
                &mem_vals,
                p.memory as f64,
                |v| format::format_memory_bytes(v as u64),
            );
            components::labeled_sparkline(
                "CPU",
                &cpu_vals,
                p.cpu_usage as f64,
                |v| format::format_cpu(v as f32),
            );
        } else {
            components::kv(
                "Memory",
                &std::format!("{}", format::format_memory_bytes(p.memory)),
            );
            components::kv("CPU", &components::bar(p.cpu_usage as f64, 20));
        }

        // ── Trend analysis ───────────────────────────────────────
        if let Some(trend_result) = trend::detect_trend(&history, config.prediction_window) {
            components::section("Trend Analysis");

            let mem_label = match trend_result.memory_trend {
                trend::TrendKind::Increasing => std::format!(
                    "{}{}📈 Increasing{} {}(possible leak){}",
                    color::BOLD, color::RED, color::RESET,
                    color::DIM, color::RESET,
                ),
                trend::TrendKind::Decreasing => std::format!(
                    "{}{}📉 Decreasing{}",
                    color::BOLD, color::GREEN, color::RESET,
                ),
                trend::TrendKind::Stable => std::format!(
                    "{}{}➖ Stable{}",
                    color::BOLD, color::CYAN, color::RESET,
                ),
            };
            components::kv("Memory", &mem_label);

            let cpu_label = match trend_result.cpu_trend {
                trend::TrendKind::Increasing => std::format!(
                    "{}{}📈 Increasing{}",
                    color::BOLD, color::RED, color::RESET,
                ),
                trend::TrendKind::Decreasing => std::format!(
                    "{}{}📉 Decreasing{}",
                    color::BOLD, color::GREEN, color::RESET,
                ),
                trend::TrendKind::Stable => std::format!(
                    "{}{}➖ Stable{}",
                    color::BOLD, color::CYAN, color::RESET,
                ),
            };
            components::kv("CPU", &cpu_label);
            components::kv(
                "Samples",
                &std::format!("{}{}{}", color::DIM, trend_result.sample_count, color::RESET),
            );

            // ── Prediction ───────────────────────────────────────
            if let Some(pred) = prediction::build_prediction(
                &history,
                p.memory,
                trend_result.memory_slope,
                trend_result.memory_trend,
            ) {
                if let Some(eta) = pred.minutes_to_anomaly {
                    let trend_label = match pred.trend {
                        trend::TrendKind::Increasing => std::format!("{}{}Increasing{}", color::BOLD, color::RED, color::RESET),
                        trend::TrendKind::Decreasing => std::format!("{}{}Decreasing{}", color::BOLD, color::GREEN, color::RESET),
                        trend::TrendKind::Stable => std::format!("{}{}Stable{}", color::BOLD, color::CYAN, color::RESET),
                    };
                    components::prediction_card(eta, pred.confidence, &trend_label, &pred.reason);
                }
            }
        }

        // ── Anomaly detection ────────────────────────────────────
        if history.len() >= 5 {
            if let Some(result) = anomaly::detect_anomaly(&history, p.memory, p.cpu_usage, 2.0) {
                if result.memory_anomaly || result.cpu_anomaly {
                    components::section("Anomaly Detected");
                    println!("  {}", components::status_badge(components::BadgeKind::Critical));

                    if result.memory_anomaly {
                        println!(
                            "  {}{}Memory anomaly{} (z-score: {}{:.2}{})",
                            color::BOLD, color::RED, color::RESET,
                            color::YELLOW, result.memory_score, color::RESET,
                        );
                    }
                    if result.cpu_anomaly {
                        println!(
                            "  {}{}CPU anomaly{} (z-score: {}{:.2}{})",
                            color::BOLD, color::RED, color::RESET,
                            color::YELLOW, result.cpu_score, color::RESET,
                        );
                    }

                    println!();
                    let explanations = explain::explain_anomaly(&p.name, &result);
                    for line in explanations {
                        println!(
                            "  {}{}→ {}{}",
                            color::DIM, color::YELLOW, line, color::RESET,
                        );
                    }
                } else {
                    components::section("Behavior");
                    println!(
                        "  {} Normal — no anomalies detected.",
                        components::status_badge(components::BadgeKind::Ok),
                    );
                }
            }
        } else {
            components::section("Behavior");
            println!(
                "  {} Building baseline ({} / 5 samples) …",
                components::status_badge(components::BadgeKind::Learning),
                history.len(),
            );
        }

        println!();
    }

    components::footer();
}

fn cmd_history(config: &PulseConfig, process: Option<&str>, limit: usize) {
    let db_path = config.db_path();
    let db = match Database::open_readonly(&db_path) {
        Ok(d) => d,
        Err(_) => {
            println!("  {}{}No database found. Start the daemon first.{}", color::YELLOW, color::BOLD, color::RESET);
            return;
        }
    };

    if let Some(name) = process {
        components::section(&std::format!("History: {}", name));
        match queries::query_process_history(db.conn(), name, limit) {
            Ok(rows) => components::process_history_table(&rows),
            Err(e) => println!("  {}Error: {}{}", color::RED, e, color::RESET),
        }
    } else {
        components::section("System History");
        match queries::query_system_history(db.conn(), limit) {
            Ok(rows) => components::system_history_table(&rows),
            Err(e) => println!("  {}Error: {}{}", color::RED, e, color::RESET),
        }
    }
}

fn cmd_anomalies(config: &PulseConfig, limit: usize) {
    let db_path = config.db_path();
    let db = match Database::open_readonly(&db_path) {
        Ok(d) => d,
        Err(_) => {
            println!("  {}{}No database found. Start the daemon first.{}", color::YELLOW, color::BOLD, color::RESET);
            return;
        }
    };

    components::section("Detected Anomalies");
    match queries::query_anomalies(db.conn(), limit) {
        Ok(rows) => components::anomaly_table(&rows),
        Err(e) => println!("  {}Error: {}{}", color::RED, e, color::RESET),
    }
}

fn cmd_summary(config: &PulseConfig, process: &str) {
    let db_path = config.db_path();
    let db = match Database::open_readonly(&db_path) {
        Ok(d) => d,
        Err(_) => {
            println!("  {}{}No database found. Start the daemon first.{}", color::YELLOW, color::BOLD, color::RESET);
            return;
        }
    };

    let history = match queries::query_process_history(db.conn(), process, config.prediction_window) {
        Ok(h) => h,
        Err(_) => {
            println!("  {}{}No history for '{}'{}", color::RED, color::BOLD, process, color::RESET);
            return;
        }
    };

    if history.len() < 5 {
        println!(
            "  {}{}Not enough data to summarize '{}' ({} samples, need 5){}",
            color::YELLOW, color::BOLD, process, history.len(), color::RESET,
        );
        return;
    }

    components::section(&std::format!("Summary: {}", process));

    let avg_mem = history.iter().map(|h| h.memory as f64).sum::<f64>() / history.len() as f64;
    let avg_cpu = history.iter().map(|h| h.cpu).sum::<f64>() / history.len() as f64;

    components::kv("Avg Memory", &format::format_memory_bytes(avg_mem as u64));
    components::kv("Avg CPU", &format::format_cpu(avg_cpu as f32));
    components::kv(
        "Samples",
        &std::format!("{}{}{}", color::CYAN, history.len(), color::RESET),
    );

    // Anomaly rate
    let anomalies = history
        .windows(5)
        .filter(|window| {
            anomaly::detect_anomaly(
                window,
                window.last().unwrap().memory,
                window.last().unwrap().cpu as f32,
                2.0,
            )
            .map(|r| r.memory_anomaly || r.cpu_anomaly)
            .unwrap_or(false)
        })
        .count();

    let freq = anomalies as f64 / history.len() as f64;

    let rate_color = if freq > 0.3 {
        color::RED
    } else if freq > 0.1 {
        color::YELLOW
    } else {
        color::GREEN
    };

    components::kv(
        "Anomaly Rate",
        &std::format!("{}{}{:.0}%{}", color::BOLD, rate_color, freq * 100.0, color::RESET),
    );

    let (risk, badge) = if freq > 0.3 {
        ("HIGH", components::status_badge(components::BadgeKind::Critical))
    } else if freq > 0.1 {
        ("MEDIUM", components::status_badge(components::BadgeKind::Warn))
    } else {
        ("LOW", components::status_badge(components::BadgeKind::Ok))
    };

    components::kv(
        "Risk",
        &std::format!("{} {}{}{}{}", badge, color::BOLD, rate_color, risk, color::RESET),
    );

    components::footer();
}

fn cmd_doctor(config: &PulseConfig) {
    let db_path = config.db_path();
    let db_size = Database::db_size_bytes(&db_path);
    let daemon_pid = is_daemon_running();

    let (sys_m, proc_m, anom) = if db_path.exists() {
        Database::open_readonly(&db_path)
            .and_then(|db| queries::query_metric_counts(db.conn()))
            .unwrap_or((0, 0, 0))
    } else {
        (0, 0, 0)
    };

    let tracked = if db_path.exists() {
        Database::open_readonly(&db_path)
            .and_then(|db| queries::query_process_names(db.conn()))
            .map(|v| v.len())
            .unwrap_or(0)
    } else {
        0
    };

    components::doctor_report(
        &PulseConfig::config_path().display().to_string(),
        &db_path.display().to_string(),
        db_size,
        daemon_pid,
        sys_m,
        proc_m,
        anom,
        tracked,
    );

    components::footer();
}

fn cmd_config(config: &PulseConfig) {
    components::section("Active Configuration");
    components::kv("Interval", &std::format!("{}s", config.interval));
    components::kv("History", &std::format!("{} days", config.history_days));
    components::kv("CPU Threshold", &std::format!("{:.0}%", config.cpu_threshold));
    components::kv("Mem Threshold", &std::format!("{:.0}%", config.memory_threshold));
    components::kv("Pred. Window", &std::format!("{} samples", config.prediction_window));
    components::kv("Notifications", &std::format!("{}", config.enable_notifications));
    components::kv("Cooldown", &std::format!("{}s", config.notification_cooldown));
    components::kv("Log Level", &config.log_level);
    components::kv("Top-N", &std::format!("{}", config.top_n));
    components::kv("DB Path", &config.db_path().display().to_string());
    components::kv("Config File", &PulseConfig::config_path().display().to_string());
}

fn cmd_daemon(action: DaemonAction) {
    match action {
        DaemonAction::Start => daemon_start(),
        DaemonAction::Stop => daemon_stop(),
        DaemonAction::Status => daemon_status(),
    }
}

fn cmd_version() {
    println!(
        "  {}{}pulse{} v{}",
        color::BOLD, color::CYAN, color::RESET,
        env!("CARGO_PKG_VERSION"),
    );
}

// ── Daemon helpers ───────────────────────────────────────────────────

fn is_daemon_running() -> Option<u32> {
    let pid_path = PulseConfig::pid_path();
    let content = std::fs::read_to_string(&pid_path).ok()?;
    let pid = content.trim().parse::<u32>().ok()?;

    // Verify the process actually exists via sysinfo
    let sys = sysinfo::System::new_all();
    let sysinfo_pid = sysinfo::Pid::from(pid as usize);
    if sys.process(sysinfo_pid).is_some() {
        Some(pid)
    } else {
        // Stale PID file — clean it up
        std::fs::remove_file(&pid_path).ok();
        None
    }
}

fn daemon_start() {
    if let Some(pid) = is_daemon_running() {
        println!(
            "  {} Daemon already running (PID {})",
            components::status_badge(components::BadgeKind::Ok),
            pid,
        );
        return;
    }

    let exe = std::env::current_exe().unwrap_or_default();
    let daemon_exe = exe.with_file_name("pulse-daemon");

    if !daemon_exe.exists() {
        println!(
            "  {}{}pulse-daemon binary not found. Run `cargo build` first.{}",
            color::RED, color::BOLD, color::RESET,
        );
        return;
    }

    match std::process::Command::new(&daemon_exe)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            println!(
                "  {} Daemon started (PID {})",
                components::status_badge(components::BadgeKind::Ok),
                child.id(),
            );
        }
        Err(e) => {
            println!(
                "  {}{}Failed to start daemon: {}{}",
                color::RED, color::BOLD, e, color::RESET,
            );
        }
    }
}

fn daemon_stop() {
    let pid_path = PulseConfig::pid_path();

    if let Some(pid) = is_daemon_running() {
        // Send SIGTERM on unix, taskkill on Windows
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status();
        }

        std::fs::remove_file(&pid_path).ok();
        println!(
            "  {} Daemon stopped (PID {})",
            components::status_badge(components::BadgeKind::Info),
            pid,
        );
    } else {
        println!(
            "  {} Daemon is not running",
            components::status_badge(components::BadgeKind::Warn),
        );
    }
}

fn daemon_status() {
    if let Some(pid) = is_daemon_running() {
        println!(
            "  {} Daemon running (PID {})",
            components::status_badge(components::BadgeKind::Ok),
            pid,
        );
    } else {
        println!(
            "  {} Daemon is not running",
            components::status_badge(components::BadgeKind::Warn),
        );
    }
}
