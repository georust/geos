use crate::{ContextHandle, Geometry, GResult, AsRaw, ContextHandling, ContextInteractions, OutputDimension};
use context_handle::PtrWrap;
use geos_sys::*;
use functions::*;
use std::sync::Arc;
use error::Error;
use enums::TryFrom;

pub struct WKTWriter<'a> {
    ptr: PtrWrap<*mut GEOSWKTWriter>,
    context: Arc<ContextHandle<'a>>,
}

impl<'a> WKTWriter<'a> {
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// ```
    pub fn new() -> GResult<WKTWriter<'a>> {
        match ContextHandle::init_e(Some("WKTWriter::new")) {
            Ok(context_handle) => Self::new_with_context(Arc::new(context_handle)),
            Err(e) => Err(e),
        }
    }

    /// Creates a new `WKTWriter` instance with a given context.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandling, Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new_with_context(point_geom.clone_context())
    ///                            .expect("Failed to create WKTWriter");
    ///
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// ```
    pub fn new_with_context(context: Arc<ContextHandle<'a>>) -> GResult<WKTWriter<'a>> {
        unsafe {
            let ptr = GEOSWKTWriter_create_r(context.as_raw());
            WKTWriter::new_from_raw(ptr, context, "new_with_context")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSWKTWriter,
        context: Arc<ContextHandle<'a>>,
        caller: &str,
    ) -> GResult<WKTWriter<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr(format!("WKTWriter::{}", caller)));
        }
        Ok(WKTWriter { ptr: PtrWrap(ptr), context })
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// ```
    pub fn write(&self, geometry: &Geometry<'_>) -> GResult<String> {
        unsafe {
            let ptr = GEOSWKTWriter_write_r(self.get_raw_context(), self.as_raw(),
                                            geometry.as_raw());
            managed_string(ptr, self.get_context_handle(), "WKTWriter::write")
        }
    }

    /// Sets the `precision` to be used when calling [`WKTWriter::write`]. Often, what users
    /// actually want is the [`WKTWriter::set_trim`] method instead.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// writer.set_rounding_precision(2);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.50 2.50)");
    ///
    /// writer.set_rounding_precision(4);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000 2.5000)");
    /// ```
    pub fn set_rounding_precision(&mut self, precision: u32) {
        unsafe {
            GEOSWKTWriter_setRoundingPrecision_r(self.get_raw_context(), self.as_raw(),
                                                 precision as _)
        }
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (1.1 2.2)");
    ///
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT Z (1.1 2.2 3.3)");
    /// ```
    pub fn set_output_dimension(&mut self, dimension: OutputDimension) {
        unsafe {
            GEOSWKTWriter_setOutputDimension_r(self.get_raw_context(), self.as_raw(),
                                               dimension.into())
        }
    }

    /// Returns the number of dimensions to be used when calling [`WKTWriter::write`]. By default, it
    /// is 2.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{OutputDimension, WKTWriter};
    ///
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    ///
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::TwoD));
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// ```
    pub fn get_out_dimension(&self) -> GResult<OutputDimension> {
        unsafe {
            let out = GEOSWKTWriter_getOutputDimension_r(self.get_raw_context(), self.as_raw());
            OutputDimension::try_from(out).map_err(|e| Error::GenericError(e.to_owned()))
        }
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    ///
    /// writer.set_trim(true);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5 2.5)");
    /// ```
    pub fn set_trim(&mut self, trim: bool) {
        unsafe {
            GEOSWKTWriter_setTrim_r(self.get_raw_context(), self.as_raw(), trim as _)
        }
    }

    /// Enables/disables 3D/4D WKT style generation.
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
        unsafe {
            GEOSWKTWriter_setOld3D_r(self.get_raw_context(), self.as_raw(), use_old_3D as _)
        }
    }
}

unsafe impl<'a> Send for WKTWriter<'a> {}
unsafe impl<'a> Sync for WKTWriter<'a> {}

impl<'a> Drop for WKTWriter<'a> {
    fn drop(&mut self) {
        unsafe { GEOSWKTWriter_destroy_r(self.get_raw_context(), self.as_raw()) };
    }
}

impl<'a> ContextInteractions<'a> for WKTWriter<'a> {
    /// Set the context handle to the `WKTWriter`.
    ///
    /// ```
    /// use geos::{ContextInteractions, ContextHandle, WKTWriter};
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// let mut writer = WKTWriter::new().expect("failed to create WKT writer");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// writer.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the `WKTWriter`.
    ///
    /// ```
    /// use geos::{ContextInteractions, WKTWriter};
    ///
    /// let mut writer = WKTWriter::new().expect("failed to create WKT writer");
    /// let context = writer.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &ContextHandle<'a> {
        &self.context
    }
}

impl<'a> AsRaw for WKTWriter<'a> {
    type RawType = *mut GEOSWKTWriter;

    fn as_raw(&self) -> Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for WKTWriter<'a> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
