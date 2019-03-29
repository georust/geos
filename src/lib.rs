#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate c_vec;
extern crate geo_types;
extern crate libc;
extern crate num;
extern crate wkt;

pub(crate) mod ffi;
pub(crate) mod functions;

pub use context_handle::GContextHandle;
pub use coord_seq::CoordSeq;
pub use enums::{ByteOrder, Dimensions, GGeomTypes, Orientation};
pub use functions::{
    orientation_index,
    version,
};
pub use geom::GGeom;
pub use prepared_geom::PreparedGGeom;
mod context_handle;
mod coord_seq;
mod error;
pub mod from_geo;
mod geom;
mod prepared_geom;
pub mod to_geo;
pub use error::{Error, GResult};
mod voronoi;
pub use voronoi::compute_voronoi;
mod enums;

#[cfg(test)]
mod test;
