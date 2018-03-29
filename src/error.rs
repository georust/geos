use std;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Invalid geometry, {}", _0)]
    InvalidGeometry(String),
    #[fail(display = "impossible to build a geometry from a nullptr")]
    NoConstructionFromNullPtr,
}

pub type Result<T> = std::result::Result<T, Error>;
