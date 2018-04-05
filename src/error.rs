use std;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Invalid geometry, {}", _0)]
    InvalidGeometry(String),
    #[fail(display = "Impossible operation, {}", _0)]
    ImpossibleOperation(String),
    #[fail(display = "error while calling libgeos while {}", _0)]
    GeosError(String),
    #[fail(display = "impossible to build a geometry from a nullptr")]
    NoConstructionFromNullPtr,
}

pub type Result<T> = std::result::Result<T, Error>;
