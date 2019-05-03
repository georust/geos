use enums::{ByteOrder, OutputDimension};
use crate::{ContextHandle, Geometry, GResult, AsRaw, ContextHandling, ContextInteractions};
use context_handle::PtrWrap;
use geos_sys::*;
use std::sync::Arc;
use error::Error;
use enums::TryFrom;
use c_vec::CVec;

pub struct WKBWriter<'a> {
    ptr: PtrWrap<*mut GEOSWKBWriter>,
    context: Arc<ContextHandle<'a>>,
}

impl<'a> WKBWriter<'a> {
    /// Creates a new `WKBWriter` instance.
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
    /// assert_eq!(Geometry::new_from_wkb(&v).unwrap().to_wkt_precision(1).unwrap(),
    ///            "POINT (2.5 2.5)");
    /// ```
    pub fn new() -> GResult<WKBWriter<'a>> {
        match ContextHandle::init_e(Some("WKBWriter::new")) {
            Ok(context_handle) => Self::new_with_context(Arc::new(context_handle)),
            Err(e) => Err(e),
        }
    }

    /// Creates a new `WKBWriter` instance with a given context.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandling, Geometry, WKBWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKBWriter::new_with_context(point_geom.clone_context())
    ///                            .expect("Failed to create WKBWriter");
    ///
    /// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
    /// assert_eq!(Geometry::new_from_wkb(&v).unwrap().to_wkt_precision(1).unwrap(),
    ///            "POINT (2.5 2.5)");
    /// ```
    pub fn new_with_context(context: Arc<ContextHandle<'a>>) -> GResult<WKBWriter<'a>> {
        unsafe {
            let ptr = GEOSWKBWriter_create_r(context.as_raw());
            WKBWriter::new_from_raw(ptr, context, "new_with_context")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSWKBWriter,
        context: Arc<ContextHandle<'a>>,
        caller: &str,
    ) -> GResult<WKBWriter<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr(format!("WKBWriter::{}", caller)));
        }
        Ok(WKBWriter { ptr: PtrWrap(ptr), context })
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
    pub fn write_wkb(&self, geometry: &Geometry<'_>) -> GResult<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSWKBWriter_write_r(self.get_raw_context(), self.as_raw(),
                                            geometry.as_raw(), &mut size);
            if ptr.is_null() {
                Err(Error::NoConstructionFromNullPtr(
                    "WKBWriter::write_wkb failed: GEOSWKBWriter_writeHEX_r returned null pointer".to_owned())
                )
            } else {
                Ok(CVec::new(ptr, size as _))
            }
        }
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
    pub fn write_hex(&self, geometry: &Geometry<'_>) -> GResult<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSWKBWriter_writeHEX_r(self.get_raw_context(), self.as_raw(),
                                               geometry.as_raw(), &mut size);
            if ptr.is_null() {
                Err(Error::NoConstructionFromNullPtr(
                    "WKBWriter::write_hex failed: GEOSWKBWriter_writeHEX_r returned null pointer".to_owned())
                )
            } else {
                Ok(CVec::new(ptr, size as _))
            }
        }
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
    /// assert_eq!(wkt_writer.write(&geom).unwrap(), "POINT (1.1 2.2)");
    ///
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// let v: Vec<u8> = writer.write_wkb(&point_geom).unwrap().into();
    /// let geom = Geometry::new_from_wkb(&v).unwrap();
    /// assert_eq!(wkt_writer.write(&geom).unwrap(), "POINT Z (1.1 2.2 3.3)");
    /// ```
    pub fn set_output_dimension(&mut self, dimension: OutputDimension) {
        unsafe {
            GEOSWKBWriter_setOutputDimension_r(self.get_raw_context(), self.as_raw(),
                                               dimension.into())
        }
    }

    /// Returns the number of dimensions to be used when calling [`WKBWriter::write`]. By default,
    /// it is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{OutputDimension, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new().expect("Failed to create WKBWriter");
    ///
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::TwoD));
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// ```
    pub fn get_out_dimension(&self) -> GResult<OutputDimension> {
        unsafe {
            let out = GEOSWKBWriter_getOutputDimension_r(self.get_raw_context(), self.as_raw());
            OutputDimension::try_from(out).map_err(|e| Error::GenericError(e.to_owned()))
        }
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
        unsafe {
            let out = GEOSWKBWriter_getByteOrder_r(self.get_raw_context(), self.as_raw());
            ByteOrder::try_from(out).map_err(|e| Error::GenericError(e.to_owned()))
        }
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
        unsafe {
            GEOSWKBWriter_setByteOrder_r(self.get_raw_context(), self.as_raw(), byte_order.into())
        }
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
        unsafe {
            let out = GEOSWKBWriter_getIncludeSRID_r(self.get_raw_context(), self.as_raw());
            if out < 0 {
                Err(Error::GenericError("GEOSWKBWriter_getIncludeSRID_r failed".to_owned()))
            } else {
                Ok(out != 0)
            }
        }
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
    pub fn set_include_SRID(&self, include_SRID: bool) {
        unsafe {
            GEOSWKBWriter_setIncludeSRID_r(self.get_raw_context(), self.as_raw(), include_SRID as _)
        }
    }
}

unsafe impl<'a> Send for WKBWriter<'a> {}
unsafe impl<'a> Sync for WKBWriter<'a> {}

impl<'a> Drop for WKBWriter<'a> {
    fn drop(&mut self) {
        unsafe { GEOSWKBWriter_destroy_r(self.get_raw_context(), self.as_raw()) };
    }
}

impl<'a> ContextInteractions<'a> for WKBWriter<'a> {
    /// Set the context handle to the `WKBWriter`.
    ///
    /// ```
    /// use geos::{ContextInteractions, ContextHandle, WKBWriter};
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// let mut writer = WKBWriter::new().expect("failed to create WKT writer");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// writer.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the `WKBWriter`.
    ///
    /// ```
    /// use geos::{ContextInteractions, WKBWriter};
    ///
    /// let mut writer = WKBWriter::new().expect("failed to create WKT writer");
    /// let context = writer.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &ContextHandle<'a> {
        &self.context
    }
}

impl<'a> AsRaw for WKBWriter<'a> {
    type RawType = *mut GEOSWKBWriter;

    fn as_raw(&self) -> Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for WKBWriter<'a> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
