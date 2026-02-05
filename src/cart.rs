use std::{fmt::Display, path::Path};

pub struct Cart(Vec<u8>);

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errored while reading ROM file: ")?;
        match self {
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl Cart {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        std::fs::read(path).map(Self).map_err(Error::Io)
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }
}
