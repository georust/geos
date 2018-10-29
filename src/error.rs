use std::{self, fmt};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Invalid geometry, {}", _0)]
    InvalidGeometry(String),
    #[fail(display = "Impossible operation, {}", _0)]
    ImpossibleOperation(String),
    #[fail(display = "error while calling libgeos while {}", _0)]
    GeosError(String),
    #[fail(
        display = "error while calling libgeos method {} (error number = {})",
        _0,
        _1
    )]
    GeosFunctionError(PredicateType, i32),
    #[fail(display = "impossible to build a geometry from a nullptr")]
    NoConstructionFromNullPtr,
    #[fail(display = "impossible to convert geometry, {}", _0)]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
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
