use std::{cell::Cell, rc::Rc};

#[derive(Clone)]
pub struct Frame(pub Rc<[[Cell<Pixel>; 160]; 144]>);

#[derive(Copy, Clone, Debug)]
pub struct Pixel(pub u8, pub u8, pub u8);

pub enum Theme {
    Grayscale,
    Classic,
}

pub(crate) type Rgb555 = [u8; 2];

impl Pixel {
    fn white() -> Self {
        Self(0, 0, 0)
    }

    fn light() -> Self {
        Self(85, 85, 85)
    }

    fn dark() -> Self {
        Self(170, 170, 170)
    }

    fn black() -> Self {
        Self(255, 255, 255)
    }

    fn lightest_green() -> Self {
        Self(155, 188, 15)
    }

    fn light_green() -> Self {
        Self(139, 172, 15)
    }

    fn dark_green() -> Self {
        Self(48, 98, 48)
    }

    fn darkest_green() -> Self {
        Self(15, 56, 15)
    }

    pub(crate) fn from_2bit(bits: u8, theme: Theme) -> Self {
        match (bits & 0b11, theme) {
            (0b00, Theme::Grayscale) => Self::white(),
            (0b01, Theme::Grayscale) => Self::light(),
            (0b10, Theme::Grayscale) => Self::dark(),
            (0b11, Theme::Grayscale) => Self::black(),
            (0b00, Theme::Classic) => Self::lightest_green(),
            (0b01, Theme::Classic) => Self::light_green(),
            (0b10, Theme::Classic) => Self::dark_green(),
            (0b11, Theme::Classic) => Self::darkest_green(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn from_rgb555([lower, upper]: Rgb555) -> Self {
        let r = lower & 0b11111000;
        let g = ((lower & 0b00000111) << 5) | ((upper & 0b11000000) >> 3);
        let b = (upper & 0b00111110) << 2;
        Self(r, g, b)
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self(Rc::new(std::array::repeat(std::array::repeat(Cell::new(
            Pixel::white(),
        )))))
    }
}
