#![crate_name = "geos"]
#![crate_type = "lib"]
#![cfg_attr(doc, doc = include_str!("../README.md"))]

extern crate c_vec;
#[cfg(any(feature = "geo", feature = "dox"))]
extern crate geo_types;
#[cfg(all(feature = "json"))]
extern crate geojson;
extern crate geos_sys;
extern crate libc;
extern crate num;
#[cfg(any(feature = "geo", feature = "dox"))]
extern crate wkt;

#[cfg(all(feature = "geo", test))]
#[macro_use]
extern crate doc_comment;

#[cfg(all(feature = "geo", test))]
doctest!("../README.md");

pub(crate) mod functions;

pub use context_handle::ContextHandle;
pub use coord_seq::CoordSeq;
#[cfg(any(feature = "v3_6_0", feature = "dox"))]
pub use enums::Precision;
pub use enums::{
    ByteOrder, CoordDimensions, Dimensions, GeometryTypes, JoinStyle, Ordinate, Orientation,
    OutputDimension,
};
#[cfg(any(feature = "v3_7_0", feature = "dox"))]
pub use functions::segment_intersection;
pub use functions::{orientation_index, version};
pub use geometry::{ConstGeometry, Geom, Geometry};
pub use prepared_geometry::PreparedGeometry;
pub use spatial_index::{STRtree, SpatialIndex};
pub use wkb_writer::WKBWriter;
pub use wkt_writer::WKTWriter;

mod context_handle;
mod coord_seq;
mod error;
#[cfg(any(feature = "geo", feature = "dox"))]
pub mod from_geo;
#[cfg(all(feature = "json"))]
pub mod from_geojson;
mod geometry;
mod prepared_geometry;
mod spatial_index;
#[cfg(any(feature = "geo", feature = "dox"))]
pub mod to_geo;
#[cfg(all(feature = "json"))]
pub mod to_geojson;
pub use error::{Error, GResult};
#[cfg(any(feature = "geo", feature = "dox"))]
mod voronoi;
#[cfg(any(feature = "geo", feature = "dox"))]
pub use voronoi::compute_voronoi;
mod enums;
mod traits;
mod wkb_writer;
mod wkt_writer;

pub(crate) use traits::{AsRaw, AsRawMut};
pub use traits::{ContextHandling, ContextInteractions};

#[cfg(test)]
mod test;
