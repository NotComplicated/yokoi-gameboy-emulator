use clap::{Parser, Subcommand};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print ROM information
    RomInfo {
        /// Hex-dump the ROM contents
        #[arg(long)]
        dump: bool,

        /// Path to ROM file
        path: PathBuf,
    },
}

trait ResultExt<T> {
    fn or_exit(self) -> T;
}

impl<T> ResultExt<T> for Result<T, std::io::Error> {
    fn or_exit(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {
                std::process::exit(141);
            }
            Err(err) => {
                eprintln!("IO Error: {err}");
                std::process::exit(1);
            }
        }
    }
}

impl<T> ResultExt<T> for Result<T, yokoi::Error> {
    fn or_exit(self) -> T {
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
    let mut out = std::io::stdout().lock();
    match cli.command {
        Commands::RomInfo { dump, path } => {
            let rom = yokoi::Rom::read(&path).or_exit();
            if dump {
                let width = crossterm::terminal::size().or_exit().0;
                let chunk_size = ((width as usize - "0000:  ".len()) / 3).next_power_of_two() / 2;
                for (i, chunk) in rom.data().chunks(chunk_size).enumerate() {
                    write!(&mut out, "{:04X}:  ", i * chunk_size).or_exit();
                    for &byte in chunk {
                        write!(&mut out, "{byte:02X} ").or_exit();
                    }
                    writeln!(&mut out).or_exit();
                }
            }
        }
    }
}
