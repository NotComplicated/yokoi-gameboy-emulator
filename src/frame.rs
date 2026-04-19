use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
};
use std::{cell::Cell, fmt, rc::Rc};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

#[derive(Clone)]
pub struct Frame(pub Rc<[[Cell<Pixel>; WIDTH]; HEIGHT]>);

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Pixel(pub u8, pub u8, pub u8);

#[derive(Copy, Clone, Default, Debug)]
pub enum Theme {
    #[default]
    Grayscale,
    Classic,
}

pub(crate) type Rgb555 = [u8; 2];

impl Pixel {
    fn white() -> Self {
        Self(255, 255, 255)
    }

    fn light() -> Self {
        Self(170, 170, 170)
    }

    fn dark() -> Self {
        Self(85, 85, 85)
    }

    fn black() -> Self {
        Self(0, 0, 0)
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

    #[expect(dead_code)]
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

impl Serialize for Frame {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(WIDTH * HEIGHT)?;
        for pixel in self.0.iter().flatten() {
            tuple.serialize_element(pixel)?;
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Frame {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FrameVisitor;
        impl<'de> Visitor<'de> for FrameVisitor {
            type Value = Frame;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} pixel color values", WIDTH * HEIGHT)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let frame = Frame::default();
                for pixel in frame.0.iter().flatten() {
                    pixel.set(
                        seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(WIDTH * HEIGHT, &self))?,
                    );
                }
                Ok(frame)
            }
        }
        deserializer.deserialize_tuple(WIDTH * HEIGHT, FrameVisitor)
    }
}
