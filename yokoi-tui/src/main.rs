mod tui;

use clap::{Parser, Subcommand};
use std::{
    io::{self, Write},
    path::PathBuf,
};
use tracing::debug;
use tracing_subscriber::EnvFilter;
use yokoi::{
    cart::{Cart, ColorSupport, Feature},
    frame::Theme,
    system::{Input, Mode, Options, System},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a cartridge in the emulator
    Run {
        /// Don't show terminal UI. For use within a debugger
        #[arg(long)]
        debug: bool,

        /// Skip the boot-up sequence
        #[arg(long)]
        skip_boot: bool,

        /// Use the classic green color scheme instead of grayscale
        #[arg(long)]
        classic_theme: bool,

        /// Short-circuit the emulator after N t-cycles
        #[arg(long)]
        short_circuit: Option<u64>,

        /// Path to boot ROM file
        #[arg(short, long)]
        boot: PathBuf,

        /// Path to cartridge file
        cart: PathBuf,
    },

    /// Print cartridge information
    CartInfo {
        /// Path to cartridge file
        cart: PathBuf,
    },

    /// Hex-dump cartridge contents
    CartDump {
        /// Only print the first N bytes
        #[arg(short = 'c', long)]
        bytes: Option<usize>,

        /// Path to cartridge file
        cart: PathBuf,
    },
}

pub enum Error {
    Io(std::io::Error),
    System(yokoi::system::Error),
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
        Err(Error::System(yokoi::system::Error::ShortCircuit)) => eprintln!("- Short-circuited -"),
        Err(Error::System(err)) => eprintln!("Internal system error: {err:?}"),
        Err(Error::Cart(yokoi::cart::Error(err))) => eprintln!("Error while parsing cart: {err}"),
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();

    match cli.command {
        Commands::Run {
            boot,
            debug,
            skip_boot,
            classic_theme,
            short_circuit,
            cart,
        } => {
            let boot_rom_data = std::fs::read(&boot)?;
            let cart_data = std::fs::read(&cart)?;
            let cart = Cart::new(cart_data).map_err(Error::Cart)?;
            let mut system = System::init_options(
                boot_rom_data,
                cart,
                Mode::Dmg,
                Options {
                    theme: if classic_theme {
                        Theme::Classic
                    } else {
                        Theme::Grayscale
                    },
                    short_circuit,
                    debug,
                    skip_boot,
                },
            )
            .map_err(Error::System)?;
            if debug {
                tracing_subscriber::fmt()
                    .with_env_filter(EnvFilter::from_default_env())
                    .without_time()
                    .with_level(false)
                    .with_target(false)
                    .fmt_fields(tracing_subscriber::fmt::format::debug_fn(
                        |writer, field, value| writeln!(writer, "{}: {value:?}", field.name()),
                    ))
                    .with_writer(|| {
                        struct Writer(io::StdoutLock<'static>);
                        impl io::Write for Writer {
                            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                                self.0.write(buf).map_err(|_| std::process::exit(0))
                            }
                            fn flush(&mut self) -> io::Result<()> {
                                self.0.flush()
                            }
                        }
                        Writer(io::stdout().lock())
                    })
                    .init();
                for i in 0.. {
                    debug!(frame = i);
                    let input = Input::<Vec<u8>>::default();
                    system.next_frame(input).map_err(Error::System)?;
                }
            } else {
                let term = ratatui::try_init()?;
                let run_result = tui::run(term, system);
                ratatui::restore();
                run_result?;
            }
        }

        Commands::CartInfo { cart } => {
            let data = std::fs::read(&cart)?;
            let cart = Cart::new(data).map_err(Error::Cart)?;

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
                    ColorSupport::BackwardsCompatible => "Backwards Compatible",
                    ColorSupport::Exclusive => "Exclusive",
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
                        Feature::Mbc1 => "MBC1",
                        Feature::Mbc2 => "MBC2",
                        Feature::Mbc3 => "MBC3",
                        Feature::Mbc5 => "MBC5",
                        Feature::Mbc6 => "MBC6",
                        Feature::Mbc7 => "MBC7",
                        Feature::Mmm01 => "MMM01",
                        Feature::Ram => "RAM",
                        Feature::Battery => "Battery",
                        Feature::Timer => "Timer",
                        Feature::Rumble => "Rumble",
                        Feature::Sensor => "Sensor",
                        Feature::Camera => "Camera",
                        Feature::Tamagotchi => "Tamagotchi",
                        Feature::HuC1 => "HuC1",
                        Feature::HuC3 => "HuC3",
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

        Commands::CartDump { bytes, cart } => {
            let data = std::fs::read(&cart)?;
            let cart = Cart::new(data).map_err(Error::Cart)?;
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
