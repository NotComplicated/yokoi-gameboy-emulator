#[derive(Clone)]
pub struct Frame(pub [[Pixel; 160]; 144]);

#[derive(Copy, Clone, Debug)]
pub struct Pixel(u8, u8, u8);

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

    fn from_rgb555([lower, upper]: Rgb555) -> Self {
        let r = lower & 0b11111000;
        let g = ((lower & 0b00000111) << 5) | ((upper & 0b11000000) >> 3);
        let b = (upper & 0b00111110) << 2;
        Self(r, g, b)
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self([[Pixel::white(); _]; _])
    }
}
