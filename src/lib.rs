#![crate_name = "geos"]
#![crate_type = "lib"]

extern crate c_vec;
extern crate libc;
extern crate num;
#[cfg(feature = "geo")]
extern crate geo_types;
#[cfg(feature = "geo")]
extern crate wkt;
extern crate geos_sys;

pub(crate) mod functions;

pub use context_handle::{
    GContextHandle,
};
pub use coord_seq::{
    CoordSeq,
};
pub use enums::{
    ByteOrder,
    CoordDimensions,
    Dimensions,
    GGeomTypes,
    Ordinate,
    Orientation,
};
pub use functions::{
    orientation_index,
    version,
};
pub use geom::{
    GGeom,
};
pub use prepared_geom::{
    PreparedGGeom,
};
mod context_handle;
mod coord_seq;
mod error;
#[cfg(feature = "geo")]
pub mod from_geo;
mod geom;
mod prepared_geom;
#[cfg(feature = "geo")]
pub mod to_geo;
pub use error::{
    Error,
    GResult,
};
#[cfg(feature = "geo")]
mod voronoi;
#[cfg(feature = "geo")]
pub use voronoi::{
    compute_voronoi,
};
mod enums;
mod traits;
pub(crate) use traits::{
    AsRaw,
    ContextHandling,
};
pub use traits::{
    ContextInteractions,
};

#[cfg(test)]
mod test;
