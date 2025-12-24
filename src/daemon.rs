use std::{thread, time::Duration};
use sysinfo::System;

mod core;
use core::metrics;

const SAMPLE_INTERVAL_SECS: u64 = 60;
const TOP_N: usize = 10;

fn main(){
    println!("Pulse daemon started (Foreground mode)");
    println!("Sampling top {} processes every {} seconds...", TOP_N, SAMPLE_INTERVAL_SECS);
    println!("Press Ctrl+C to stop.");

    loop{
        let mut system = System::new_all();
        system.refresh_processes();

        let mut processes: Vec<_> = system.processes().values().collect();

        processes.sort_by_key(|p| std::cmp::Reverse(p.memory()));

        for process in processes.iter().take(TOP_N){
            let name = process.name();

            let _ = metrics::inspect_process(name);
        }

        thread::sleep(Duration::from_secs(SAMPLE_INTERVAL_SECS));

    }
}