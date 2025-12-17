use clap::{Parser, Subcommand};

mod core;
use core::{metrics, format, anomaly, history, explain, trend, ui};

#[derive(Parser)]
#[command(name = "pulse")]
#[command(about = "An intelligent system behaviour analyzer", long_about = None)]
struct Cli{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands{
    Status,
    Inspect{
        process: String,
    },
    Summary{
        process: String,
    }
}

fn main(){
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let snapshot = metrics::system_snapshot();
            println!("System Status");
            println!("─────────────");
            println!(
                "Memory: {} / {}",
                format::format_memory_kb(snapshot.used_memory),
                format::format_memory_kb(snapshot.total_memory)
            );
            println!("CPU Usage: {}", format::format_cpu(snapshot.cpu_usage));

        }
        Commands::Inspect { process } => {
            let processes = metrics::inspect_process(&process);

            let history_store = history::load_history();

            if processes.is_empty(){
                println!("No process named '{}' found", process);
            }else{
                for p in processes{
                    ui::section(&format!("Process: {} (PID {})", p.name, p.pid));

                    ui::kv("Memory", &format::format_memory_kb(p.memory));
                    ui::kv("CPU", &format::format_cpu(p.cpu_usage));
                    
                    if let Some(history) = history_store.processes.get(&p.name){
                        // General Trend
                        if let Some(trend_result) = trend::detect_trend(&history){
                            ui::section("Trend");
                            
                            match trend_result.memory_trend {
                                trend::TrendKind::Increasing => {
                                    println!("📈Memory trend: Increasing(Possible leak)");
                                }
                                trend::TrendKind::Decreasing => {
                                    println!("📉Memory trend: Decreasing");
                                }
                                trend::TrendKind::Stable => {
                                    println!("➖Memory trend: Stable");
                                }
                            }

                            match trend_result.cpu_trend {
                                trend::TrendKind::Increasing => {
                                    println!("📈CPU trend: Increasing");
                                }
                                trend::TrendKind::Decreasing => {
                                    println!("📉CPU trend: Decreasing");
                                }
                                trend::TrendKind::Stable => {
                                    println!("➖CPU trend: Stable");
                                }
                            }
                        }

                        // Anomaly detected
                        if let Some(result) = anomaly::detect_anomaly(
                            history, 
                            p.memory, 
                            p.cpu_usage,){
                            if result.memory_anomaly || result.cpu_anomaly{
                                ui::section("⚠️Anomaly");

                                if result.memory_anomaly{
                                    println!("Memory anomaly (z-score: {:.2})", result.memory_score);
                                }

                                if result.cpu_anomaly{
                                    println!("CPU anomaly (z-score: {:.2})", result.cpu_score);
                                }

                                let explanations = explain::explain_anomaly(&p.name, &result);
                                for line in explanations{
                                    println!("-> {}", line);
                                }
                            }else{
                                println!("Behavior: Normal");
                            }
                        }else {
                            println!("Behavior: Learning Baseline...");
                        }
                    }else {
                        println!("Behavior: No history yet!");
                    }
                    println!();
                }
            }
        }
        Commands::Summary { process } => {
            let history_store = history::load_history();

            let Some(entries) = history_store.processes.get(&process) else{
                println!("No history available for '{}'", process);
                return;
            };

            if entries.len() < 5{
                println!("Not enough data to summarize '{}'", process);
                return;
            }

            ui::section(&format!("Summary: {}", process));

            //average
            let avg_mem = entries.iter().map(|h| h.memory as f64).sum::<f64>() / entries.len() as f64;
            let avg_cpu = entries.iter().map(|h| h.cpu_usage as f64).sum::<f64>() / entries.len() as f64;

            ui::kv("Avg Memory", &format::format_memory_kb(avg_mem as u64));
            ui::kv("Avg CPU", &format::format_cpu(avg_cpu as f32));

            let anomalies = entries
                .windows(5)
                .filter(|window| {
                    if let Some(result) = anomaly::detect_anomaly(
                        &window, 
                        window.last().unwrap().memory, 
                        window.last().unwrap().cpu_usage){
                        result.memory_anomaly || result.cpu_anomaly
                    }else{
                        false
                    }
                })
                .count();

            let freq = anomalies as f64 / entries.len() as f64;

            ui::kv("Anomaly rate", &format!("{:.0}%", freq * 100.0));

            let risk = if freq > 0.3{
                "HIGH"
            } else if freq > 0.1{
                "MEDIUM"
            }else{
                "LOW"
            };

            ui::kv("Risk", risk);
        }
    }
}