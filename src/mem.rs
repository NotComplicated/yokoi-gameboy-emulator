use crate::{
    cart::Cart,
    frame::Rgb555,
    opcode::{self, Op},
    system::{Joypad, Mode},
    timer::Timer,
    util::Hex,
};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteArray;

pub const ROM_BANK_0_START: u16 = 0x0000;
pub const ROM_BANK_N_START: u16 = 0x4000;
pub const VRAM_START: u16 = 0x8000;
pub const SRAM_START: u16 = 0xA000;
pub const WRAM_BANK_0_START: u16 = 0xC000;
pub const WRAM_BANK_N_START: u16 = 0xD000;
pub const ERAM_START: u16 = 0xE000;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFEA0;
pub const WAVE_PAT_START: u16 = 0xFF30;
pub const WAVE_PAT_END: u16 = 0xFF40;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFF;

pub const JOYPAD_REG: u16 = 0xFF00;
pub const SERIAL_0_REG: u16 = 0xFF01;
pub const SERIAL_1_REG: u16 = 0xFF02;
pub const DIVIDER_REG: u16 = 0xFF04;
pub const TIMER_COUNT_REG: u16 = 0xFF05;
pub const TIMER_MOD_REG: u16 = 0xFF06;
pub const TIMER_CTRL_REG: u16 = 0xFF07;
pub const IF_REG: u16 = 0xFF0F;
pub const CH1_SWEEP_REG: u16 = 0xFF10;
pub const CH1_DUTY_LENGTH_REG: u16 = 0xFF11;
pub const CH1_VOLUME_ENV_REG: u16 = 0xFF12;
pub const CH1_PERIOD_LOW_REG: u16 = 0xFF13;
pub const CH1_PERIOD_HIGH_CTRL_REG: u16 = 0xFF14;
pub const CH2_DUTY_LENGTH_REG: u16 = 0xFF16;
pub const CH2_VOLUME_ENV_REG: u16 = 0xFF17;
pub const CH2_PERIOD_LOW_REG: u16 = 0xFF18;
pub const CH2_PERIOD_HIGH_CTRL_REG: u16 = 0xFF19;
pub const CH3_DAC_REG: u16 = 0xFF1A;
pub const CH3_LENGTH_REG: u16 = 0xFF1B;
pub const CH3_OUTPUT_LEVEL_REG: u16 = 0xFF1C;
pub const CH3_PERIOD_LOW_REG: u16 = 0xFF1D;
pub const CH3_PERIOD_HIGH_CTRL_REG: u16 = 0xFF1E;
pub const CH4_LENGTH_REG: u16 = 0xFF20;
pub const CH4_VOLUME_ENV_REG: u16 = 0xFF21;
pub const CH4_FREQ_RAND_REG: u16 = 0xFF22;
pub const CH4_CTRL_REG: u16 = 0xFF23;
pub const VIN_VOLUME_REG: u16 = 0xFF24;
pub const PANNING_REG: u16 = 0xFF25;
pub const AUDIO_MASTER_REG: u16 = 0xFF26;
pub const LCD_CTRL_REG: u16 = 0xFF40;
pub const LCD_STAT_REG: u16 = 0xFF41;
pub const SCROLL_Y_REG: u16 = 0xFF42;
pub const SCROLL_X_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const OAM_DMA_REG: u16 = 0xFF46;
pub const BG_PALETTE_REG: u16 = 0xFF47;
pub const OBJ_PALETTE_0_REG: u16 = 0xFF48;
pub const OBJ_PALETTE_1_REG: u16 = 0xFF49;
pub const WINDOW_Y_REG: u16 = 0xFF4A;
pub const WINDOW_X_REG: u16 = 0xFF4B;
pub const KEY0_REG: u16 = 0xFF4C;
pub const KEY1_REG: u16 = 0xFF4D;
pub const VRAM_BANK_REG: u16 = 0xFF4F;
pub const BOOT_ROM_CTRL_REG: u16 = 0xFF50;
pub const VRAM_DMA_SRC_0_REG: u16 = 0xFF51;
pub const VRAM_DMA_SRC_1_REG: u16 = 0xFF52;
pub const VRAM_DMA_DEST_0_REG: u16 = 0xFF53;
pub const VRAM_DMA_DEST_1_REG: u16 = 0xFF54;
pub const VRAM_DMA_CTRL_REG: u16 = 0xFF55;
pub const IR_PORT_REG: u16 = 0xFF56;
pub const BG_COLOR_PALETTE_SPEC_REG: u16 = 0xFF68;
pub const BG_COLOR_PALETTE_DATA_REG: u16 = 0xFF69;
pub const OBJ_COLOR_PALETTE_SPEC_REG: u16 = 0xFF6A;
pub const OBJ_COLOR_PALETTE_DATA_REG: u16 = 0xFF6B;
pub const OBJ_PRIORITY_MODE_REG: u16 = 0xFF6C;
pub const WRAM_BANK_REG: u16 = 0xFF70;
pub const IE_REG: u16 = 0xFFFF;

const TILES_LEN: usize = (0x9800 - VRAM_START) as usize / std::mem::size_of::<Tile>();

pub type Tile = [(u8, u8); 8];

type Sram = Box<ByteArray<{ 8 * 1024 }>>;

#[derive(Serialize, Deserialize)]
pub struct Memory {
    mode: Mode,
    #[serde(skip)]
    boot_rom: Vec<u8>,
    #[serde(skip)]
    cart: Cart,
    mbc: Mbc,
    lock: Lock,
    #[serde(with = "serde_bytes")]
    vram: [u8; { SRAM_START - VRAM_START } as _],
    vram_cgb: Option<Box<ByteArray<{ 8 * 1024 }>>>,
    wram: [ByteArray<{ 4 * 1024 }>; 2],
    wram_cgb: Option<Box<[ByteArray<{ 4 * 1024 }>; 6]>>,
    #[serde(with = "serde_bytes")]
    oam: [u8; 160],
    joypad: Joypad,
    joypad_reg: u8,
    serial_transfer: [u8; 2],
    timer: Timer,
    interrupts: u8,
    audio: Audio,
    lcd: Lcd,
    oam_dma: u8,
    oam_dma_ticks: Option<u16>,
    cgb_key0: u8,
    cgb_key1: u8,
    cgb_vram_bank: u8,
    boot_rom_ctrl: u8,
    cgb_vram_dma_src: [u8; 2],
    cgb_vram_dma_dest: [u8; 2],
    cgb_vram_dma_ctrl: u8,
    cgb_ir: u8,
    cgb_obj_priority: u8,
    cgb_wram_bank: u8,
    #[serde(with = "serde_bytes")]
    hram: [u8; (HRAM_END - HRAM_START) as _],
    ie: u8,
}

#[derive(Debug)]
pub enum Error {
    Op(opcode::Error),
    OutOfBounds(u16),
    SegFault,
}

#[derive(Serialize, Deserialize, Debug)]
enum Mbc {
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
        #[serde(with = "serde_bytes")]
        sram_4bit: [u8; 512],
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
enum Mbc1ExtBank {
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
    fn from_cart(cart: &Cart) -> Self {
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
                        sram_4bit: [0; _],
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

    fn bank_and_cart_addr(&self, addr: u16) -> Option<(u16, usize)> {
        match addr {
            ROM_BANK_0_START..ROM_BANK_N_START => {
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

            ROM_BANK_N_START..VRAM_START => match self {
                Self::None { .. } => Some((0, addr.into())),
                Self::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank: Mbc1ExtBank::Ram { .. },
                    ..
                } => {
                    let bank =
                        if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg } & rom_bank_reg_mask;
                    let addr = ((bank as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
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
                    let addr = ((bank as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Some((bank.into(), addr))
                }
                Self::Two { rom_bank_reg, .. } | Self::Three { rom_bank_reg, .. } => {
                    let bank = if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg };
                    let addr = ((bank as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Some((bank.into(), addr))
                }
                Self::Five { rom_bank_reg, .. } => {
                    // no bank == 0 check here
                    let addr =
                        ((*rom_bank_reg as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Some(((addr >> 14) as _, addr))
                }
            },

            _ => None,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Lock {
    Unlocked,
    Oam,
    VramOam,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Audio {
    master: u8,
    panning: u8,
    vin_volume: u8,
    // pulse w/ period sweep
    ch1_sweep: u8,
    ch1_duty_length: u8,
    ch1_volume_env: u8,
    ch1_period_low: u8,
    ch1_period_high_ctrl: u8,
    // pulse
    ch2_duty_length: u8,
    ch2_volume_env: u8,
    ch2_period_low: u8,
    ch2_period_high_ctrl: u8,
    // wave
    ch3_dac: u8,
    ch3_length: u8,
    ch3_output_level: u8,
    ch3_period_low: u8,
    ch3_period_high_ctrl: u8,
    ch3_wave_pattern: [u8; 16],
    // noise
    ch4_length: u8,
    ch4_volume_env: u8,
    ch4_freq_rand: u8,
    ch4_ctrl: u8,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Lcd {
    ctrl: u8,
    stat: u8,
    scroll_y: u8,
    scroll_x: u8,
    ly: u8,
    lyc: u8,
    bg_palette: u8,
    obj_palettes: [u8; 2],
    window_y: u8,
    window_x_plus_7: u8,
    cgb_bg_palettes: [[Rgb555; 4]; 8],
    cgb_obj_palettes: [[Rgb555; 4]; 8],
    cgb_bg_palette_spec: u8,
    cgb_obj_palette_spec: u8,
}

impl Memory {
    pub fn init(boot_rom: Vec<u8>, cart: Cart, mode: Mode) -> Self {
        let is_cgb = mode == Mode::Cgb;
        let mbc = Mbc::from_cart(&cart);
        Self {
            mode,
            boot_rom,
            cart,
            mbc,
            lock: Lock::Unlocked,
            vram: [0; _],
            vram_cgb: is_cgb.then(Default::default),
            wram: Default::default(),
            wram_cgb: is_cgb.then(Default::default),
            oam: [0; _],
            joypad: Default::default(),
            joypad_reg: 0,
            serial_transfer: [0, 0],
            timer: Default::default(),
            interrupts: 0,
            audio: Default::default(),
            lcd: Default::default(),
            oam_dma: 0,
            oam_dma_ticks: None,
            cgb_key0: 0,
            cgb_key1: 0,
            cgb_vram_bank: 0,
            boot_rom_ctrl: 0,
            cgb_vram_dma_src: [0, 0],
            cgb_vram_dma_dest: [0, 0],
            cgb_vram_dma_ctrl: 0,
            cgb_ir: 0,
            cgb_obj_priority: 0,
            cgb_wram_bank: 0,
            hram: [0; _],
            ie: 0,
        }
    }

    pub fn bank(&self, addr: u16) -> Option<u16> {
        self.mbc.bank_and_cart_addr(addr).map(|(bank, _)| bank)
    }

    pub fn set_cart(&mut self, cart: Cart) {
        self.cart = cart;
    }

    pub fn reset_mbc(&mut self) {
        self.mbc = Mbc::from_cart(&self.cart);
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        let timer_result = self.timer.tick();
        if timer_result.interrupt {
            self.interrupts |= 0b00000100;
        }
        if timer_result.div_apu {
            // TODO
        }
        match &mut self.oam_dma_ticks {
            Some(0) => {
                self.oam_dma_ticks = None;
                let source = (self.oam_dma as u16) << 8;
                for offset in 0..160 {
                    self.write(OAM_START + offset, self.read(source + offset)?)?;
                }
            }
            Some(ticks) => *ticks -= 1,
            None => {}
        }
        Ok(())
    }

    pub fn set_joypad(&mut self, joypad: Joypad) {
        let joyp_before = self.joypad_reg;
        self.joypad = joypad;
        self.write(JOYPAD_REG, joyp_before).expect("valid address");
        if self.read(JOYPAD_REG).unwrap() != 0xff {
            info!("joyp_reg: {:b}", self.read(JOYPAD_REG).unwrap());
        }
        if [0b00000001, 0b00000010, 0b00000100, 0b00001000]
            .iter()
            .any(|b| joyp_before & b != 0 && self.joypad_reg & b == 0)
        {
            // trigger interrupt if any buttons went hi -> lo
            self.interrupts |= 0b00010000;
        }
    }

    pub fn set_lock(&mut self, lock: Lock) {
        self.lock = lock;
    }

    pub fn read(&self, addr: u16) -> Result<u8, Error> {
        self.read_inner(addr, false).map(|mem| mem[0])
    }

    pub fn read_ppu(&self, addr: u16) -> Result<u8, Error> {
        self.read_inner(addr, true).map(|mem| mem[0])
    }

    pub fn read_op(&self, pc: u16) -> Result<(Op, u16), Error> {
        let mem = self.read_inner(pc, false)?;
        Op::read(mem)
            .map(|(op, new_mem)| (op, pc + (mem.len() - new_mem.len()) as u16))
            .map_err(Error::Op)
    }

    pub fn oam(&self) -> &[u8; 160] {
        &self.oam
    }

    fn read_inner(&self, addr: u16, ppu: bool) -> Result<&[u8], Error> {
        fn as_slice(byte: &u8) -> &[u8] {
            std::slice::from_ref(byte)
        }

        match addr {
            ROM_BANK_0_START..ROM_BANK_N_START
                if (addr as usize) < self.boot_rom.len() && self.read(BOOT_ROM_CTRL_REG)? == 0 =>
            {
                Ok(&self.boot_rom[addr.into()..])
            }

            ROM_BANK_0_START..VRAM_START => {
                let (_, addr) = self
                    .mbc
                    .bank_and_cart_addr(addr)
                    .ok_or(Error::OutOfBounds(addr))?;
                Ok(&self.cart.data()[addr..])
            }

            VRAM_START..SRAM_START => {
                if !ppu && self.lock == Lock::VramOam {
                    Ok(&[0xFF; 16])
                } else {
                    match self.mode {
                        Mode::Cgb if self.read(VRAM_BANK_REG)? != 0 => {
                            Ok(&self.vram_cgb.as_ref().expect("is_some if cgb")
                                [(addr - VRAM_START).into()..])
                        }
                        _ => Ok(&self.vram[(addr - VRAM_START).into()..]),
                    }
                }
            }

            SRAM_START..WRAM_BANK_0_START => match &self.mbc {
                Mbc::None { sram } => Ok(&sram[(addr - SRAM_START).into()..]),
                Mbc::One {
                    sram_enabled: false,
                    ..
                }
                | Mbc::Two {
                    sram_enabled: false,
                    ..
                }
                | Mbc::Three {
                    sram_and_rtc_enabled: false,
                    ..
                }
                | Mbc::Five {
                    sram_enabled: false,
                    ..
                } => Ok(&[0xFF; 16]),
                Mbc::One {
                    extended_bank: Mbc1ExtBank::Rom { sram, .. },
                    ..
                } => Ok(&sram[(addr - SRAM_START).into()..]),
                Mbc::One {
                    extended_bank:
                        Mbc1ExtBank::Ram {
                            advanced,
                            sram_bank_reg,
                            sram,
                        },
                    ..
                } => {
                    let sram_bank = if *advanced {
                        *sram_bank_reg as usize
                    } else {
                        0
                    };
                    Ok(&sram[sram_bank][(addr - SRAM_START).into()..])
                }
                Mbc::Two { sram_4bit, .. } => Ok(&sram_4bit[(addr & 0x01FF).into()..]),
                Mbc::Three {
                    sram_bank_or_rtc_reg: sram_bank @ 0x00..=0x07,
                    sram,
                    ..
                } => Ok(&sram[*sram_bank as usize][(addr - SRAM_START).into()..]),
                Mbc::Three {
                    sram_bank_or_rtc_reg,
                    rtc,
                    ..
                } => Ok(&rtc[(sram_bank_or_rtc_reg - 0x08).into()..]),
                Mbc::Five {
                    sram_bank_reg,
                    sram,
                    ..
                } => Ok(&sram[*sram_bank_reg as usize][(addr - SRAM_START).into()..]),
            },

            WRAM_BANK_0_START..WRAM_BANK_N_START => {
                Ok(&self.wram[0][(addr - WRAM_BANK_0_START).into()..])
            }

            WRAM_BANK_N_START..ERAM_START => match self.mode {
                Mode::Dmg => Ok(&self.wram[1][(addr - WRAM_BANK_N_START).into()..]),
                _ => match self.read(WRAM_BANK_REG)? {
                    0 | 1 => Ok(&self.wram[1][(addr - WRAM_BANK_N_START).into()..]),
                    wram_bank => Ok(&self.wram_cgb.as_ref().expect("is_some if cgb")
                        [wram_bank as usize][(addr - WRAM_BANK_N_START).into()..]),
                },
            },

            ERAM_START..OAM_START => self.read_inner(addr - (ERAM_START - WRAM_BANK_0_START), ppu),

            OAM_START..OAM_END => {
                if ppu || self.lock == Lock::Unlocked {
                    Ok(&self.oam[(addr - OAM_START).into()..])
                } else {
                    Ok(&[0xFF; 16])
                }
            }

            JOYPAD_REG => Ok(as_slice(&self.joypad_reg)),

            SERIAL_0_REG => Ok(&self.serial_transfer),
            SERIAL_1_REG => Ok(&self.serial_transfer[1..]),

            DIVIDER_REG => Ok(self.timer.read_div()),
            TIMER_COUNT_REG => Ok(as_slice(&self.timer.tima)),
            TIMER_MOD_REG => Ok(as_slice(&self.timer.tma)),
            TIMER_CTRL_REG => Ok(as_slice(&self.timer.tac)),

            IF_REG => Ok(as_slice(&self.interrupts)),

            CH1_SWEEP_REG => Ok(as_slice(&self.audio.ch1_sweep)),
            CH1_DUTY_LENGTH_REG => Ok(as_slice(&self.audio.ch1_duty_length)),
            CH1_VOLUME_ENV_REG => Ok(as_slice(&self.audio.ch1_volume_env)),
            CH1_PERIOD_LOW_REG => Ok(as_slice(&self.audio.ch1_period_low)),
            CH1_PERIOD_HIGH_CTRL_REG => Ok(as_slice(&self.audio.ch1_period_high_ctrl)),
            CH2_DUTY_LENGTH_REG => Ok(as_slice(&self.audio.ch2_duty_length)),
            CH2_VOLUME_ENV_REG => Ok(as_slice(&self.audio.ch2_volume_env)),
            CH2_PERIOD_LOW_REG => Ok(as_slice(&self.audio.ch2_period_low)),
            CH2_PERIOD_HIGH_CTRL_REG => Ok(as_slice(&self.audio.ch2_period_high_ctrl)),
            CH3_DAC_REG => Ok(as_slice(&self.audio.ch3_dac)),
            CH3_LENGTH_REG => Ok(as_slice(&self.audio.ch3_length)),
            CH3_OUTPUT_LEVEL_REG => Ok(as_slice(&self.audio.ch3_output_level)),
            CH3_PERIOD_LOW_REG => Ok(as_slice(&self.audio.ch3_period_low)),
            CH3_PERIOD_HIGH_CTRL_REG => Ok(as_slice(&self.audio.ch3_period_high_ctrl)),
            CH4_LENGTH_REG => Ok(as_slice(&self.audio.ch4_length)),
            CH4_VOLUME_ENV_REG => Ok(as_slice(&self.audio.ch4_volume_env)),
            CH4_FREQ_RAND_REG => Ok(as_slice(&self.audio.ch4_freq_rand)),
            CH4_CTRL_REG => Ok(as_slice(&self.audio.ch4_ctrl)),
            VIN_VOLUME_REG => Ok(as_slice(&self.audio.vin_volume)),
            PANNING_REG => Ok(as_slice(&self.audio.panning)),
            AUDIO_MASTER_REG => Ok(as_slice(&self.audio.master)),
            WAVE_PAT_START..WAVE_PAT_END => {
                Ok(&self.audio.ch3_wave_pattern[(addr - WAVE_PAT_START).into()..])
            }

            LCD_CTRL_REG => Ok(as_slice(&self.lcd.ctrl)),
            LCD_STAT_REG => Ok(as_slice(&self.lcd.stat)),
            SCROLL_Y_REG => Ok(as_slice(&self.lcd.scroll_y)),
            SCROLL_X_REG => Ok(as_slice(&self.lcd.scroll_x)),
            LY_REG => Ok(as_slice(&self.lcd.ly)),
            LYC_REG => Ok(as_slice(&self.lcd.lyc)),

            OAM_DMA_REG => Ok(as_slice(&self.oam_dma)),

            BG_PALETTE_REG => Ok(as_slice(&self.lcd.bg_palette)),
            OBJ_PALETTE_0_REG => Ok(&self.lcd.obj_palettes),
            OBJ_PALETTE_1_REG => Ok(&self.lcd.obj_palettes[1..]),
            WINDOW_Y_REG => Ok(as_slice(&self.lcd.window_y)),
            WINDOW_X_REG => Ok(as_slice(&self.lcd.window_x_plus_7)),

            KEY0_REG => Ok(as_slice(&self.cgb_key0)),
            KEY1_REG => Ok(as_slice(&self.cgb_key1)),

            VRAM_BANK_REG => Ok(as_slice(&self.cgb_vram_bank)),

            BOOT_ROM_CTRL_REG => Ok(as_slice(&self.boot_rom_ctrl)),

            VRAM_DMA_SRC_0_REG => Ok(&self.cgb_vram_dma_src),
            VRAM_DMA_SRC_1_REG => Ok(&self.cgb_vram_dma_src[1..]),
            VRAM_DMA_DEST_0_REG => Ok(&self.cgb_vram_dma_dest),
            VRAM_DMA_DEST_1_REG => Ok(&self.cgb_vram_dma_dest[1..]),
            VRAM_DMA_CTRL_REG => Ok(as_slice(&self.cgb_vram_dma_ctrl)),

            IR_PORT_REG => Ok(as_slice(&self.cgb_ir)),

            BG_COLOR_PALETTE_SPEC_REG => Ok(as_slice(&self.lcd.cgb_bg_palette_spec)),
            BG_COLOR_PALETTE_DATA_REG => {
                if !ppu && self.lock == Lock::VramOam {
                    Ok(&[0xFF; 16])
                } else {
                    let spec = self.read(BG_COLOR_PALETTE_SPEC_REG)?;
                    let palette = (spec & 0b00111000) as usize >> 3;
                    let color = (spec & 0b00000110) as usize >> 1;
                    Ok(as_slice(
                        &self.lcd.cgb_bg_palettes[palette][color][(spec % 2) as usize],
                    ))
                }
            }
            OBJ_COLOR_PALETTE_SPEC_REG => Ok(as_slice(&self.lcd.cgb_obj_palette_spec)),
            OBJ_COLOR_PALETTE_DATA_REG => {
                if !ppu && self.lock == Lock::VramOam {
                    Ok(&[0xFF; 16])
                } else {
                    let spec = self.read(OBJ_COLOR_PALETTE_SPEC_REG)?;
                    let palette = (spec & 0b00111000) as usize >> 3;
                    let color = (spec & 0b00000110) as usize >> 1;
                    Ok(as_slice(
                        &self.lcd.cgb_obj_palettes[palette][color][(spec % 2) as usize],
                    ))
                }
            }

            OBJ_PRIORITY_MODE_REG => Ok(as_slice(&self.cgb_obj_priority)),

            WRAM_BANK_REG => Ok(as_slice(&self.cgb_wram_bank)),

            HRAM_START..HRAM_END => Ok(&self.hram[(addr - HRAM_START).into()..]),

            IE_REG => Ok(as_slice(&self.ie)),

            _ => return Ok(&[0xFF]),
            //TODO _ => Err(Error::OutOfBounds(addr)),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        self.write_slice_inner(addr, &[data], false)
    }

    pub fn write_ppu(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        self.write_slice_inner(addr, &[data], true)
    }

    pub fn write_slice(&mut self, addr: u16, data: &[u8]) -> Result<(), Error> {
        self.write_slice_inner(addr, data, false)
    }

    fn write_slice_inner(&mut self, addr: u16, data: &[u8], ppu: bool) -> Result<(), Error> {
        trace!(addr:? = Hex(addr), data:? = Hex(data); "mem write");
        fn as_slice(byte: &mut u8) -> &mut [u8] {
            std::slice::from_mut(byte)
        }

        let slice = match addr {
            ROM_BANK_0_START..VRAM_START => {
                match &mut self.mbc {
                    Mbc::One { sram_enabled, .. } if addr < 0x2000 => {
                        *sram_enabled = data[0] & 0b00001111 == 0x0A;
                    }
                    Mbc::One { rom_bank_reg, .. } if addr < 0x4000 => {
                        *rom_bank_reg = data[0] & 0b00011111;
                    }
                    Mbc::One {
                        extended_bank:
                            Mbc1ExtBank::Ram {
                                sram_bank_reg: target,
                                ..
                            }
                            | Mbc1ExtBank::Rom {
                                rom_bank_upper_reg: target,
                                ..
                            },
                        ..
                    } if addr < 0x6000 => {
                        *target = data[0] & 0b00000011;
                    }
                    Mbc::One {
                        extended_bank:
                            Mbc1ExtBank::Ram { advanced, .. } | Mbc1ExtBank::Rom { advanced, .. },
                        ..
                    } => {
                        *advanced = data[0] % 2 == 1;
                    }
                    Mbc::Two {
                        rom_bank_reg,
                        sram_enabled,
                        ..
                    } if addr < 0x4000 => {
                        if addr & 0x0100 == 0 {
                            *sram_enabled = data[0] & 0b00001111 == 0x0A;
                        } else {
                            *rom_bank_reg = data[0] & 0b00001111;
                        }
                    }
                    Mbc::Three {
                        sram_and_rtc_enabled,
                        ..
                    } if addr < 0x2000 => {
                        *sram_and_rtc_enabled = data[0] & 0b00001111 == 0x0A;
                    }
                    Mbc::Three { rom_bank_reg, .. } if addr < 0x4000 => {
                        *rom_bank_reg = data[0] & 0b01111111;
                    }
                    Mbc::Three {
                        sram_bank_or_rtc_reg,
                        ..
                    } if addr < 0x6000 => {
                        if data[0] > 0x0C {
                            panic!("unexpected SRAM bank / RTC register value")
                        };
                        *sram_bank_or_rtc_reg = data[0];
                    }
                    Mbc::Three { latching, .. } => {
                        if data[0] == 0x00 {
                            *latching = true;
                        } else if data[0] == 0x01 {
                            *latching = false;
                        }
                    }
                    Mbc::Five { sram_enabled, .. } if addr < 0x2000 => {
                        *sram_enabled = data[0] & 0b00001111 == 0x0A;
                    }
                    Mbc::Five { rom_bank_reg, .. } if addr < 0x3000 => {
                        *rom_bank_reg = (*rom_bank_reg & 0xFF00) + data[0] as u16;
                    }
                    Mbc::Five { rom_bank_reg, .. } if addr < 0x4000 => {
                        *rom_bank_reg = ((data[0] as u16 & 0x0001) << 8) + (*rom_bank_reg & 0x00FF);
                    }
                    Mbc::Five { sram_bank_reg, .. } if addr < 0x6000 => {
                        *sram_bank_reg = data[0] & 0x0F;
                    }
                    _ => {}
                }
                return Ok(());
            }

            VRAM_START..SRAM_START => {
                if self.lock == Lock::VramOam {
                    return Ok(());
                } else {
                    match self.mode {
                        Mode::Cgb if self.read(VRAM_BANK_REG)? != 0 => {
                            &mut self.vram_cgb.as_mut().expect("is_some if cgb")
                                [(addr - VRAM_START).into()..]
                        }
                        _ => &mut self.vram[(addr - VRAM_START).into()..],
                    }
                }
            }

            SRAM_START..WRAM_BANK_0_START => match &mut self.mbc {
                Mbc::None { sram } => &mut sram[(addr - SRAM_START).into()..],
                Mbc::One {
                    sram_enabled: false,
                    ..
                }
                | Mbc::Two {
                    sram_enabled: false,
                    ..
                }
                | Mbc::Three {
                    sram_and_rtc_enabled: false,
                    ..
                }
                | Mbc::Five {
                    sram_enabled: false,
                    ..
                } => return Ok(()),
                Mbc::One {
                    extended_bank: Mbc1ExtBank::Rom { sram, .. },
                    ..
                } => &mut sram[(addr - SRAM_START).into()..],
                Mbc::One {
                    extended_bank:
                        Mbc1ExtBank::Ram {
                            advanced,
                            sram_bank_reg,
                            sram,
                        },
                    ..
                } => {
                    let sram_bank = if *advanced {
                        *sram_bank_reg as usize
                    } else {
                        0
                    };
                    &mut sram[sram_bank][(addr - SRAM_START).into()..]
                }
                Mbc::Two { sram_4bit, .. } => &mut sram_4bit[(addr & 0x01FF).into()..],
                Mbc::Three {
                    sram_bank_or_rtc_reg: sram_bank @ 0x00..=0x07,
                    sram,
                    ..
                } => &mut sram[*sram_bank as usize][(addr - SRAM_START).into()..],
                Mbc::Three {
                    sram_bank_or_rtc_reg,
                    rtc,
                    ..
                } => &mut rtc[(*sram_bank_or_rtc_reg - 0x08).into()..],
                Mbc::Five {
                    sram_bank_reg,
                    sram,
                    ..
                } => &mut sram[*sram_bank_reg as usize][(addr - SRAM_START).into()..],
            },

            WRAM_BANK_0_START..WRAM_BANK_N_START => {
                &mut self.wram[0][(addr - WRAM_BANK_0_START).into()..]
            }

            WRAM_BANK_N_START..ERAM_START => match self.mode {
                Mode::Dmg => &mut self.wram[1][(addr - WRAM_BANK_N_START).into()..],
                _ => match self.read(WRAM_BANK_REG)? {
                    0 | 1 => &mut self.wram[1][(addr - WRAM_BANK_N_START).into()..],
                    wram_bank => &mut self.wram_cgb.as_mut().expect("is_some if cgb")
                        [wram_bank as usize][(addr - WRAM_BANK_N_START).into()..],
                },
            },

            ERAM_START..OAM_START => {
                return self.write_slice_inner(addr - (ERAM_START - WRAM_BANK_0_START), data, ppu);
            }

            OAM_START..OAM_END => {
                if self.lock == Lock::Unlocked {
                    &mut self.oam[(addr - OAM_START).into()..]
                } else {
                    return Ok(());
                }
            }

            JOYPAD_REG => {
                let &[selection] = data else {
                    return Err(Error::SegFault);
                };
                // bits are inverted. 0 = on, 1 = off
                self.joypad_reg = match (selection & 0b00100000 != 0, selection & 0b00010000 != 0) {
                    (true, true) => 0xFF,
                    (true, false) => {
                        0b11101111
                            & if self.joypad.right { 0b11111110 } else { 0xFF }
                            & if self.joypad.left { 0b11111101 } else { 0xFF }
                            & if self.joypad.up { 0b11111011 } else { 0xFF }
                            & if self.joypad.down { 0b11110111 } else { 0xFF }
                    }
                    (false, true) => {
                        0b11011111
                            & if self.joypad.a { 0b11111110 } else { 0xFF }
                            & if self.joypad.b { 0b11111101 } else { 0xFF }
                            & if self.joypad.select { 0b11111011 } else { 0xFF }
                            & if self.joypad.start { 0b11110111 } else { 0xFF }
                    }
                    (false, false) => {
                        0b11001111
                            & if self.joypad.right || self.joypad.a {
                                0b11111110
                            } else {
                                0xFF
                            }
                            & if self.joypad.left || self.joypad.b {
                                0b11111101
                            } else {
                                0xFF
                            }
                            & if self.joypad.up || self.joypad.select {
                                0b11111011
                            } else {
                                0xFF
                            }
                            & if self.joypad.down || self.joypad.start {
                                0b11110111
                            } else {
                                0xFF
                            }
                    }
                };
                return Ok(());
            }

            SERIAL_0_REG => &mut self.serial_transfer,
            SERIAL_1_REG => &mut self.serial_transfer[1..],

            DIVIDER_REG => {
                self.timer.write_div();
                return Ok(());
            }
            TIMER_COUNT_REG => as_slice(&mut self.timer.tima),
            TIMER_MOD_REG => as_slice(&mut self.timer.tma),
            TIMER_CTRL_REG => as_slice(&mut self.timer.tac),

            IF_REG => as_slice(&mut self.interrupts),

            CH1_SWEEP_REG => as_slice(&mut self.audio.ch1_sweep),
            CH1_DUTY_LENGTH_REG => as_slice(&mut self.audio.ch1_duty_length),
            CH1_VOLUME_ENV_REG => as_slice(&mut self.audio.ch1_volume_env),
            CH1_PERIOD_LOW_REG => as_slice(&mut self.audio.ch1_period_low),
            CH1_PERIOD_HIGH_CTRL_REG => as_slice(&mut self.audio.ch1_period_high_ctrl),
            CH2_DUTY_LENGTH_REG => as_slice(&mut self.audio.ch2_duty_length),
            CH2_VOLUME_ENV_REG => as_slice(&mut self.audio.ch2_volume_env),
            CH2_PERIOD_LOW_REG => as_slice(&mut self.audio.ch2_period_low),
            CH2_PERIOD_HIGH_CTRL_REG => as_slice(&mut self.audio.ch2_period_high_ctrl),
            CH3_DAC_REG => as_slice(&mut self.audio.ch3_dac),
            CH3_LENGTH_REG => as_slice(&mut self.audio.ch3_length),
            CH3_OUTPUT_LEVEL_REG => as_slice(&mut self.audio.ch3_output_level),
            CH3_PERIOD_LOW_REG => as_slice(&mut self.audio.ch3_period_low),
            CH3_PERIOD_HIGH_CTRL_REG => as_slice(&mut self.audio.ch3_period_high_ctrl),
            CH4_LENGTH_REG => as_slice(&mut self.audio.ch4_length),
            CH4_VOLUME_ENV_REG => as_slice(&mut self.audio.ch4_volume_env),
            CH4_FREQ_RAND_REG => as_slice(&mut self.audio.ch4_freq_rand),
            CH4_CTRL_REG => as_slice(&mut self.audio.ch4_ctrl),
            VIN_VOLUME_REG => as_slice(&mut self.audio.vin_volume),
            PANNING_REG => as_slice(&mut self.audio.panning),
            AUDIO_MASTER_REG => as_slice(&mut self.audio.master),
            WAVE_PAT_START..WAVE_PAT_END => {
                &mut self.audio.ch3_wave_pattern[(addr - WAVE_PAT_START).into()..]
            }

            LCD_CTRL_REG => as_slice(&mut self.lcd.ctrl),
            LCD_STAT_REG => {
                self.lcd.stat = (data[0] & 0b11111100) | (self.lcd.stat & 0b00000011);
                return Ok(());
            }
            SCROLL_Y_REG => as_slice(&mut self.lcd.scroll_y),
            SCROLL_X_REG => as_slice(&mut self.lcd.scroll_x),
            LY_REG => as_slice(&mut self.lcd.ly),
            LYC_REG => as_slice(&mut self.lcd.lyc),

            OAM_DMA_REG => {
                self.oam_dma_ticks = Some(640);
                as_slice(&mut self.oam_dma)
            }

            BG_PALETTE_REG => as_slice(&mut self.lcd.bg_palette),
            OBJ_PALETTE_0_REG => &mut self.lcd.obj_palettes,
            OBJ_PALETTE_1_REG => &mut self.lcd.obj_palettes[1..],
            WINDOW_Y_REG => as_slice(&mut self.lcd.window_y),
            WINDOW_X_REG => as_slice(&mut self.lcd.window_x_plus_7),

            KEY0_REG => as_slice(&mut self.cgb_key0),
            KEY1_REG => as_slice(&mut self.cgb_key1),

            VRAM_BANK_REG => as_slice(&mut self.cgb_vram_bank),

            BOOT_ROM_CTRL_REG => as_slice(&mut self.boot_rom_ctrl),

            VRAM_DMA_SRC_0_REG => &mut self.cgb_vram_dma_src,
            VRAM_DMA_SRC_1_REG => &mut self.cgb_vram_dma_src[1..],
            VRAM_DMA_DEST_0_REG => &mut self.cgb_vram_dma_dest,
            VRAM_DMA_DEST_1_REG => &mut self.cgb_vram_dma_dest[1..],
            VRAM_DMA_CTRL_REG => as_slice(&mut self.cgb_vram_dma_ctrl),

            IR_PORT_REG => as_slice(&mut self.cgb_ir),

            BG_COLOR_PALETTE_SPEC_REG => as_slice(&mut self.lcd.cgb_bg_palette_spec),
            BG_COLOR_PALETTE_DATA_REG => {
                if self.lock == Lock::VramOam {
                    return Ok(());
                }
                let spec = self.read(BG_COLOR_PALETTE_SPEC_REG)?;
                if (spec & 0b10000000) != 0 {
                    // auto-increment
                    self.write(BG_COLOR_PALETTE_SPEC_REG, spec + 1)?;
                }
                let palette = (spec & 0b00111000) as usize >> 3;
                let color = (spec & 0b00000110) as usize >> 1;
                as_slice(&mut self.lcd.cgb_bg_palettes[palette][color][(spec % 2) as usize])
            }
            OBJ_COLOR_PALETTE_SPEC_REG => as_slice(&mut self.lcd.cgb_obj_palette_spec),
            OBJ_COLOR_PALETTE_DATA_REG => {
                if self.lock == Lock::VramOam {
                    return Ok(());
                }
                let spec = self.read(OBJ_COLOR_PALETTE_SPEC_REG)?;
                if (spec & 0b10000000) != 0 {
                    // auto-increment
                    self.write(OBJ_COLOR_PALETTE_SPEC_REG, spec + 1)?;
                }
                let palette = (spec & 0b00111000) as usize >> 3;
                let color = (spec & 0b00000110) as usize >> 1;
                as_slice(&mut self.lcd.cgb_obj_palettes[palette][color][(spec % 2) as usize])
            }

            OBJ_PRIORITY_MODE_REG => as_slice(&mut self.cgb_obj_priority),

            WRAM_BANK_REG => as_slice(&mut self.cgb_wram_bank),

            HRAM_START..HRAM_END => &mut self.hram[(addr - HRAM_START).into()..],

            IE_REG => as_slice(&mut self.ie),

            _ => return Ok(()),
            //TODO _ => return Err(Error::OutOfBounds(addr)),
        };

        if slice.len() < data.len() {
            Err(Error::SegFault)
        } else {
            slice[..data.len()].copy_from_slice(data);
            Ok(())
        }
    }

    pub fn log_registers(&self) {
        enum For {
            Only(Mode),
            Both,
        }
        let registers = [
            (For::Both, "JOYP", JOYPAD_REG),
            (For::Both, "SB", SERIAL_0_REG),
            (For::Both, "SC", SERIAL_1_REG),
            (For::Both, "DIV", DIVIDER_REG),
            (For::Both, "TIMA", TIMER_COUNT_REG),
            (For::Both, "TMA", TIMER_MOD_REG),
            (For::Both, "TAC", TIMER_CTRL_REG),
            (For::Both, "IE", IE_REG),
            (For::Both, "IF", IF_REG),
            (For::Both, "LCDC", LCD_CTRL_REG),
            (For::Both, "LY", LY_REG),
            (For::Both, "LYC", LYC_REG),
            (For::Both, "STAT", LCD_STAT_REG),
            (For::Both, "SCY", SCROLL_Y_REG),
            (For::Both, "SCX", SCROLL_X_REG),
            (For::Both, "WX", WINDOW_X_REG),
            (For::Both, "WY", WINDOW_Y_REG),
            (For::Only(Mode::Dmg), "BGP", BG_PALETTE_REG),
            (For::Only(Mode::Dmg), "OBP0", OBJ_PALETTE_0_REG),
            (For::Only(Mode::Dmg), "OBP1", OBJ_PALETTE_1_REG),
            (For::Only(Mode::Cgb), "BCPS", BG_COLOR_PALETTE_SPEC_REG),
            (For::Only(Mode::Cgb), "BCPD", BG_COLOR_PALETTE_DATA_REG),
            (For::Only(Mode::Cgb), "OCPS", OBJ_COLOR_PALETTE_SPEC_REG),
            (For::Only(Mode::Cgb), "OCPD", OBJ_COLOR_PALETTE_DATA_REG),
            (For::Both, "DMA", OAM_DMA_REG),
            (For::Only(Mode::Cgb), "KEY0", KEY0_REG),
            (For::Only(Mode::Cgb), "KEY1", KEY1_REG),
            (For::Only(Mode::Cgb), "VBK", VRAM_BANK_REG),
            (For::Both, "BANK", BOOT_ROM_CTRL_REG),
            (For::Only(Mode::Cgb), "HDMA1", VRAM_DMA_SRC_0_REG),
            (For::Only(Mode::Cgb), "HDMA2", VRAM_DMA_SRC_1_REG),
            (For::Only(Mode::Cgb), "HDMA3", VRAM_DMA_DEST_0_REG),
            (For::Only(Mode::Cgb), "HDMA4", VRAM_DMA_DEST_1_REG),
            (For::Only(Mode::Cgb), "HDMA5", VRAM_DMA_CTRL_REG),
            (For::Only(Mode::Cgb), "RP", IR_PORT_REG),
            (For::Only(Mode::Cgb), "OPRI", OBJ_PRIORITY_MODE_REG),
            (For::Only(Mode::Cgb), "SVBK", WRAM_BANK_REG),
        ];

        let mut prev = None;
        for (r#for, name, addr) in registers {
            match (self.mode, r#for) {
                (Mode::Dmg, For::Only(Mode::Dmg))
                | (Mode::Cgb, For::Only(Mode::Cgb))
                | (_, For::Both) => {
                    if let Some((prev_name, prev_addr)) = prev.take() {
                        info!(
                            "{prev_name:<6}(0x{prev_addr:04X}): 0x{0:02X} 0b{0:08b} | {name:<6}(0x{addr:04X}): 0x{1:02X} 0b{1:08b}",
                            self.read(prev_addr).unwrap(),
                            self.read(addr).unwrap(),
                        );
                    } else {
                        prev = Some((name, addr));
                    }
                }
                _ => {}
            }
        }
        if let Some((prev_name, prev_addr)) = prev {
            info!(
                "{prev_name:<6}(0x{prev_addr:04X}): 0x{0:02X} 0b{0:08b}",
                self.read(prev_addr).unwrap(),
            );
        }
    }

    pub fn tiles(&self) -> [Tile; TILES_LEN] {
        std::array::from_fn(|i| {
            let offset = i * std::mem::size_of::<Tile>();
            std::array::from_fn(|j| (self.vram[offset + j * 2], self.vram[offset + j * 2 + 1]))
        })
    }
}
