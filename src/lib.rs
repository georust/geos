#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate geo_types;
extern crate libc;
extern crate num;
#[macro_use]
extern crate failure;
extern crate wkt;
extern crate c_vec;

mod ffi;
pub use ffi::{
    version,
    CoordSeq,
    GContextHandle,
    GGeom,
    PreparedGGeom,
};
pub use enums::{
    ByteOrder,
    Dimensions,
    GGeomTypes,
};
mod error;
pub mod from_geo;
pub mod to_geo;
pub use error::Error;
mod voronoi;
pub use voronoi::compute_voronoi;
mod enums;

#[cfg(test)]
mod test;
