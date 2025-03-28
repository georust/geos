use crate::context_handle::{with_context, PtrWrap};
use crate::error::Error;
use crate::functions::*;
use crate::{AsRaw, AsRawMut, ContextHandle, GResult, Geom};
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
            let ptr = GEOSGeoJSONWriter_create_r(ctx.as_raw());
            GeoJSONWriter::new_from_raw(ptr, ctx, "new_with_context")
        })
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSGeoJSONWriter,
        ctx: &ContextHandle,
        caller: &str,
    ) -> GResult<GeoJSONWriter> {
        if ptr.is_null() {
            let extra = if let Some(x) = ctx.get_last_error() {
                format!("\nLast error: {x}")
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "GeoJSONWriter::{caller}{extra}",
            )));
        }
        Ok(GeoJSONWriter { ptr: PtrWrap(ptr) })
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
            let ptr = GEOSGeoJSONWriter_writeGeometry_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                indent,
            );
            managed_string(ptr, ctx, "GeoJSONWriter::write")
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

impl AsRaw for GeoJSONWriter {
    type RawType = GEOSGeoJSONWriter;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl AsRawMut for GeoJSONWriter {
    type RawType = GEOSGeoJSONWriter;

    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
        *self.ptr
    }
}
