use crate::context_handle::with_context;
use crate::enums::{ByteOrder, OutputDimension};
use crate::functions::{errcheck, nullcheck, predicate};
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, Geom, PtrWrap};

use c_vec::CVec;
use geos_sys::*;
use std::convert::TryFrom;

/// The `WKBWriter` type is used to generate `HEX` or `WKB` formatted output from [`Geometry`](crate::Geometry).
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry, WKBWriter};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
/// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
///
/// // Output as WKB
/// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
/// assert_eq!(Geometry::new_from_wkb(&v).unwrap().to_wkt_precision(1).unwrap(),
///            "POINT (2.5 2.5)");
///
/// // Output as HEX
/// let v: Vec<u8> = writer.write_hex(&point_geom).unwrap().into();
/// assert_eq!(Geometry::new_from_hex(&v).unwrap().to_wkt_precision(1).unwrap(),
///            "POINT (2.5 2.5)");
/// ```
pub struct WKBWriter {
    ptr: PtrWrap<*mut GEOSWKBWriter>,
}

impl WKBWriter {
    /// Creates a new `WKBWriter` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
    /// assert_eq!(Geometry::new_from_wkb(&v).unwrap().to_wkt_precision(1).unwrap(),
    ///            "POINT (2.5 2.5)");
    /// ```
    pub fn new() -> GResult<WKBWriter> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_create_r(ctx.as_raw()))?;
            Ok(WKBWriter {
                ptr: PtrWrap(ptr.as_ptr()),
            })
        })
    }

    /// Writes out the given `geometry` as WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
    /// let expected = vec![1u8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 64, 0, 0, 0, 0, 0, 0, 4, 64];
    /// assert_eq!(v, expected);
    /// ```
    pub fn write_wkb<G: Geom>(&mut self, geometry: &G) -> GResult<CVec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_write_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                &mut size,
            ))?;
            Ok(CVec::new(ptr.as_ptr(), size as _))
        })
    }

    /// Writes out the given `geometry` as WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// let v: Vec<u8> = writer.write_hex(&point_geom).unwrap().into();
    /// let expected = vec![48u8,49,48,49,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,
    ///                     52,52,48,48,48,48,48,48,48,48,48,48,48,48,48,48,52,52,48];
    /// assert_eq!(v, expected);
    /// ```
    pub fn write_hex<G: Geom>(&mut self, geometry: &G) -> GResult<CVec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSWKBWriter_writeHEX_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                geometry.as_raw(),
                &mut size,
            ))?;
            Ok(CVec::new(ptr.as_ptr(), size as _))
        })
    }

    /// Sets the number of dimensions to be used when calling [`WKBWriter::write_wkb`] or
    /// [`WKBWriter::write_hex`]. By default, it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, OutputDimension, WKBWriter, WKTWriter};
    ///
    /// let mut wkt_writer = WKTWriter::new().expect("Failed to create WKTWriter");
    /// wkt_writer.set_output_dimension(OutputDimension::ThreeD);
    /// wkt_writer.set_trim(true);
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.1 2.2 3.3)").expect("Invalid geometry");
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
    /// let geom = Geometry::new_from_wkb(&v).unwrap();
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(wkt_writer.write(&geom).unwrap(), "POINT (1.1 2.2)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(wkt_writer.write(&geom).unwrap(), "POINT Z (1.1 2.2 3.3)");
    /// ```
    pub fn set_output_dimension(&mut self, dimension: OutputDimension) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setOutputDimension_r(ctx.as_raw(), self.as_raw_mut(), dimension.into());
        })
    }

    /// Returns the number of dimensions to be used when calling [`WKBWriter::write_wkb`].
    /// By default, it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{OutputDimension, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
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
            let out = GEOSWKBWriter_getOutputDimension_r(ctx.as_raw(), self.as_raw());
            OutputDimension::try_from(out)
        })
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ByteOrder, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// writer.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert_eq!(writer.get_wkb_byte_order(), Ok(ByteOrder::LittleEndian));
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
    /// use geos::{WKBWriter, ByteOrder};
    ///
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// writer.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert_eq!(writer.get_wkb_byte_order(), Ok(ByteOrder::LittleEndian));
    /// ```
    pub fn set_wkb_byte_order(&mut self, byte_order: ByteOrder) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setByteOrder_r(ctx.as_raw(), self.as_raw_mut(), byte_order.into());
        })
    }

    /// Gets if output will include SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::WKBWriter;
    ///
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// writer.set_include_SRID(true);
    /// assert_eq!(writer.get_include_SRID(), Ok(true));
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
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// writer.set_include_SRID(true);
    /// assert_eq!(writer.get_include_SRID(), Ok(true));
    /// ```
    #[allow(non_snake_case)]
    pub fn set_include_SRID(&mut self, include_SRID: bool) {
        with_context(|ctx| unsafe {
            GEOSWKBWriter_setIncludeSRID_r(ctx.as_raw(), self.as_raw_mut(), include_SRID.into());
        })
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
