use clap::{Parser, Subcommand};

mod core;
use core::{metrics, format};

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

            if processes.is_empty(){
                println!("No process named '{}' found", process);
            }else{
                for p in processes{
                    println!("PID {}", p.pid);
                    println!("  Name: {}", p.name);
                    println!("  Memory: {}", format::format_memory_kb(p.memory));
                    println!("  CPU: {}", format::format_cpu(p.cpu_usage));
                    println!();
                }
            }
        }
    }
}