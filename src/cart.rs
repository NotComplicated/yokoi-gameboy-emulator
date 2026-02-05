const ENTRY_POINT: usize = 0x0100;
const LOGO_START: usize = 0x0104;
const LOGO_END: usize = 0x0134;
const CHECKSUM_END: usize = 0x0150;

const LOGO_BYTES: &[u8] = &[
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub struct Cart(Vec<u8>);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Errored while reading cartridge file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid cartridge file: {0}")]
    Invalid(&'static str),
}

impl Cart {
    pub fn read(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let data = std::fs::read(path)?;
        if data.len() < CHECKSUM_END {
            Err(Error::Invalid("not enough data"))
        } else if &data[LOGO_START..LOGO_END] != LOGO_BYTES {
            Err(Error::Invalid("missing Nintendo logo"))
        } else {
            Ok(Self(data))
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }
}
