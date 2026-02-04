pub enum Error {
    Rom(super::RomError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rom(err) => err.fmt(f),
        }
    }
}
