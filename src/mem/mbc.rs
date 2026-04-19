use crate::cart::Cart;
use crate::mem;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteArray;

type Sram = Box<ByteArray<{ 8 * 1024 }>>;

#[derive(Serialize, Deserialize, Debug)]
pub enum Mbc {
    None {
        sram: Sram,
    },
    One {
        rom_bank_reg: u8,
        rom_bank_reg_mask: u8,
        sram_enabled: bool,
        extended_bank: Mbc1ExtBank,
    },
    Two {
        rom_bank_reg: u8,
        sram_4bit: Box<ByteArray<512>>,
        sram_enabled: bool,
    },
    Three {
        rom_bank_reg: u8,
        sram_bank_or_rtc_reg: u8,
        sram_and_rtc_enabled: bool,
        sram: [Sram; 8],
        latching: bool,
        rtc: [u8; 5],
    },
    Five {
        rom_bank_reg: u16,
        sram_enabled: bool,
        sram_bank_reg: u8,
        sram: [Sram; 16],
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Mbc1ExtBank {
    Ram {
        advanced: bool,
        sram_bank_reg: u8,
        sram: [Sram; 4],
    },
    Rom {
        advanced: bool,
        rom_bank_upper_reg: u8,
        sram: Sram,
    },
}

impl Mbc {
    pub fn from_cart(cart: &Cart) -> Self {
        for feature in cart.features() {
            match feature {
                crate::cart::Feature::Mbc1 => {
                    let bank_count: u8 = cart
                        .data()
                        .len()
                        .div_ceil(16 * 1024)
                        .try_into()
                        .expect("cart isn't too large");
                    return Self::One {
                        rom_bank_reg: 0,
                        rom_bank_reg_mask: bank_count.next_power_of_two() - 1,
                        sram_enabled: false,
                        extended_bank: if bank_count > 32 {
                            Mbc1ExtBank::Rom {
                                advanced: false,
                                rom_bank_upper_reg: 0,
                                sram: Default::default(),
                            }
                        } else {
                            Mbc1ExtBank::Ram {
                                advanced: false,
                                sram_bank_reg: 0,
                                sram: Default::default(),
                            }
                        },
                    };
                }
                crate::cart::Feature::Mbc2 => {
                    return Self::Two {
                        rom_bank_reg: 0,
                        sram_4bit: Default::default(),
                        sram_enabled: false,
                    };
                }
                crate::cart::Feature::Mbc3 => {
                    return Self::Three {
                        rom_bank_reg: 0,
                        sram_bank_or_rtc_reg: 0,
                        sram_and_rtc_enabled: false,
                        sram: Default::default(),
                        latching: false,
                        rtc: [0; _],
                    };
                }
                crate::cart::Feature::Mbc5 => {
                    return Self::Five {
                        rom_bank_reg: 1,
                        sram_enabled: false,
                        sram_bank_reg: 0,
                        sram: Default::default(),
                    };
                }
                crate::cart::Feature::Mbc6 | crate::cart::Feature::Mbc7 => {
                    unimplemented!("very rare cart")
                }
                _ => {}
            }
        }
        Self::None {
            sram: Default::default(),
        }
    }

    pub fn bank_and_cart_addr(&self, addr: u16) -> Option<(u16, usize)> {
        match addr {
            mem::ROM_BANK_0_START..mem::ROM_BANK_N_START => {
                if let Self::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank:
                        Mbc1ExtBank::Rom {
                            advanced: true,
                            rom_bank_upper_reg,
                            ..
                        },
                    ..
                } = self
                {
                    // MBC1 advanced mode on 1MB+ cart, bank # comes from upper/lower regs
                    let bank_lower = rom_bank_reg & rom_bank_reg_mask;
                    let bank = (rom_bank_upper_reg << 5) + bank_lower;
                    let addr = ((bank as usize) << 14) + addr as usize;
                    Some((bank.into(), addr))
                } else {
                    // otherwise, simply read the first ROM bank
                    Some((0, addr.into()))
                }
            }

            mem::ROM_BANK_N_START..mem::VRAM_START => match self {
                Self::None { .. } => Some((0, addr.into())),
                Self::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank: Mbc1ExtBank::Ram { .. },
                    ..
                } => {
                    let bank =
                        if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg } & rom_bank_reg_mask;
                    let addr = ((bank as usize) << 14) + (addr - mem::ROM_BANK_N_START) as usize;
                    Some((bank.into(), addr))
                }
                Self::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank:
                        Mbc1ExtBank::Rom {
                            rom_bank_upper_reg, ..
                        },
                    ..
                } => {
                    // bank == 0 check must come *before* mask check
                    let bank_lower =
                        if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg } & rom_bank_reg_mask;
                    let bank = (rom_bank_upper_reg << 5) + bank_lower;
                    let addr = ((bank as usize) << 14) + (addr - mem::ROM_BANK_N_START) as usize;
                    Some((bank.into(), addr))
                }
                Self::Two { rom_bank_reg, .. } | Self::Three { rom_bank_reg, .. } => {
                    let bank = if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg };
                    let addr = ((bank as usize) << 14) + (addr - mem::ROM_BANK_N_START) as usize;
                    Some((bank.into(), addr))
                }
                Self::Five { rom_bank_reg, .. } => {
                    // no bank == 0 check here
                    let addr =
                        ((*rom_bank_reg as usize) << 14) + (addr - mem::ROM_BANK_N_START) as usize;
                    Some(((addr >> 14) as _, addr))
                }
            },

            _ => None,
        }
    }
}
