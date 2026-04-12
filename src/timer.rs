use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Timer {
    state: State,
    sys: [u8; 2],
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

#[derive(Default, Serialize, Deserialize, Debug)]
enum State {
    #[default]
    Ticking,
    Overflow(u8),
}

#[derive(Default, Debug)]
pub struct Result {
    pub interrupt: bool,
    pub div_apu: bool,
}

impl Timer {
    pub fn tick(&mut self) -> Result {
        let mut result = Result::default();
        let sys_prev = u16::from_be_bytes(self.sys);
        self.sys = sys_prev.wrapping_add(1).to_be_bytes();
        if sys_prev >> 12 % 2 == 1 && (sys_prev >> 12 + 1) % 2 == 0 {
            result.div_apu = true;
        }
        match &mut self.state {
            State::Ticking => {
                // check TAC.enable
                if (self.tac >> 2) % 2 == 1 {
                    let shift = match self.tac % 4 {
                        0 => 9,
                        1 => 3,
                        2 => 5,
                        3 => 7,
                        _ => unreachable!(),
                    };
                    let bit_prev = (sys_prev >> shift) % 2;
                    let bit_curr = (u16::from_be_bytes(self.sys) >> shift) % 2;
                    if bit_prev == 1 && bit_curr == 0 {
                        // falling edge on the bit selected by TAC.freq
                        self.tima = self.tima.wrapping_add(1);
                        if self.tima == 0 {
                            self.state = State::Overflow(4);
                        }
                    }
                }
            }
            State::Overflow(0) => {
                self.tima = self.tma;
                self.state = State::Ticking;
                result.interrupt = true;
            }
            State::Overflow(tick) => *tick -= 1,
        }
        result
    }

    pub fn read_div(&self) -> &[u8] {
        &self.sys[..1]
    }

    pub fn write_div(&mut self) {
        self.sys = [0, 0];
    }
}
