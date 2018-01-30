#![crate_name="geos"]
#![crate_type="lib"]

extern crate libc;
extern crate num;
extern crate geo;

mod ffi;
pub use ffi::{GGeom, CoordSeq, PreparedGGeom, _point, _lineString, _linearRing, version};
pub mod types_geom;

#[cfg(test)]
mod test;
#[cfg(test)]
mod conversion_test;
