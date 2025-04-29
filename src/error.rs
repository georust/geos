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
            Error::GeosError((caller, Some(err))) => write!(f, "{caller} failed with {err}"),
            Error::GeosError((caller, None)) => write!(f, "{caller} failed"),
            Error::ImpossibleOperation(ref s) => write!(f, "impossible operation: {s}"),
            Error::ConversionError(ref s) => write!(f, "impossible to convert geometry: {s}"),
            Error::GenericError(ref s) => write!(f, "{s}"),
        }
    }
}

pub type GResult<T> = std::result::Result<T, Error>;
