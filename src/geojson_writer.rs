use crate::context_handle::with_context;
use crate::functions::*;
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, Geom, PtrWrap};
use geos_sys::*;

/// The `GeoJSONWriter` type is used to generate `GeoJSON` formatted output from [`Geometry`](crate::Geometry).
///
/// # Example
///
/// ```
/// use geos::{Geometry, GeoJSONWriter};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
/// let mut writer = GeoJSONWriter::new().expect("Failed to create GeoJSONWriter");
///
/// assert_eq!(writer.write(&point_geom).unwrap(), r#"{"type":"Point","coordinates":[2.5,2.5]}"#);
/// ```
pub struct GeoJSONWriter {
    ptr: PtrWrap<*mut GEOSGeoJSONWriter>,
}

impl GeoJSONWriter {
    /// Creates a new `GeoJSONWriter` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, GeoJSONWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = GeoJSONWriter::new().expect("Failed to create GeoJSONWriter");
    ///
    /// assert_eq!(writer.write(&point_geom).unwrap(), r#"{"type":"Point","coordinates":[2.5,2.5]}"#);
    /// ```
    pub fn new() -> GResult<GeoJSONWriter> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeoJSONWriter_create_r(ctx.as_raw()))?;
            Ok(GeoJSONWriter {
                ptr: PtrWrap(ptr.as_ptr()),
            })
        })
    }

    /// Writes out the given `geometry` as GeoJSON format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, GeoJSONWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = GeoJSONWriter::new().expect("Failed to create GeoJSONWriter");
    ///
    /// assert_eq!(writer.write(&point_geom).unwrap(), r#"{"type":"Point","coordinates":[2.5,2.5]}"#);
    /// ```
    pub fn write<G: Geom>(&mut self, geometry: &G) -> GResult<String> {
        self.write_formatted(geometry, -1)
    }

    pub fn write_formatted<G: Geom>(&mut self, geometry: &G, indent: i32) -> GResult<String> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeoJSONWriter_writeGeometry_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                indent,
            ))?;
            managed_string(ptr, ctx)
        })
    }
}

unsafe impl Send for GeoJSONWriter {}
unsafe impl Sync for GeoJSONWriter {}

impl Drop for GeoJSONWriter {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSGeoJSONWriter_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

as_raw_mut_impl!(GeoJSONWriter, GEOSGeoJSONWriter);
