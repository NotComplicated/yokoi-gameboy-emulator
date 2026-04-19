mod fetcher;
pub mod ppu;

use crate::mem;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    Memory(mem::Error),
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Self {
        Self::Memory(err)
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug)]
struct OamBuf {
    buffer: [Object; 10],
    len: usize,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug)]
struct Object {
    y: u8,
    x: u8,
    tile: u8,
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette: u8,
    bank: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Fifo {
    buffer: [Pixel; 16],
    len: usize,
    front: usize,
    back: usize,
}

impl Fifo {
    fn new() -> Self {
        Self {
            buffer: Default::default(),
            len: 0,
            front: 0,
            back: 0,
        }
    }

    fn push_8(&mut self, pixels: [Pixel; 8]) -> Result<(), ()> {
        if self.len + 8 > self.buffer.len() {
            Err(())
        } else {
            for pixel in pixels {
                self.buffer[self.back] = pixel;
                self.back = (self.back + 1) % self.buffer.len();
            }
            self.len += 8;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<Pixel> {
        if self.len == 0 {
            None
        } else {
            let pixel = self.buffer[self.front];
            self.front = (self.front + 1) % self.buffer.len();
            self.len -= 1;
            Some(pixel)
        }
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug)]
struct Pixel {
    color: u8,
    palette: u8,
    priority: u8,
    from_obj: bool,
}
