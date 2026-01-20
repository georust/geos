#![crate_name = "geos"]
#![crate_type = "lib"]
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#[cfg(feature = "geo")]
pub use geo_types;
#[cfg(feature = "json")]
pub use geojson;
pub use geos_sys as sys;
#[cfg(feature = "geo")]
pub use wkt;

pub(crate) mod functions;

pub use buffer_params::{BufferParams, BufferParamsBuilder};
pub use context_handle::{ContextHandle, HandlerCallback};
pub use coord_seq::CoordSeq;
#[cfg(feature = "v3_6_0")]
pub use enums::Precision;
pub use enums::{
    ByteOrder, CapStyle, CoordDimensions, GeometryTypes, JoinStyle, Ordinate, Orientation,
    OutputDimension,
};
#[cfg(feature = "v3_7_0")]
pub use functions::segment_intersection;
pub use functions::{orientation_index, version};
#[cfg(feature = "v3_10_0")]
pub use geojson_writer::GeoJSONWriter;
pub use geometry::{ConstGeometry, Geom, Geometry};
pub use prepared_geometry::PreparedGeometry;
pub use spatial_index::{STRtree, SpatialIndex};
pub use wkb_writer::WKBWriter;
pub use wkt_writer::WKTWriter;

mod buffer_params;
mod context_handle;
mod coord_seq;
mod error;
#[cfg(feature = "geo")]
pub mod from_geo;
#[cfg(feature = "json")]
pub mod from_geojson;
mod geometry;
mod prepared_geometry;
mod spatial_index;
#[cfg(feature = "geo")]
pub mod to_geo;
#[cfg(feature = "json")]
pub mod to_geojson;
pub use error::{Error, GResult};
#[cfg(feature = "geo")]
mod voronoi;
#[cfg(feature = "geo")]
pub use voronoi::compute_voronoi;
mod enums;
#[cfg(feature = "v3_10_0")]
mod geojson_writer;
mod traits;
mod wkb_writer;
mod wkt_writer;

pub(crate) use traits::{AsRaw, AsRawMut};

#[cfg(test)]
mod test;
