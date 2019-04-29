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

#[cfg(all(feature = "geo", test))]
#[macro_use]
extern crate doc_comment;

#[cfg(all(feature = "geo", test))]
doctest!("../README.md");

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
#[cfg(feature = "v3_6_0")]
pub use enums::{
    Precision,
};
pub use functions::{
    orientation_index,
    version,
};
#[cfg(feature = "v3_7_0")]
pub use functions::{
    segment_intersection,
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
