use clap::{Parser, Subcommand};

mod core;
use core::metrics;

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
            println!("{:#?}", snapshot);
        }
        Commands::Inspect { process } => {
            let processes = metrics::inspect_process(&process);

            if processes.is_empty(){
                println!("No process named '{}' found", process);
            }else{
                for p in processes{
                    println!("{:#?}", p);
                }
            }
        }
    }
}