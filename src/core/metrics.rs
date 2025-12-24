use sysinfo::{System};
use std::{thread, time::Duration};

use crate::core::history;

const MAX_ENTRIES_PER_PROCESS: usize = 1000;

#[derive(Debug)]
pub struct SystemSnapshot{
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpu_usage: f32,
}

#[derive(Debug)]
pub struct ProcessSnapshot{
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub cpu_usage: f32,
}

pub fn system_snapshot() -> SystemSnapshot{
    let mut system = System::new_all();
    system.refresh_all();

    SystemSnapshot{
        total_memory: system.total_memory(),
        used_memory: system.used_memory(),
        cpu_usage: system.global_cpu_info().cpu_usage(),
    }
}

pub fn inspect_process(name: &str) -> Vec<ProcessSnapshot> {
    let mut system = System::new_all();
    system.refresh_processes();
    thread::sleep(Duration::from_millis(500));
    system.refresh_processes();
    
    // system.refresh_all();

    let mut history_store = history::load_history();
    let mut results = Vec::new();

    for (pid, process) in system.processes(){
        if process.name().eq_ignore_ascii_case(name){
            let snapshot = ProcessSnapshot{
                pid: pid.as_u32() as i32,
                name: process.name().to_string(),
                memory: process.memory(),
                cpu_usage: process.cpu_usage(),
            };

            let entries = history_store
                .processes
                .entry(snapshot.name.clone())
                .or_default();

            entries.push(history::ProcessHistoryEntry{
                memory: snapshot.memory,
                cpu_usage: snapshot.cpu_usage,
                timestamp: history::now_ts(),
            });

            if entries.len() > MAX_ENTRIES_PER_PROCESS{
                let excess = entries.len() - MAX_ENTRIES_PER_PROCESS;
                entries.drain(0..excess);
            }
            
            results.push(snapshot);
        }
    }

    history::save_history(&history_store);
    results
}