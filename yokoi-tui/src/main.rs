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

enum Error {
    Io(std::io::Error),
    Cart(yokoi::cart::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(Error::Io(err)) => eprintln!("Error: {err}"),
        Err(Error::Cart(yokoi::cart::Error(err))) => eprintln!("Error while parsing cart: {err}"),
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();

    match cli.command {
        Commands::CartInfo { path } => {
            let data = std::fs::read(&path)?;
            let cart = yokoi::cart::Cart::new(data).map_err(Error::Cart)?;

            writeln!(out, "Title: {}", cart.title())?;

            let len = cart.data().len();
            let field = if len >= 1_000_000 {
                format_args!("{} ({:.2} MB)", len, len as f32 / 1_000_000.0)
            } else if len >= 1_000 {
                format_args!("{} ({:.2} KB)", len, len as f32 / 1_000.0)
            } else {
                format_args!("{len}")
            };
            writeln!(out, "Bytes: {}", field)?;

            writeln!(
                out,
                "Color Support: {}",
                match cart.color_supported() {
                    yokoi::cart::ColorSupport::BackwardsCompatible => "Backwards Compatible",
                    yokoi::cart::ColorSupport::Exclusive => "Exclusive",
                    _ => "No",
                }
            )?;

            writeln!(out, "Licensee: {}", cart.licensee())?;

            write!(out, "Features: ")?;
            let features = cart.features();
            let mut first = true;
            for feature in features {
                if first {
                    first = false;
                } else {
                    write!(out, ", ")?;
                }
                write!(
                    out,
                    "{}",
                    match feature {
                        yokoi::cart::Feature::Mbc1 => "MBC1",
                        yokoi::cart::Feature::Mbc2 => "MBC2",
                        yokoi::cart::Feature::Mbc3 => "MBC3",
                        yokoi::cart::Feature::Mbc5 => "MBC5",
                        yokoi::cart::Feature::Mbc6 => "MBC6",
                        yokoi::cart::Feature::Mbc7 => "MBC7",
                        yokoi::cart::Feature::Mmm01 => "MMM01",
                        yokoi::cart::Feature::Ram => "RAM",
                        yokoi::cart::Feature::Battery => "Battery",
                        yokoi::cart::Feature::Timer => "Timer",
                        yokoi::cart::Feature::Rumble => "Rumble",
                        yokoi::cart::Feature::Sensor => "Sensor",
                        yokoi::cart::Feature::Camera => "Camera",
                        yokoi::cart::Feature::Tamagotchi => "Tamagotchi",
                        yokoi::cart::Feature::HuC1 => "HuC1",
                        yokoi::cart::Feature::HuC3 => "HuC3",
                    }
                )?;
            }
            if features.is_empty() {
                write!(out, "ROM only")?;
            }
            writeln!(out)?;

            write!(out, "ROM Size: ")?;
            let size = cart.rom_size();
            if size >= 1024 * 1024 {
                writeln!(out, "{} MiB", size / 1024 / 1024)?;
            } else {
                writeln!(out, "{} KiB", size / 1024)?;
            }

            write!(out, "RAM Size: ")?;
            let size = cart.ram_size();
            if size >= 1024 {
                writeln!(out, "{} KiB", size / 1024)?;
            } else {
                writeln!(out, "0 B")?;
            }
        }

        Commands::CartDump { bytes, path } => {
            let data = std::fs::read(&path)?;
            let cart = yokoi::cart::Cart::new(data).map_err(Error::Cart)?;
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
