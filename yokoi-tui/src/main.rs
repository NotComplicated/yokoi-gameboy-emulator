use clap::{Parser, Subcommand};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print cartridge information
    CartInfo {
        /// Path to cartridge file
        path: PathBuf,
    },

    /// Hex-dump cartridge contents
    CartDump {
        /// Only print the first N bytes
        #[arg(short = 'c', long)]
        bytes: Option<usize>,

        /// Path to cartridge file
        path: PathBuf,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();
    match cli.command {
        Commands::CartInfo { path } => {
            let cart = yokoi::cart::Cart::read(&path)?;
            write!(out, "Bytes: {}", cart.data().len())?;
        }
        Commands::CartDump { bytes, path } => {
            let cart = yokoi::cart::Cart::read(&path)?;
            let width = crossterm::terminal::size()?.0 as usize;
            let chunk_size = ((width - "000000:".len()) / 3).next_power_of_two() / 2;
            let data = if let Some(n) = bytes {
                &cart.data()[0..n.min(cart.data().len())]
            } else {
                cart.data()
            };
            for (i, chunk) in data.chunks(chunk_size).enumerate() {
                write!(out, "{:06X}:", i * chunk_size)?;
                for byte in chunk {
                    write!(out, " {byte:02X}")?;
                }
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
