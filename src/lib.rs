#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate geo;
extern crate libc;
extern crate num;
#[macro_use]
extern crate failure;

mod ffi;
pub use ffi::{version, CoordSeq, GGeom, PreparedGGeom};
pub mod from_geo;
mod error;
pub use error::Error;

#[cfg(test)]
mod test;
