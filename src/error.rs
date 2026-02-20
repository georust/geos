use std::{self, fmt};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Error {
    GeosError((&'static str, Option<String>)),
    ImpossibleOperation(String),
    ConversionError(String),
    GenericError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GeosError((caller, Some(err))) => write!(f, "{caller} failed with {err}"),
            Self::GeosError((caller, None)) => write!(f, "{caller} failed"),
            Self::ImpossibleOperation(ref s) => write!(f, "impossible operation: {s}"),
            Self::ConversionError(ref s) => write!(f, "impossible to convert geometry: {s}"),
            Self::GenericError(ref s) => write!(f, "{s}"),
        }
    }
}

pub type GResult<T> = std::result::Result<T, Error>;
