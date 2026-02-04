use clap::{Parser, Subcommand};
use std::{fmt::Display, path::PathBuf};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print ROM information
    RomInfo {
        /// Path to ROM file
        path: PathBuf,
    },
}

trait ResultExt<T> {
    fn or_terminate(self) -> T;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn or_terminate(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::RomInfo { path } => {
            let rom = yokoi::Rom::read(&path).or_terminate();
        }
    }
}
