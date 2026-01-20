use crate::context_handle::with_context;
use crate::functions::*;
use crate::traits::as_raw_mut_impl;
#[cfg(feature = "v3_14_0")]
use crate::OutputDimension;
use crate::{AsRaw, AsRawMut, GResult, Geom};
use geos_sys::*;
use std::ptr::NonNull;

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
    ptr: NonNull<GEOSGeoJSONWriter>,
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
            Ok(GeoJSONWriter { ptr })
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

    /// Sets the number of dimensions to be used when calling [`GeoJSONWriter::write`]. By default, it
    /// is 3.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, OutputDimension, GeoJSONWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.1 2.2 3.3)").expect("Invalid geometry");
    /// let mut writer = GeoJSONWriter::new().expect("Failed to create GeoJSONWriter");
    ///
    /// writer.set_output_dimension(OutputDimension::TwoD);
    /// assert_eq!(writer.write(&point_geom).unwrap(), r#"{"type":"Point","coordinates":[1.1,2.2]}"#);
    ///
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.write(&point_geom).unwrap(), r#"{"type":"Point","coordinates":[1.1,2.2,3.3]}"#);
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn set_output_dimension(&mut self, dimension: OutputDimension) {
        with_context(|ctx| unsafe {
            GEOSGeoJSONWriter_setOutputDimension_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                dimension.into(),
            );
        })
    }

    /// Returns the number of dimensions to be used when calling [`GeoJSONWriter::write`]. By default,
    /// it is 3.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{OutputDimension, GeoJSONWriter};
    ///
    /// let mut writer = GeoJSONWriter::new().expect("Failed to create GeoJSONWriter");
    ///
    /// #[cfg(all(feature = "v3_12_0", not(feature = "v3_14_0")))]
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::FourD));
    /// #[cfg(feature = "v3_14_0")]
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::TwoD));
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn get_out_dimension(&self) -> GResult<OutputDimension> {
        with_context(|ctx| unsafe {
            let out = errcheck!(
                -1,
                GEOSGeoJSONWriter_getOutputDimension_r(ctx.as_raw(), self.as_raw_mut_override())
            )?;
            OutputDimension::try_from(out)
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
