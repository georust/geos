use crate::GGeom;
use c_vec::CVec;
use enums::{ByteOrder, Dimensions};
use error::{Error, GResult};
use ffi::*;
use functions::*;
use libc::{c_char, c_void, strlen};
use std::ffi::CStr;
use std::slice;
use std::sync::Mutex;
use std::ops::Deref;

macro_rules! set_callbacks {
    ($c_func:ident, $kind:ident, $callback_name:ident, $last:ident) => {
        fn $kind<'a>(ptr: GEOSContextHandle_t, nf: *mut InnerContext<'a>) {
            unsafe extern "C" fn message_handler_func<'a>(message: *const c_char, data: *mut c_void) {
                let inner_context: &InnerContext<'a> = &*(data as *mut _);

                if let Ok(callback) = inner_context.$callback_name.lock() {
                    let bytes = slice::from_raw_parts(message as *const u8, strlen(message));
                    let s = CStr::from_bytes_with_nul_unchecked(bytes);
                    let notif = s.to_str().expect("invalid CStr -> &str conversion");
                    callback(notif);
                    if let Ok(mut last) = inner_context.$last.lock() {
                        *last = Some(notif.to_owned());
                    }
                }
            }

            unsafe {
                $c_func(ptr, Some(message_handler_func), nf as *mut _);
            }
        }
    };
}

set_callbacks!(GEOSContext_setNoticeMessageHandler_r, set_notif, notif_callback, last_notification);
set_callbacks!(GEOSContext_setErrorMessageHandler_r, set_error, error_callback, last_error);

struct PtrWrap<T>(T);

impl<T> Deref for PtrWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T> Send for PtrWrap<T> {}
unsafe impl<T> Sync for PtrWrap<T> {}

pub(crate) struct InnerContext<'a> {
    last_notification: Mutex<Option<String>>,
    last_error: Mutex<Option<String>>,
    notif_callback: Mutex<Box<dyn Fn(&str) + Send + Sync + 'a>>,
    error_callback: Mutex<Box<dyn Fn(&str) + Send + Sync + 'a>>,
}

pub struct GContextHandle<'a> {
    ptr: PtrWrap<GEOSContextHandle_t>,
    inner: PtrWrap<*mut InnerContext<'a>>,
}

impl<'a> GContextHandle<'a> {
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
            return Err(Error::GenericError("GEOS_init_r failed".to_owned()));
        }
        let last_notification = Mutex::new(None);
        let last_error = Mutex::new(None);

        let notif_callback: Mutex<Box<dyn Fn(&str) + Send + Sync + 'a>> = Mutex::new(Box::new(|_| {}));
        let error_callback: Mutex<Box<dyn Fn(&str) + Send + Sync + 'a>> = Mutex::new(Box::new(|_| {}));

        let inner = Box::into_raw(Box::new(InnerContext {
            notif_callback,
            error_callback,
            last_notification,
            last_error,
        }));

        set_notif(ptr, inner);
        set_error(ptr, inner);

        Ok(GContextHandle {
            ptr: PtrWrap(ptr),
            inner: PtrWrap(inner),
        })
    }

    pub(crate) fn as_raw(&self) -> GEOSContextHandle_t {
        *self.ptr
    }

    pub(crate) fn get_inner(&self) -> &InnerContext<'a> {
        unsafe { &*self.inner.0 }
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
    pub fn set_notice_message_handler(&self, nf: Option<Box<dyn Fn(&str) + Send + Sync + 'a>>) {
        let inner_context = self.get_inner();
        if let Ok(mut callback) = inner_context.notif_callback.lock() {
            if let Some(nf) = nf {
                *callback = nf;
            } else {
                *callback = Box::new(|_| {});
            }
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
    pub fn set_error_message_handler(&self, ef: Option<Box<dyn Fn(&str) + Send + Sync + 'a>>) {
        let inner_context = self.get_inner();
        if let Ok(mut callback) = inner_context.error_callback.lock() {
            if let Some(ef) = ef {
                *callback = ef;
            } else {
                *callback = Box::new(|_| {});
            }
        }
    }

    /// Returns the last error encountered.
    ///
    /// Please note that calling this function will remove the current last error!
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// // make some functions calls...
    /// if let Some(last_error) = context_handle.get_last_error() {
    ///     println!("We have an error: {}", last_error);
    /// } else {
    ///     println!("No error occurred!");
    /// }
    /// ```
    pub fn get_last_error(&self) -> Option<String> {
        let inner_context = self.get_inner();
        if let Ok(mut last) = inner_context.last_error.lock() {
            last.take()
        } else {
            None
        }
    }

    /// Returns the last notification encountered.
    ///
    /// Please note that calling this function will remove the current last notification!
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// // make some functions calls...
    /// if let Some(last_notif) = context_handle.get_last_notification() {
    ///     println!("We have a notification: {}", last_notif);
    /// } else {
    ///     println!("No notifications!");
    /// }
    /// ```
    pub fn get_last_notification(&self) -> Option<String> {
        let inner_context = self.get_inner();
        if let Ok(mut last) = inner_context.last_notification.lock() {
            last.take()
        } else {
            None
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
        Dimensions::from(unsafe { GEOS_getWKBOutputDims_r(self.as_raw()) })
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
        Dimensions::from(unsafe { GEOS_setWKBOutputDims_r(self.as_raw(), dimensions.into()) })
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
        ByteOrder::from(unsafe { GEOS_getWKBByteOrder_r(self.as_raw()) })
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
        ByteOrder::from(unsafe { GEOS_setWKBByteOrder_r(self.as_raw(), byte_order.into()) })
    }
}

impl<'a> Drop for GContextHandle<'a> {
    fn drop(&mut self) {
        unsafe {
            GEOS_finish_r(self.as_raw());
            // Now we just have to clear stuff!
            let _inner: Box<InnerContext<'a>> = Box::from_raw(self.inner.0);
        }
    }
}
