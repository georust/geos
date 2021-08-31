use std::{self, fmt};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Error {
    InvalidGeometry(String),
    ImpossibleOperation(String),
    GeosError(String),
    GeosFunctionError(PredicateType, i32),
    NoConstructionFromNullPtr(String),
    ConversionError(String),
    GenericError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidGeometry(ref s) => write!(f, "Invalid geometry, {}", s),
            Error::ImpossibleOperation(ref s) => write!(f, "Impossible operation, {}", s),
            Error::GeosError(ref s) => write!(f, "error while calling libgeos while {}", s),
            Error::GeosFunctionError(p, e) => write!(
                f,
                "error while calling libgeos method {} (error number = {})",
                p, e
            ),
            Error::NoConstructionFromNullPtr(ref s) => write!(
                f,
                "impossible to build a geometry from a nullptr in \"{}\"",
                s
            ),
            Error::ConversionError(ref s) => write!(f, "impossible to convert geometry, {}", s),
            Error::GenericError(ref s) => write!(f, "generic error: {}", s),
        }
    }
}

pub type GResult<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum PredicateType {
    Intersects,
    Crosses,
    Disjoint,
    Touches,
    Overlaps,
    Within,
    Equals,
    EqualsExact,
    Covers,
    CoveredBy,
    Contains,
    IsRing,
    IsEmpty,
    IsSimple,
    PreparedContains,
    PreparedContainsProperly,
    PreparedCoveredBy,
    PreparedCovers,
    PreparedCrosses,
    PreparedDisjoint,
    PreparedIntersects,
    PreparedOverlaps,
    PreparedTouches,
    PreparedWithin,
    Normalize,
}

impl std::fmt::Display for PredicateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}
