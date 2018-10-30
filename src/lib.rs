#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate geo_types;
extern crate libc;
extern crate num;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;
extern crate wkt;

mod ffi;
pub use ffi::{version, CoordSeq, GGeom, PreparedGGeom};
mod error;
pub mod from_geo;
pub mod to_geo;
pub use error::Error;
mod voronoi;
pub use voronoi::compute_voronoi;

#[cfg(test)]
mod test;
