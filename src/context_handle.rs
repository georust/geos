use crate::GGeom;
use c_vec::CVec;
use enums::{ByteOrder, Dimensions};
use error::{Error, GResult};
use ffi::*;
use functions::*;
use libc::{c_char, c_void, strlen};
use std::cell::RefCell;
use std::ffi::CStr;
use std::slice;

pub struct GContextHandle {
    ptr: GEOSContextHandle_t,
    // TODO: maybe store the closure directly?
    notice_message: RefCell<*mut c_void>,
    // TODO: maybe store the closure directly?
    error_message: RefCell<*mut c_void>,
}

impl GContextHandle {
    /// Creates a new `GContextHandle`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// ```
    pub fn init() -> GResult<Self> {
        initialize();
        let ptr = unsafe { GEOS_init_r() };
        if ptr.is_null() {
            Err(Error::GenericError("GEOS_init_r failed".to_owned()))
        } else {
            Ok(GContextHandle {
                ptr,
                notice_message: RefCell::new(::std::ptr::null_mut()),
                error_message: RefCell::new(::std::ptr::null_mut()),
            })
        }
    }

    /// Allows to set a notice message handler.
    ///
    /// Passing [`None`] as parameter will unset this callback.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_notice_message_handler<'a>(&'a self, nf: Option<Box<dyn Fn(&str) + 'a>>) {
        let nf_data: Box<Option<Box<dyn Fn(&str) + 'a>>> = Box::new(nf);

        unsafe extern "C" fn message_handler_func<'a>(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'a>> = &*(data as *mut _);

            let bytes = slice::from_raw_parts(message as *const u8, strlen(message));
            if let Some(ref callback) = *callback {
                let s = CStr::from_bytes_with_nul_unchecked(bytes);
                callback(s.to_str().expect("invalid CStr -> &str conversion"))
            } else {
                panic!("cannot get closure...")
            }
        }

        let page_func = if nf_data.is_some() {
            Some(message_handler_func as _)
        } else {
            None
        };
        let nf_data = Box::into_raw(nf_data) as *mut _;
        let previous_ptr = self.notice_message.replace(nf_data);
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
        unsafe {
            GEOSContext_setNoticeMessageHandler_r(self.ptr, page_func, nf_data);
        }
    }

    /// Allows to set an error message handler.
    ///
    /// Passing [`None`] as parameter will unset this callback.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_error_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_error_message_handler<'a>(&'a self, ef: Option<Box<dyn Fn(&str) + 'a>>) {
        let ef_data: Box<Option<Box<dyn Fn(&str) + 'a>>> = Box::new(ef);

        unsafe extern "C" fn message_handler_func<'a>(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'a>> = &*(data as *mut _);

            let bytes = slice::from_raw_parts(message as *const u8, strlen(message));
            if let Some(ref callback) = *callback {
                let s = CStr::from_bytes_with_nul_unchecked(bytes);
                callback(s.to_str().expect("invalid CStr -> &str conversion"))
            } else {
                panic!("cannot get closure...")
            }
        }

        let page_func = if ef_data.is_some() {
            Some(message_handler_func as _)
        } else {
            None
        };
        let ef_data = Box::into_raw(ef_data) as *mut _;
        let previous_ptr = self.notice_message.replace(ef_data);
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
        unsafe {
            GEOSContext_setErrorMessageHandler_r(self.ptr, page_func, ef_data);
        }
    }

    /// Gets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, Dimensions};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(Dimensions::TwoD);
    /// assert!(context_handle.get_wkb_output_dimensions() == Dimensions::TwoD);
    /// ```
    pub fn get_wkb_output_dimensions(&self) -> Dimensions {
        Dimensions::from(unsafe { GEOS_getWKBOutputDims_r(self.ptr) })
    }

    /// Sets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, Dimensions};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(Dimensions::TwoD);
    /// assert!(context_handle.get_wkb_output_dimensions() == Dimensions::TwoD);
    /// ```
    pub fn set_wkb_output_dimensions(&self, dimensions: Dimensions) -> Dimensions {
        Dimensions::from(unsafe { GEOS_setWKBOutputDims_r(self.ptr, dimensions.into()) })
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, ByteOrder};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn get_wkb_byte_order(&self) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_getWKBByteOrder_r(self.ptr) })
    }

    /// Sets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, ByteOrder};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn set_wkb_byte_order(&self, byte_order: ByteOrder) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_setWKBByteOrder_r(self.ptr, byte_order.into()) })
    }

    /// Convert [`GGeom`] from WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_wkb_buf(&point_geom)
    ///                             .expect("conversion to WKB failed");
    /// let new_geom = context_handle.geom_from_wkb_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_from_wkb_buf(&self, wkb: &[u8]) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeomFromWKB_buf_r(self.ptr, wkb.as_ptr(), wkb.len())) }
    }

    /// Convert [`GGeom`] to WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_wkb_buf(&point_geom)
    ///                             .expect("conversion to WKB failed");
    /// let new_geom = context_handle.geom_from_wkb_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_to_wkb_buf(&self, g: &GGeom) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.ptr, g.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Convert [`GGeom`] from HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_hex_buf(&point_geom)
    ///                             .expect("conversion to HEX failed");
    /// let new_geom = context_handle.geom_from_hex_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_from_hex_buf(&self, hex: &[u8]) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeomFromHEX_buf_r(self.ptr, hex.as_ptr(), hex.len())) }
    }

    /// Convert [`GGeom`] to HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_hex_buf(&point_geom)
    ///                             .expect("conversion to HEX failed");
    /// let new_geom = context_handle.geom_from_hex_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_to_hex_buf(&self, g: &GGeom) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.ptr, g.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }
}

impl Drop for GContextHandle {
    fn drop<'a>(&'a mut self) {
        unsafe { GEOS_finish_r(self.ptr) };

        let previous_ptr = self.notice_message.replace(::std::ptr::null_mut());
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }

        let previous_ptr = self.error_message.replace(::std::ptr::null_mut());
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
    }
}
