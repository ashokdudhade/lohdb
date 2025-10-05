use anyhow::Result;
use clap::Parser;
use lohdb::{run_cli, Database, DatabaseConfig};

#[derive(Parser)]
#[command(name = "lohdb")]
#[command(about = "A simple embedded key-value database")]
struct Cli {
    #[arg(short, long, default_value = "./lohdb_data")]
    data_dir: String,
    
    #[arg(short, long)]
    interactive: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let config = DatabaseConfig {
        data_dir: cli.data_dir,
        wal_sync_interval_ms: 1000,
    };
    
    let db = Database::open(config)?;
    
    if cli.interactive {
        run_cli(db)?;
    } else {
        println!("LohDB started. Use --interactive for CLI mode.");
        // In a real application, you might start a server here
    }
    
    Ok(())
}