//! Background daemon — collection loop with graceful shutdown,
//! anomaly detection, prediction, and alert dispatching.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};
use log;

use crate::config::PulseConfig;
use crate::storage::Database;
use crate::storage::queries;
use crate::collector::SystemCollector;
use crate::analytics::{anomaly, trend, prediction, explain};
use crate::alerts::AlertManager;

/// Manages the daemon lifecycle: collection, analysis, and shutdown.
pub struct DaemonRunner {
    config: PulseConfig,
    db: Database,
    collector: SystemCollector,
    alerts: AlertManager,
    running: Arc<AtomicBool>,
    cycles: u64,
}

impl DaemonRunner {
    /// Create a new runner and register the Ctrl-C handler.
    pub fn new(config: PulseConfig, db: Database) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            log::info!("Shutdown signal received");
            r.store(false, Ordering::SeqCst);
        })
        .expect("Failed to set Ctrl+C handler");

        let collector = SystemCollector::new();
        let alerts = AlertManager::new(
            config.enable_notifications,
            config.notification_cooldown,
        );

        DaemonRunner {
            config,
            db,
            collector,
            alerts,
            running,
            cycles: 0,
        }
    }

    /// Run the main collection loop until a shutdown signal is received.
    pub fn run(&mut self) {
        log::info!("Pulse daemon started");
        log::info!(
            "Interval: {}s | Top-N: {} | DB: {}",
            self.config.interval,
            self.config.top_n,
            self.config.db_path().display(),
        );

        // Write PID file
        let pid = std::process::id();
        let pid_path = PulseConfig::pid_path();
        std::fs::write(&pid_path, pid.to_string()).ok();

        println!(
            "Pulse daemon started (PID {})\n\
             Interval: {}s | Tracking top {} processes\n\
             Press Ctrl+C to stop.",
            pid, self.config.interval, self.config.top_n,
        );

        while self.running.load(Ordering::SeqCst) {
            self.collect_cycle();
            self.cycles += 1;

            // Periodic cleanup (roughly every hour)
            let cycles_per_hour = 3600 / self.config.interval.max(1);
            if cycles_per_hour > 0 && self.cycles % cycles_per_hour == 0 {
                if let Err(e) = self.db.cleanup(self.config.history_days) {
                    log::warn!("Cleanup failed: {}", e);
                }
            }

            // Interruptible sleep
            let sleep_ms = self.config.interval * 1000;
            let mut elapsed = 0u64;
            while elapsed < sleep_ms && self.running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(250));
                elapsed += 250;
            }
        }

        // Cleanup on exit
        log::info!("Daemon shutting down …");
        std::fs::remove_file(&pid_path).ok();
        log::info!(
            "Daemon stopped (ran {} collection cycles)",
            self.cycles,
        );
        println!("Daemon stopped.");
    }

    /// Execute one collection + analysis cycle.
    fn collect_cycle(&mut self) {
        self.collector.refresh();
        let now = chrono::Utc::now().timestamp();

        // ── System metrics ───────────────────────────────────────────
        let sys = self.collector.system_snapshot();
        if let Err(e) = queries::insert_system_metric(
            self.db.conn(),
            now,
            sys.cpu_usage as f64,
            sys.used_memory,
            sys.swap_used,
        ) {
            log::error!("Failed to insert system metric: {}", e);
        }

        // System-level threshold alerts
        let mem_pct =
            (sys.used_memory as f64 / sys.total_memory.max(1) as f64) * 100.0;
        if mem_pct > self.config.memory_threshold {
            self.alerts.try_send(
                "memory_critical",
                "⚡ Pulse: High Memory",
                &format!("System memory at {:.0}%", mem_pct),
            );
        }
        if (sys.cpu_usage as f64) > self.config.cpu_threshold {
            self.alerts.try_send(
                "cpu_critical",
                "⚡ Pulse: High CPU",
                &format!("System CPU at {:.1}%", sys.cpu_usage),
            );
        }

        // ── Process metrics ──────────────────────────────────────────
        let processes = self.collector.top_processes(self.config.top_n);

        let batch: Vec<(i64, i32, &str, f64, u64)> = processes
            .iter()
            .map(|p| {
                (now, p.pid, p.name.as_str(), p.cpu_usage as f64, p.memory)
            })
            .collect();

        if let Err(e) =
            queries::insert_process_metrics_batch(self.db.conn(), &batch)
        {
            log::error!("Failed to insert process metrics: {}", e);
        }

        // ── Per-process anomaly detection + prediction ───────────────
        for p in &processes {
            let history = match queries::query_process_history(
                self.db.conn(),
                &p.name,
                self.config.prediction_window,
            ) {
                Ok(h) => h,
                Err(_) => continue,
            };

            // Anomaly detection
            if let Some(result) = anomaly::detect_anomaly(
                &history,
                p.memory,
                p.cpu_usage,
                2.0,
            ) {
                if result.memory_anomaly || result.cpu_anomaly {
                    let reasons = explain::explain_anomaly(&p.name, &result);
                    let reason = reasons.join("; ");

                    let _ = queries::insert_anomaly(
                        self.db.conn(),
                        now,
                        Some(p.pid),
                        &p.name,
                        result.severity.as_str(),
                        &reason,
                    );

                    self.alerts.try_send(
                        &format!("anomaly_{}", p.name),
                        "⚡ Pulse: Anomaly Detected",
                        &format!("{}: {}", p.name, reason),
                    );
                }
            }

            // Prediction
            if let Some(trend_result) = trend::detect_trend(
                &history,
                self.config.prediction_window,
            ) {
                if let Some(pred) = prediction::build_prediction(
                    &history,
                    p.memory,
                    trend_result.memory_slope,
                    trend_result.memory_trend,
                ) {
                    if let Some(eta) = pred.minutes_to_anomaly {
                        if eta < 60.0 && pred.confidence > 0.5 {
                            let _ = queries::insert_prediction(
                                self.db.conn(),
                                now,
                                "memory_leak",
                                pred.confidence,
                                Some(eta),
                            );

                            self.alerts.try_send(
                                &format!("prediction_{}", p.name),
                                "⚡ Pulse: Memory Leak Predicted",
                                &format!(
                                    "{}: anomaly in ~{:.0} min ({:.0}% confidence)",
                                    p.name,
                                    eta,
                                    pred.confidence * 100.0,
                                ),
                            );
                        }
                    }
                }
            }
        }

        log::debug!(
            "Cycle {} complete ({} processes)",
            self.cycles + 1,
            processes.len(),
        );
    }
}
