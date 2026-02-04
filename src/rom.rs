use std::{fmt::Display, path::Path};

pub struct Rom {
    data: Box<[u8]>,
}

pub enum RomError {
    Io(std::io::Error),
}

impl Display for RomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errored while reading ROM file: ")?;
        match self {
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl Rom {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, super::Error> {
        let rom = std::fs::read(path)
            .map(|data| Self { data: data.into() })
            .map_err(RomError::Io)
            .map_err(super::Error::Rom)?;

        Ok(rom)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
