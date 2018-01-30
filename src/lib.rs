#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate geo;
extern crate libc;
extern crate num;

mod from_geo;
mod ffi;
pub use ffi::{_point, version, CoordSeq, GGeom, PreparedGGeom, _lineString, _linearRing};

#[cfg(test)]
mod test;
