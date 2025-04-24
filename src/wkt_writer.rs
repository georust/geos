use crate::context_handle::with_context;
use crate::functions::*;
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, Geom, OutputDimension};
use geos_sys::*;
use std::convert::TryFrom;
use std::ptr::NonNull;

/// The `WKTWriter` type is used to generate `WKT` formatted output from [`Geometry`](crate::Geometry).
///
/// # Example
///
/// ```
/// use geos::{Geometry, WKTWriter};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
/// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
///
/// #[cfg(not(feature = "v3_12_0"))]
/// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
/// #[cfg(feature = "v3_12_0")]
/// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5)");
/// ```
pub struct WKTWriter {
    ptr: NonNull<GEOSWKTWriter>,
}

impl WKTWriter {
    /// Creates a new `WKTWriter` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5)");
    /// ```
    pub fn new() -> GResult<WKTWriter> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKTWriter_create_r(ctx.as_raw()))?;
            Ok(WKTWriter { ptr })
        })
    }

    /// Writes out the given `geometry` as WKT format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5)");
    /// ```
    pub fn write<G: Geom>(&mut self, geometry: &G) -> GResult<String> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKTWriter_write_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw()
            ))?;
            managed_string(ptr, ctx)
        })
    }

    /// Sets the `precision` to be used when calling [`WKTWriter::write`]. Often, what users
    /// actually want is the [`WKTWriter::set_trim`] method instead.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.543 2.567)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// writer.set_rounding_precision(1);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.6)");
    ///
    /// writer.set_rounding_precision(3);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.543 2.567)");
    /// ```
    pub fn set_rounding_precision(&mut self, precision: u32) {
        with_context(|ctx| unsafe {
            GEOSWKTWriter_setRoundingPrecision_r(ctx.as_raw(), self.as_raw_mut(), precision as _);
        })
    }

    /// Sets the number of dimensions to be used when calling [`WKTWriter::write`]. By default, it
    /// is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, OutputDimension, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.1 2.2 3.3)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    /// writer.set_trim(true);
    ///
    /// writer.set_output_dimension(OutputDimension::TwoD);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (1.1 2.2)");
    ///
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT Z (1.1 2.2 3.3)");
    /// ```
    pub fn set_output_dimension(&mut self, dimension: OutputDimension) {
        with_context(|ctx| unsafe {
            GEOSWKTWriter_setOutputDimension_r(ctx.as_raw(), self.as_raw_mut(), dimension.into());
        })
    }

    /// Returns the number of dimensions to be used when calling [`WKTWriter::write`]. By default,
    /// it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{OutputDimension, WKTWriter};
    ///
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::FourD));
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::TwoD));
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// ```
    pub fn get_out_dimension(&self) -> GResult<OutputDimension> {
        with_context(|ctx| unsafe {
            let out = errcheck!(
                -1,
                GEOSWKTWriter_getOutputDimension_r(ctx.as_raw(), self.as_raw_mut_override())
            )?;
            OutputDimension::try_from(out)
        })
    }

    /// Enables/disables trimming of unnecessary decimals.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// writer.set_trim(false);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    ///
    /// writer.set_trim(true);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5)");
    /// ```
    pub fn set_trim(&mut self, trim: bool) {
        with_context(|ctx| unsafe {
            GEOSWKTWriter_setTrim_r(ctx.as_raw(), self.as_raw_mut(), trim.into());
        })
    }

    /// Enables/disables old 3D/4D WKT style generation.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, OutputDimension, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// writer.set_trim(true);
    ///
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT Z (2.5 2.5 2.5)");
    ///
    /// writer.set_old_3D(true);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5 2.5)");
    /// ```
    #[allow(non_snake_case)]
    pub fn set_old_3D(&mut self, use_old_3D: bool) {
        with_context(|ctx| unsafe {
            GEOSWKTWriter_setOld3D_r(ctx.as_raw(), self.as_raw_mut(), use_old_3D.into());
        })
    }
}

unsafe impl Send for WKTWriter {}
unsafe impl Sync for WKTWriter {}

impl Drop for WKTWriter {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSWKTWriter_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

as_raw_mut_impl!(WKTWriter, GEOSWKTWriter);
