const START_VECTOR: usize = 0x100;

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
        if data.len() <= START_VECTOR {
            Err(Error::Invalid("not enough data"))
        } else {
            Ok(Self(data))
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }
}
