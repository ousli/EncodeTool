mod models;
mod ffmpeg;
mod processor;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use models::{ProcessConfig, Event};
use processor::{Action, process_batch, scan_files};

#[derive(Parser)]
#[command(name = "encodetool")]
#[command(about = "Batch video processor for macOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output JSON Lines for progress and logs
    #[arg(long, global = true)]
    jsonl: bool,

    /// Dry run: don't execute ffmpeg or renames
    #[arg(long, global = true)]
    dry_run: bool,

    /// Overwrite existing output files
    #[arg(long, global = true)]
    overwrite: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Rename files with YYYY-MM-DD_HHMM prefix
    Rename {
        #[arg(short, long)]
        source: PathBuf,
        #[arg(short, long)]
        export: Option<PathBuf>,
    },
    /// Reencode to H.265 10-bit VideoToolbox
    Reencode {
        #[arg(short, long)]
        source: PathBuf,
        #[arg(short, long)]
        export: Option<PathBuf>,
        #[arg(short, long, default_value_t = 65)]
        quality: u8,
    },
    /// Rename and Reencode to H.265 10-bit
    RenameReencode {
        #[arg(short, long)]
        source: PathBuf,
        #[arg(short, long)]
        export: Option<PathBuf>,
        #[arg(short, long, default_value_t = 65)]
        quality: u8,
    },
    /// Apply 3D LUT and encode to H.265 10-bit
    Lut {
        #[arg(short, long)]
        source: PathBuf,
        #[arg(short, long)]
        export: Option<PathBuf>,
        #[arg(short, long)]
        lut: PathBuf,
        #[arg(short, long, default_value_t = 60)]
        quality: u8,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (source, export, action) = match &cli.command {
        Commands::Rename { source, export } => {
            (source, export, Action::Rename)
        }
        Commands::Reencode { source, export, quality } => {
            (source, export, Action::Reencode { quality: *quality })
        }
        Commands::RenameReencode { source, export, quality } => {
            (source, export, Action::RenameReencode { quality: *quality })
        }
        Commands::Lut { source, export, lut, quality } => {
            (source, export, Action::Lut { path: lut.clone(), quality: *quality })
        }
    };

    let export_path = export.clone().unwrap_or_else(|| source.join("export"));

    let config = ProcessConfig {
        dry_run: cli.dry_run,
        jsonl: cli.jsonl,
        overwrite: cli.overwrite,
        source: source.clone(),
        export: export_path,
    };

    let files = scan_files(&config.source);
    
    if let Err(e) = process_batch(files, &config, &action) {
        if cli.jsonl {
            let err_ev = Event::Error {
                message: e.to_string(),
                code: "INTERNAL_ERROR".to_string(),
            };
            println!("{}", serde_json::to_string(&err_ev)?);
        } else {
            eprintln!("❌ Error: {}", e);
        }
        std::process::exit(1);
    }

    Ok(())
}
