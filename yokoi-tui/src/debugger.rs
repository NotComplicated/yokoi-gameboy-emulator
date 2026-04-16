use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use log::info;
use yokoi::system::Input;

use crate::Error;

const HELP_TEXT: &str = "q - quit
r - show main memory registers
s - step over to the next instruction
t - show a stack trace
c - continue running the emulator";

pub fn run(mut system: yokoi::system::System) -> Result<(), Error> {
    loop {
        let input = Input::default();
        if let Err(err) = system.next_frame(input) {
            if let yokoi::system::Error::Breakpoint(breakpoint) = err {
                info!(breakpoint;"");
                info!("press 'h' for help, 'q' to quit");
                loop {
                    crossterm::terminal::enable_raw_mode()?;
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Char(key),
                        kind: KeyEventKind::Press,
                        ..
                    }) = crossterm::event::read()?
                    {
                        crossterm::terminal::disable_raw_mode()?;
                        match key {
                            'h' => info!("{HELP_TEXT}"),
                            'q' => return Ok(()),
                            'r' => system.log_mem_registers(),
                            's' => match system.step() {
                                Ok(()) => {}
                                Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                                    info!(breakpoint;"")
                                }
                                Err(err) => return Err(Error::System(err)),
                            },
                            't' => {
                                for (i, frame) in system.stack_frames().iter().enumerate() {
                                    info!(
                                        frame = i,
                                        bank = frame.bank,
                                        address = format!("{:04X}", frame.addr),
                                        symbol = frame.latest_symbol
                                        ;""
                                    );
                                }
                            }
                            'c' => break,
                            _ => {}
                        }
                    }
                }
            } else {
                return Err(Error::System(err));
            }
        }
    }
}
