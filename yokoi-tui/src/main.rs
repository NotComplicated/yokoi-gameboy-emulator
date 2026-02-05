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

            writeln!(out, "Title: {}", cart.title())?;

            let len = cart.data().len();
            let field = if len > 1_000_000 {
                format_args!("{} ({:.2} MB)", len, len as f32 / 1_000_000.0)
            } else if len > 1_000 {
                format_args!("{} ({:.2} KB)", len, len as f32 / 1_000.0)
            } else {
                format_args!("{len}")
            };
            writeln!(out, "Bytes: {}", field)?;
        }

        Commands::CartDump { bytes, path } => {
            let cart = yokoi::cart::Cart::read(&path)?;
            let width = crossterm::terminal::size()?.0 as usize;
            let chunk_size = ((width - "000000:".len()) / 3).next_power_of_two() / 2;
            let data = if let Some(n) = bytes
                && n < cart.data().len()
            {
                &cart.data()[0..n]
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
