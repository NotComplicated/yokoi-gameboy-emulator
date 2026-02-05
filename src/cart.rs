const START_VECTOR: usize = 0x100;

pub struct Cart(Vec<u8>);

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Invalid(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errored while reading cartridge file: ")?;
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Invalid(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl Cart {
    pub fn read(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let cart = std::fs::read(path).map(Self).map_err(Error::Io)?;
        if cart.0.len() <= START_VECTOR {
            Err(Error::Invalid("Cartridge is too small"))
        } else {
            Ok(cart)
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }
}
