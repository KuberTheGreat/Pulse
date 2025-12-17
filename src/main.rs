use clap::{Parser, Subcommand};

mod core;
use core::{metrics, format, anomaly, history, explain};

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
                    println!("PID {}", p.pid);
                    println!("  Name: {}", p.name);
                    println!("  Memory: {}", format::format_memory_kb(p.memory));
                    println!("  CPU: {}", format::format_cpu(p.cpu_usage));
                    
                    if let Some(history) = history_store.processes.get(&p.name){
                        if let Some(result) = anomaly::detect_anomaly(
                            history, 
                            p.memory, 
                            p.cpu_usage,){
                            if result.memory_anomaly || result.cpu_anomaly{
                                println!("  ⚠️  Anomaly detected");

                                if result.memory_anomaly{
                                    println!("      Memory anomaly (z-score: {:.2})", result.memory_score);
                                }

                                if result.cpu_anomaly{
                                    println!("      CPU anomaly (z-score: {:.2})", result.cpu_score);
                                }

                                let explanations = explain::explain_anomaly(&p.name, &result);
                                for line in explanations{
                                    println!("      -> {}", line);
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
    }
}