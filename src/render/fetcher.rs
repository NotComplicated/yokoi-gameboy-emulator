use crate::{
    mem::Memory,
    render::{self, Error, Pixel},
};
use serde::{Deserialize, Serialize};

pub const FETCH_STEPS: u8 = 6;

pub fn fetch_tile_pixels(memory: &Memory, addr: u16) -> Result<[Pixel; 8], Error> {
    let lo = memory.read_ppu(addr)?;
    let hi = memory.read_ppu(addr + 1)?;
    Ok(std::array::from_fn(|i| render::Pixel {
        color: ((lo >> (7 - i)) % 2) * 2 + ((hi >> (7 - i)) % 2),
        ..Default::default()
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Fetcher {
    Bg {
        tile_x: u8,
        progress: u8,
        cached: Option<[Pixel; 8]>,
        obj_queued: Option<usize>,
    },
    Window {
        tile_x: u8,
        progress: u8,
        cached: Option<[Pixel; 8]>,
        obj_queued: Option<usize>,
    },
    Object {
        tile_x: u8,
        progress: u8,
        index: usize,
    },
}

impl Fetcher {
    pub fn tile_x(&self) -> u8 {
        match self {
            Fetcher::Bg { tile_x, .. } => *tile_x,
            Fetcher::Window { tile_x, .. } => *tile_x,
            Fetcher::Object { tile_x, .. } => *tile_x,
        }
    }

    pub fn fetching_obj(&self) -> bool {
        matches!(
            self,
            Self::Object { .. }
                | Self::Bg {
                    obj_queued: Some(_),
                    ..
                }
                | Self::Window {
                    obj_queued: Some(_),
                    ..
                }
        )
    }
}
