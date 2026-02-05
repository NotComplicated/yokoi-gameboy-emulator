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
        /// Hex-dump the cartridge file contents
        #[arg(long)]
        dump: bool,

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
        Commands::CartInfo { dump, path } => {
            let cart = yokoi::cart::Cart::read(&path)?;
            if dump {
                let width = crossterm::terminal::size()?.0;
                let chunk_size = ((width as usize - "000000:  ".len()) / 3).next_power_of_two() / 2;
                for (i, chunk) in cart.data().chunks(chunk_size).enumerate() {
                    write!(&mut out, "{:06X}:  ", i * chunk_size)?;
                    for byte in chunk {
                        write!(&mut out, "{byte:02X} ")?;
                    }
                    writeln!(&mut out)?;
                }
            }
            println!("{}", cart.data().len());
        }
    }
    Ok(())
}
