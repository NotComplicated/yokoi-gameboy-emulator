mod debugger;
mod logger;
mod tui;

use clap::{Parser, Subcommand};
use log::{LevelFilter, error};
use logger::Logger;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
};
use yokoi::{
    cart::{Cart, ColorSupport, Feature},
    frame::Theme,
    system::{Mode, Options, System},
};

/// Interface with the Yokoi emulator backend from the terminal.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a cartridge in the emulator
    Run {
        /// Launch the emulator in a new ghostty window
        #[arg(long)]
        new_window: bool,

        /// Use the classic green color scheme instead of grayscale
        #[arg(long)]
        classic_theme: bool,

        /// Skip the boot-up sequence
        #[arg(long)]
        skip_boot: bool,

        /// Don't show terminal UI. For use within a debugger
        #[arg(long)]
        debug: bool,

        /// Log level when debugging. Overriden by RUST_LOG
        #[arg(long, requires = "debug", default_value_t = LevelFilter::Info)]
        log_level: LevelFilter,

        /// Send logs to this socket address
        #[arg(long)]
        log_socket: Option<SocketAddr>,

        /// Short-circuit the emulator after N t-cycles
        #[arg(long)]
        short_circuit: Option<u64>,

        /// Path to debug symbols used for debugging
        #[arg(long, requires = "debug")]
        symbols: Option<PathBuf>,

        /// Set a breakpoint on a debug symbol. Can be provided multiple times
        #[arg(short = 'B', long = "breakpoint", requires = "symbols")]
        breakpoints: Vec<String>,

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

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    System(yokoi::system::Error),
    Cart(yokoi::cart::Error),
    Image(viuer::ViuError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => writeln!(f, "Error: {err}"),
            Self::System(yokoi::system::Error::ShortCircuit) => writeln!(f, "- Short-circuited -"),
            Self::System(yokoi::system::Error::Symbol(
                yokoi::system::SymbolError::BreakpointNotFound(breakpoint),
            )) => writeln!(f, "'{breakpoint}' not found in symbols"),
            Self::System(yokoi::system::Error::Breakpoint(breakpoint)) => {
                writeln!(f, "Reached breakpoint: {breakpoint}")
            }
            Self::Image(err) => writeln!(f, "Error while rendering image: {err}"),
            Self::System(err) => writeln!(f, "Internal system error: {err:?}"),
            Self::Cart(yokoi::cart::Error(err)) => writeln!(f, "Error while parsing cart: {err}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
    if crossterm::terminal::is_raw_mode_enabled().unwrap() {
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();

    match cli.command {
        Commands::Run {
            new_window: true, ..
        } => {
            let server = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
            let addr = server.local_addr()?.to_string();
            let mut runner =
                Command::new("ghostty")
                    .args(
                        [
                            "--font-size=5",
                            "--window-width=320",
                            "--window-height=144",
                            "-e",
                        ]
                        .into_iter()
                        .map(Into::into)
                        .chain(std::env::args().filter(|arg| {
                            !["--new-window", "--log-socket"].contains(&arg.as_str())
                        }))
                        .chain(["--log-socket".into(), addr]),
                    )
                    .stderr(Stdio::null())
                    .spawn()?;
            let (client, _) = server.accept()?;
            let log_messages = BufReader::new(client);
            for line in log_messages.lines() {
                writeln!(out, "{}", line?)?;
            }
            runner.wait()?;
        }

        Commands::Run {
            classic_theme,
            skip_boot,
            debug,
            log_level,
            log_socket,
            short_circuit,
            symbols,
            breakpoints,
            boot,
            cart,
            ..
        } => {
            let stream = log_socket
                .map(|addr| TcpStream::connect(addr))
                .transpose()?;
            if debug {
                if let Some(stream) = stream {
                    Logger(stream).init();
                } else {
                    drop(out);
                    Logger(io::stdout()).init();
                }
                log::set_max_level(
                    std::env::var("RUST_LOG")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(log_level),
                );
            }

            let boot_rom_data = std::fs::read(&boot)?;
            let cart_data = std::fs::read(&cart)?;
            let cart = Cart::new(cart_data).map_err(Error::Cart)?;
            let system = System::init_options(
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
                    symbols: symbols
                        .map(File::open)
                        .transpose()
                        .map_err(Error::Io)?
                        .map(|f| Box::new(f) as _),
                    breakpoints,
                },
            )
            .map_err(Error::System)?;

            // if this a lone debugging session (not connected to a server), don't create a TUI
            if debug && log_socket.is_none() {
                debugger::run(system)?;
            } else {
                let term = ratatui::try_init()?;
                if let Err(err) = tui::run(term, system) {
                    error!("{err}");
                }
                ratatui::restore();
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
