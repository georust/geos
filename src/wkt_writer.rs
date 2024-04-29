use crate::context_handle::PtrWrap;
use crate::error::Error;
use crate::functions::*;
use crate::{
    AsRaw, AsRawMut, ContextHandle, ContextHandling, ContextInteractions, GResult, Geom,
    OutputDimension,
};
use geos_sys::*;
use std::convert::TryFrom;
use std::sync::Arc;

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
/// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
/// ```
pub struct WKTWriter {
    ptr: PtrWrap<*mut GEOSWKTWriter>,
    context: Arc<ContextHandle>,
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// ```
    pub fn new() -> GResult<WKTWriter> {
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
    pub fn new_with_context(context: Arc<ContextHandle>) -> GResult<WKTWriter> {
        unsafe {
            let ptr = GEOSWKTWriter_create_r(context.as_raw());
            WKTWriter::new_from_raw(ptr, context, "new_with_context")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSWKTWriter,
        context: Arc<ContextHandle>,
        caller: &str,
    ) -> GResult<WKTWriter> {
        if ptr.is_null() {
            let extra = if let Some(x) = context.get_last_error() {
                format!("\nLast error: {x}")
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "WKTWriter::{caller}{extra}",
            )));
        }
        Ok(WKTWriter {
            ptr: PtrWrap(ptr),
            context,
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
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    /// ```
    pub fn write<'b, G: Geom>(&mut self, geometry: &G) -> GResult<String> {
        unsafe {
            let ptr =
                GEOSWKTWriter_write_r(self.get_raw_context(), self.as_raw_mut(), geometry.as_raw());
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
            GEOSWKTWriter_setRoundingPrecision_r(
                self.get_raw_context(),
                self.as_raw_mut(),
                precision as _,
            )
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
            GEOSWKTWriter_setOutputDimension_r(
                self.get_raw_context(),
                self.as_raw_mut(),
                dimension.into(),
            )
        }
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
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::TwoD));
    /// writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(writer.get_out_dimension(), Ok(OutputDimension::ThreeD));
    /// ```
    pub fn get_out_dimension(&self) -> GResult<OutputDimension> {
        unsafe {
            let out = GEOSWKTWriter_getOutputDimension_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
            );
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
        unsafe { GEOSWKTWriter_setTrim_r(self.get_raw_context(), self.as_raw_mut(), trim as _) }
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
        unsafe {
            GEOSWKTWriter_setOld3D_r(self.get_raw_context(), self.as_raw_mut(), use_old_3D as _)
        }
    }
}

unsafe impl Send for WKTWriter {}
unsafe impl Sync for WKTWriter {}

impl Drop for WKTWriter {
    fn drop(&mut self) {
        unsafe { GEOSWKTWriter_destroy_r(self.get_raw_context(), self.as_raw_mut()) };
    }
}

impl ContextInteractions for WKTWriter {
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
    fn set_context_handle(&mut self, context: ContextHandle) {
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
    fn get_context_handle(&self) -> &ContextHandle {
        &self.context
    }
}

impl AsRaw for WKTWriter {
    type RawType = GEOSWKTWriter;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl AsRawMut for WKTWriter {
    type RawType = GEOSWKTWriter;

    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
        *self.ptr
    }
}

impl ContextHandling for WKTWriter {
    type Context = Arc<ContextHandle>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle> {
        Arc::clone(&self.context)
    }
}
