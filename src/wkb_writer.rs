use crate::context_handle::with_context;
use crate::enums::{ByteOrder, CoordDimensions};
use crate::functions::{errcheck, managed_vec, nullcheck, predicate};
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, Geom};

use geos_sys::*;
use std::convert::TryFrom;
use std::ptr::NonNull;

/// The `WKBWriter` type is used to generate `HEX` or `WKB` formatted output from [`Geometry`](crate::Geometry).
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry, WKBWriter};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
/// let mut writer = WKBWriter::new()?;
///
/// // Output as WKB
/// let v: Vec<u8> = writer.write_wkb(&point_geom)?.into();
/// assert_eq!(Geometry::new_from_wkb(&v)?.to_wkt()?, "POINT (2.5 2.5)");
///
/// // Output as HEX
/// let v: Vec<u8> = writer.write_hex(&point_geom)?.into();
/// assert_eq!(Geometry::new_from_hex(&v)?.to_wkt()?, "POINT (2.5 2.5)");
/// # Ok::<(), geos::Error>(())
/// ```
pub struct WKBWriter {
    ptr: NonNull<GEOSWKBWriter>,
}

impl WKBWriter {
    /// Creates a new `WKBWriter` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    /// let mut writer = WKBWriter::new()?;
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom)?.into();
    /// assert_eq!(Geometry::new_from_wkb(&v)?.to_wkt()?, "POINT (2.5 2.5)");
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(not(feature = "tests"))]
    pub fn new() -> GResult<Self> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_create_r(ctx.as_raw()))?;
            Ok(Self { ptr })
        })
    }
    #[cfg(feature = "tests")]
    pub fn new() -> GResult<WKBWriter> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_create_r(ctx.as_raw()))?;
            #[cfg(not(feature = "v3_12_0"))]
            GEOSWKBWriter_setOutputDimension_r(ctx.as_raw(), ptr.as_ptr(), 3);
            #[cfg(feature = "v3_12_0")]
            GEOSWKBWriter_setOutputDimension_r(ctx.as_raw(), ptr.as_ptr(), 4);
            Ok(WKBWriter { ptr })
        })
    }

    /// Writes out the given `geometry` as WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    /// let mut writer = WKBWriter::new()?;
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom)?.into();
    /// let expected = vec![
    ///     1u8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 64, 0, 0, 0, 0, 0, 0, 4, 64,
    /// ];
    /// assert_eq!(v, expected);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn write_wkb<G: Geom>(&mut self, geometry: &G) -> GResult<Vec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_write_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                &mut size,
            ))?;
            Ok(managed_vec(ptr, size, ctx))
        })
    }

    /// Writes out the given `geometry` as WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    /// let mut writer = WKBWriter::new()?;
    ///
    /// let v: Vec<u8> = writer.write_hex(&point_geom)?.into();
    /// let expected = vec![
    ///     48u8, 49, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
    ///     48, 52, 52, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 52, 52, 48,
    /// ];
    /// assert_eq!(v, expected);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn write_hex<G: Geom>(&mut self, geometry: &G) -> GResult<Vec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_writeHEX_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                &mut size,
            ))?;
            Ok(managed_vec(ptr, size, ctx))
        })
    }

    /// Sets the number of dimensions to be used when calling [`WKBWriter::write_wkb`] or
    /// [`WKBWriter::write_hex`]. By default, it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, Geom, Geometry, WKBWriter, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT Z (1.1 2.2 3.3)")?;
    /// let mut writer = WKBWriter::new()?;
    /// writer.set_output_dimension(CoordDimensions::TwoD);
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom)?.into();
    /// let geom = Geometry::new_from_wkb(&v)?;
    /// assert_eq!(geom.get_coordinate_dimension()?, CoordDimensions::TwoD);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_output_dimension(&mut self, dimension: CoordDimensions) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setOutputDimension_r(ctx.as_raw(), self.as_raw_mut(), dimension.into());
        });
    }

    /// Returns the number of dimensions to be used when calling [`WKBWriter::write_wkb`].
    /// By default, it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new()?;
    ///
    /// writer.set_output_dimension(CoordDimensions::TwoD);
    /// assert_eq!(writer.get_out_dimension()?, CoordDimensions::TwoD);
    ///
    /// writer.set_output_dimension(CoordDimensions::ThreeD);
    /// assert_eq!(writer.get_out_dimension()?, CoordDimensions::ThreeD);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_out_dimension(&self) -> GResult<CoordDimensions> {
        with_context(|ctx| unsafe {
            let out = GEOSWKBWriter_getOutputDimension_r(ctx.as_raw(), self.as_raw());
            CoordDimensions::try_from(out)
        })
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ByteOrder, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new()?;
    ///
    /// writer.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert_eq!(writer.get_wkb_byte_order()?, ByteOrder::LittleEndian);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_wkb_byte_order(&self) -> GResult<ByteOrder> {
        with_context(|ctx| unsafe {
            let out = GEOSWKBWriter_getByteOrder_r(ctx.as_raw(), self.as_raw());
            ByteOrder::try_from(out)
        })
    }

    /// Sets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ByteOrder, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new()?;
    ///
    /// writer.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert_eq!(writer.get_wkb_byte_order()?, ByteOrder::LittleEndian);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_wkb_byte_order(&mut self, byte_order: ByteOrder) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setByteOrder_r(ctx.as_raw(), self.as_raw_mut(), byte_order.into());
        });
    }

    /// Gets if output will include SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::WKBWriter;
    ///
    /// let mut writer = WKBWriter::new()?;
    ///
    /// writer.set_include_SRID(true);
    /// assert_eq!(writer.get_include_SRID()?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[allow(non_snake_case)]
    pub fn get_include_SRID(&self) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSWKBWriter_getIncludeSRID_r(ctx.as_raw(), self.as_raw()))
        })
    }

    /// Sets if output will include SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::WKBWriter;
    ///
    /// let mut writer = WKBWriter::new()?;
    ///
    /// writer.set_include_SRID(true);
    /// assert_eq!(writer.get_include_SRID()?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[allow(non_snake_case)]
    pub fn set_include_SRID(&mut self, include_SRID: bool) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setIncludeSRID_r(ctx.as_raw(), self.as_raw_mut(), include_SRID.into());
        });
    }
}

unsafe impl Send for WKBWriter {}
unsafe impl Sync for WKBWriter {}

impl Drop for WKBWriter {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSWKBWriter_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

as_raw_mut_impl!(WKBWriter, GEOSWKBWriter);
