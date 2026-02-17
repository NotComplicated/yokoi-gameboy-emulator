use crate::{
    cart::Cart,
    frame::Rgb555,
    opcode::{self, Op},
    system::Mode,
};

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
pub const INTERRUPTS_REG: u16 = 0xFF0F;
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

type Sram = Box<[u8; 8 * 1024]>;

#[derive(Debug)]
pub struct Memory {
    mode: Mode,
    boot_rom: Vec<u8>,
    cart: Cart,
    mbc: Mbc,
    vram: [u8; 8 * 1024],
    vram_cgb: Option<Box<[u8; 8 * 1024]>>,
    wram: [[u8; 4 * 1024]; 2],
    wram_cgb: Option<Box<[[u8; 4 * 1024]; 6]>>,
    oam: [u8; 160],
    joypad: u8,
    serial_transfer: [u8; 2],
    timer_divider: [u8; 4],
    interrupts: u8,
    audio: Audio,
    lcd: Lcd,
    oam_dma: u8,
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
    hram: [u8; 127],
    ie: u8,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
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

#[derive(Debug)]
pub enum Error {
    Op(opcode::Error),
    OutOfBounds,
    SegFault,
}

impl Memory {
    pub fn init(boot_rom: Vec<u8>, cart: Cart, mode: Mode) -> Self {
        let is_cgb = matches!(mode, Mode::Gbc);
        let mbc = 'mbc: {
            for feature in cart.features() {
                match feature {
                    crate::cart::Feature::Mbc1 => {
                        let bank_count: u8 = cart
                            .data()
                            .len()
                            .div_ceil(16 * 1024)
                            .try_into()
                            .expect("cart isn't too large");
                        break 'mbc Mbc::One {
                            rom_bank_reg: 0,
                            rom_bank_reg_mask: bank_count.next_power_of_two() - 1,
                            sram_enabled: false,
                            extended_bank: if bank_count > 32 {
                                Mbc1ExtBank::Rom {
                                    advanced: false,
                                    rom_bank_upper_reg: 0,
                                    sram: Sram::new([0; _]),
                                }
                            } else {
                                Mbc1ExtBank::Ram {
                                    advanced: false,
                                    sram_bank_reg: 0,
                                    sram: std::array::repeat(Sram::new([0; _])),
                                }
                            },
                        };
                    }
                    crate::cart::Feature::Mbc2 => {
                        break 'mbc Mbc::Two {
                            rom_bank_reg: 0,
                            sram_4bit: [0; _],
                            sram_enabled: false,
                        };
                    }
                    crate::cart::Feature::Mbc3 => {
                        break 'mbc Mbc::Three {
                            rom_bank_reg: 0,
                            sram_bank_or_rtc_reg: 0,
                            sram_and_rtc_enabled: false,
                            sram: std::array::repeat(Sram::new([0; _])),
                            latching: false,
                            rtc: [0; _],
                        };
                    }
                    crate::cart::Feature::Mbc5 => {
                        break 'mbc Mbc::Five {
                            rom_bank_reg: 0,
                            sram_enabled: false,
                            sram_bank_reg: 0,
                            sram: std::array::repeat(Sram::new([0; _])),
                        };
                    }
                    crate::cart::Feature::Mbc6 | crate::cart::Feature::Mbc7 => {
                        unimplemented!("very rare cart")
                    }
                    _ => {}
                }
            }
            Mbc::None {
                sram: Sram::new([0; _]),
            }
        };

        Self {
            mode,
            boot_rom,
            cart,
            mbc,
            vram: [0; _],
            vram_cgb: is_cgb.then(|| Box::new([0; _])),
            wram: [[0; _]; _],
            wram_cgb: is_cgb.then(|| Box::new([[0; _]; _])),
            oam: [0; _],
            joypad: 0,
            serial_transfer: [0, 0],
            timer_divider: [0, 0, 0, 0],
            interrupts: 0,
            audio: Default::default(),
            lcd: Default::default(),
            oam_dma: 0,
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

    pub fn read(&self, addr: u16) -> Result<u8, Error> {
        self.read_inner(addr).map(|mem| mem[0])
    }

    pub fn read_op(&self, pc: u16) -> Result<(Op, u16), Error> {
        let mem = self.read_inner(pc)?;
        Op::read(mem)
            .map(|(op, new_mem)| (op, pc + (new_mem.len() - mem.len()) as u16))
            .map_err(Error::Op)
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        self.write_slice(addr, &[data])
    }

    fn read_inner(&self, addr: u16) -> Result<&[u8], Error> {
        fn as_slice(byte: &u8) -> &[u8] {
            std::slice::from_ref(byte)
        }

        match addr {
            ROM_BANK_0_START..ROM_BANK_N_START
                if (addr as usize) < self.boot_rom.len() && self.read(BOOT_ROM_CTRL_REG)? != 0 =>
            {
                Ok(&self.boot_rom[addr.into()..])
            }

            ROM_BANK_0_START..ROM_BANK_N_START => match self.mbc {
                Mbc::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank:
                        Mbc1ExtBank::Rom {
                            advanced: true,
                            rom_bank_upper_reg,
                            ..
                        },
                    ..
                } => {
                    // MBC1 advanced mode on 1MB+ cart, bank # comes from upper/lower regs
                    let rom_bank_lower = rom_bank_reg & rom_bank_reg_mask;
                    let rom_bank = (rom_bank_upper_reg << 5) + rom_bank_lower;
                    let addr = ((rom_bank as usize) << 14) + addr as usize;
                    Ok(&self.cart.data()[addr..])
                }
                _ => {
                    // otherwise, simply read the first ROM bank
                    Ok(&self.cart.data()[addr.into()..])
                }
            },

            ROM_BANK_N_START..VRAM_START => match &self.mbc {
                Mbc::None { .. } => Ok(&self.cart.data()[(addr - ROM_BANK_N_START).into()..]),
                Mbc::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank: Mbc1ExtBank::Ram { .. },
                    ..
                } => {
                    let addr = (((rom_bank_reg & rom_bank_reg_mask) as usize) << 14)
                        + (addr - ROM_BANK_N_START) as usize;
                    Ok(&self.cart.data()[addr..])
                }
                Mbc::One {
                    rom_bank_reg,
                    rom_bank_reg_mask,
                    extended_bank:
                        Mbc1ExtBank::Rom {
                            rom_bank_upper_reg, ..
                        },
                    ..
                } => {
                    // bank == 0 check must come *before* mask check
                    let rom_bank_lower = if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg };
                    let rom_bank_lower = rom_bank_lower & rom_bank_reg_mask;
                    let rom_bank = (rom_bank_upper_reg << 5) + rom_bank_lower;
                    let addr = ((rom_bank as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Ok(&self.cart.data()[addr..])
                }
                Mbc::Two { rom_bank_reg, .. } | Mbc::Three { rom_bank_reg, .. } => {
                    let rom_bank = if *rom_bank_reg == 0 { 1 } else { *rom_bank_reg };
                    let addr = ((rom_bank as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Ok(&self.cart.data()[addr..])
                }
                Mbc::Five { rom_bank_reg, .. } => {
                    // no bank == 0 check here
                    let addr =
                        ((*rom_bank_reg as usize) << 14) + (addr - ROM_BANK_N_START) as usize;
                    Ok(&self.cart.data()[addr..])
                }
            },

            VRAM_START..SRAM_START => match self.mode {
                Mode::Gbc if self.read(VRAM_BANK_REG)? != 0 => {
                    Ok(&self.vram_cgb.as_ref().expect("is_some if cgb")
                        [(addr - VRAM_START).into()..])
                }
                _ => Ok(&self.vram[(addr - VRAM_START).into()..]),
            },

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

            ERAM_START..OAM_START => self.read_inner(addr - (ERAM_START - WRAM_BANK_0_START)),

            OAM_START..OAM_END => Ok(&self.oam[(addr - OAM_START).into()..]),

            JOYPAD_REG => Ok(as_slice(&self.joypad)),

            SERIAL_0_REG => Ok(&self.serial_transfer),
            SERIAL_1_REG => Ok(&self.serial_transfer[1..]),

            DIVIDER_REG => Ok(&self.timer_divider),
            TIMER_COUNT_REG => Ok(&self.timer_divider[1..]),
            TIMER_MOD_REG => Ok(&self.timer_divider[2..]),
            TIMER_CTRL_REG => Ok(&self.timer_divider[3..]),

            INTERRUPTS_REG => Ok(as_slice(&self.interrupts)),

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
                let spec = self.read(BG_COLOR_PALETTE_SPEC_REG)?;
                let palette = (spec & 0b00111000) as usize >> 3;
                let color = (spec & 0b00000110) as usize >> 1;
                Ok(as_slice(
                    &self.lcd.cgb_bg_palettes[palette][color][(spec % 2) as usize],
                ))
            }
            OBJ_COLOR_PALETTE_SPEC_REG => Ok(as_slice(&self.lcd.cgb_obj_palette_spec)),
            OBJ_COLOR_PALETTE_DATA_REG => {
                let spec = self.read(OBJ_COLOR_PALETTE_SPEC_REG)?;
                let palette = (spec & 0b00111000) as usize >> 3;
                let color = (spec & 0b00000110) as usize >> 1;
                Ok(as_slice(
                    &self.lcd.cgb_obj_palettes[palette][color][(spec % 2) as usize],
                ))
            }

            OBJ_PRIORITY_MODE_REG => Ok(as_slice(&self.cgb_obj_priority)),

            WRAM_BANK_REG => Ok(as_slice(&self.cgb_wram_bank)),

            HRAM_START..HRAM_END => Ok(&self.hram[(addr - HRAM_START).into()..]),

            IE_REG => Ok(as_slice(&self.ie)),

            _ => Err(Error::OutOfBounds),
        }
    }

    pub fn write_slice(&mut self, addr: u16, data: &[u8]) -> Result<(), Error> {
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
                        } else if data[0] == 0x01 && *latching == true {
                            *latching = false;
                        }
                    }
                    Mbc::Five { sram_enabled, .. } if addr < 0x2000 => {
                        *sram_enabled = data[0] & 0b00001111 == 0x0A;
                    }
                    Mbc::Five { rom_bank_reg, .. } if addr < 0x3000 => {
                        *rom_bank_reg = (*rom_bank_reg & 0xF0) + data[0] as u16;
                    }
                    Mbc::Five { rom_bank_reg, .. } if addr < 0x4000 => {
                        *rom_bank_reg = ((data[0] as u16 & 0x01) << 8) + (*rom_bank_reg & 0x0F);
                    }
                    Mbc::Five { sram_bank_reg, .. } if addr < 0x6000 => {
                        *sram_bank_reg = data[0] & 0x0F;
                    }
                    _ => {}
                }
                return Ok(());
            }

            VRAM_START..SRAM_START => match self.mode {
                Mode::Gbc if self.read(VRAM_BANK_REG)? != 0 => {
                    &mut self.vram_cgb.as_mut().expect("is_some if cgb")
                        [(addr - VRAM_START).into()..]
                }
                _ => &mut self.vram[(addr - VRAM_START).into()..],
            },

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
                return self.write_slice(addr - (ERAM_START - WRAM_BANK_0_START), data);
            }

            OAM_START..OAM_END => &mut self.oam[(addr - OAM_START).into()..],

            JOYPAD_REG => as_slice(&mut self.joypad),

            SERIAL_0_REG => &mut self.serial_transfer,
            SERIAL_1_REG => &mut self.serial_transfer[1..],

            DIVIDER_REG => &mut self.timer_divider,
            TIMER_COUNT_REG => &mut self.timer_divider[1..],
            TIMER_MOD_REG => &mut self.timer_divider[2..],
            TIMER_CTRL_REG => &mut self.timer_divider[3..],

            INTERRUPTS_REG => as_slice(&mut self.interrupts),

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
            LCD_STAT_REG => as_slice(&mut self.lcd.stat),
            SCROLL_Y_REG => as_slice(&mut self.lcd.scroll_y),
            SCROLL_X_REG => as_slice(&mut self.lcd.scroll_x),
            LY_REG => as_slice(&mut self.lcd.ly),
            LYC_REG => as_slice(&mut self.lcd.lyc),

            OAM_DMA_REG => as_slice(&mut self.oam_dma),

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

            _ => return Err(Error::OutOfBounds),
        };

        if slice.len() < data.len() {
            Err(Error::SegFault)
        } else {
            slice[..data.len()].copy_from_slice(data);
            Ok(())
        }
    }
}
