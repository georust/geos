use crate::context_handle::with_context;
use crate::functions::*;
use crate::traits::as_raw_mut_impl;
#[cfg(feature = "v3_14_0")]
use crate::CoordDimensions;
use crate::{AsRaw, AsRawMut, GResult, Geom};
use geos_sys::*;
use std::ptr::NonNull;

/// The `GeoJSONWriter` type is used to generate `GeoJSON` formatted output from [`Geometry`](crate::Geometry).
///
/// # Example
///
/// ```
/// use geos::{GeoJSONWriter, Geometry};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
/// let mut writer = GeoJSONWriter::new()?;
///
/// assert_eq!(
///     writer.write(&point_geom)?,
///     r#"{"type":"Point","coordinates":[2.5,2.5]}"#
/// );
/// # Ok::<(), geos::Error>(())
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
    /// use geos::{GeoJSONWriter, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    /// let mut writer = GeoJSONWriter::new()?;
    ///
    /// assert_eq!(
    ///     writer.write(&point_geom)?,
    ///     r#"{"type":"Point","coordinates":[2.5,2.5]}"#
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn new() -> GResult<Self> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeoJSONWriter_create_r(ctx.as_raw()))?;
            Ok(Self { ptr })
        })
    }

    /// Writes out the given `geometry` as `GeoJSON` format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GeoJSONWriter, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    /// let mut writer = GeoJSONWriter::new()?;
    ///
    /// assert_eq!(
    ///     writer.write(&point_geom)?,
    ///     r#"{"type":"Point","coordinates":[2.5,2.5]}"#
    /// );
    /// # Ok::<(), geos::Error>(())
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
    /// use geos::{CoordDimensions, GeoJSONWriter, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.1 2.2 3.3)")?;
    /// let mut writer = GeoJSONWriter::new()?;
    ///
    /// writer.set_output_dimension(CoordDimensions::TwoD);
    /// assert_eq!(
    ///     writer.write(&point_geom)?,
    ///     r#"{"type":"Point","coordinates":[1.1,2.2]}"#
    /// );
    ///
    /// writer.set_output_dimension(CoordDimensions::ThreeD);
    /// assert_eq!(
    ///     writer.write(&point_geom)?,
    ///     r#"{"type":"Point","coordinates":[1.1,2.2,3.3]}"#
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn set_output_dimension(&mut self, dimension: CoordDimensions) {
        with_context(|ctx| unsafe {
            GEOSGeoJSONWriter_setOutputDimension_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                dimension.into(),
            );
        });
    }

    /// Returns the number of dimensions to be used when calling [`GeoJSONWriter::write`]. By default,
    /// it is 3.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, GeoJSONWriter};
    ///
    /// let mut writer = GeoJSONWriter::new()?;
    ///
    /// writer.set_output_dimension(CoordDimensions::TwoD);
    /// assert_eq!(writer.get_out_dimension()?, CoordDimensions::TwoD);
    ///
    /// writer.set_output_dimension(CoordDimensions::ThreeD);
    /// assert_eq!(writer.get_out_dimension()?, CoordDimensions::ThreeD);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn get_out_dimension(&self) -> GResult<CoordDimensions> {
        with_context(|ctx| unsafe {
            let out = errcheck!(
                -1,
                GEOSGeoJSONWriter_getOutputDimension_r(ctx.as_raw(), self.as_raw_mut_override())
            )?;
            CoordDimensions::try_from(out)
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
