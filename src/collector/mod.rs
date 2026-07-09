//! System and process metric collection using `sysinfo`.
//!
//! The [`SystemCollector`] struct keeps a reusable `sysinfo::System`
//! instance to avoid per-call reallocation.

use sysinfo::System;
use std::{thread, time::Duration};

/// A point-in-time snapshot of system-wide metrics.
#[derive(Debug)]
pub struct SystemSnapshot {
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpu_usage: f32,
    pub swap_total: u64,
    pub swap_used: u64,
}

/// A point-in-time snapshot of a single process.
#[derive(Debug)]
pub struct ProcessSnapshot {
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub cpu_usage: f32,
}

/// Reusable metric collector that keeps its `System` instance alive
/// across refreshes for better performance and accurate CPU deltas.
pub struct SystemCollector {
    system: System,
}

impl SystemCollector {
    /// Create a new collector, performing the initial double-refresh
    /// required for accurate CPU readings.
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        thread::sleep(Duration::from_millis(250));
        system.refresh_all();
        SystemCollector { system }
    }

    /// Refresh all metrics (call between collection cycles).
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Capture a system-wide snapshot.
    pub fn system_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            cpu_usage: self.system.global_cpu_info().cpu_usage(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        }
    }

    /// Return the top `n` processes sorted by memory usage (descending).
    pub fn top_processes(&self, n: usize) -> Vec<ProcessSnapshot> {
        let mut procs: Vec<ProcessSnapshot> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| ProcessSnapshot {
                pid: pid.as_u32() as i32,
                name: p.name().to_string(),
                memory: p.memory(),
                cpu_usage: p.cpu_usage(),
            })
            .collect();

        procs.sort_by(|a, b| b.memory.cmp(&a.memory));
        procs.truncate(n);
        procs
    }

    /// Find all process instances matching `name` (case-insensitive).
    pub fn find_process(&self, name: &str) -> Vec<ProcessSnapshot> {
        self.system
            .processes()
            .iter()
            .filter(|(_, p)| p.name().eq_ignore_ascii_case(name))
            .map(|(pid, p)| ProcessSnapshot {
                pid: pid.as_u32() as i32,
                name: p.name().to_string(),
                memory: p.memory(),
                cpu_usage: p.cpu_usage(),
            })
            .collect()
    }

    /// Find a specific process by its PID.
    pub fn find_process_by_pid(&self, target_pid: i32) -> Option<ProcessSnapshot> {
        self.system
            .processes()
            .iter()
            .find(|(pid, _)| pid.as_u32() as i32 == target_pid)
            .map(|(pid, p)| ProcessSnapshot {
                pid: pid.as_u32() as i32,
                name: p.name().to_string(),
                memory: p.memory(),
                cpu_usage: p.cpu_usage(),
            })
    }
}
