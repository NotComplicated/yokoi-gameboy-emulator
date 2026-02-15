use crate::{
    cart::Cart,
    opcode::{self, Op},
    system::Mode,
};

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

type Rgb555 = (u8, u8);

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
                            rom_bank_reg: 1,
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
                            rom_bank_reg: 1,
                            sram_4bit: [0; _],
                            sram_enabled: false,
                        };
                    }
                    crate::cart::Feature::Mbc3 => {
                        break 'mbc Mbc::Three {
                            rom_bank_reg: 1,
                            sram_bank_or_rtc_reg: 0,
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

    fn read_inner(&self, addr: u16) -> Result<&[u8], Error> {
        match addr {
            _ => Err(Error::OutOfBounds),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        self.write_inner(addr, &[data])
    }

    pub fn write_inner(&mut self, addr: u16, data: &[u8]) -> Result<(), Error> {
        for (&byte, offset) in data.iter().zip(0..) {
            match addr + offset {
                _ => return Err(Error::OutOfBounds),
            }
        }
        Ok(())
    }
}
