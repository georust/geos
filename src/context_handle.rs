use crate::enums::{ByteOrder, OutputDimension};
use crate::error::{Error, GResult};
use geos_sys::*;
use libc::{c_char, c_void, strlen};
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ops::Deref;
use std::slice;
use std::sync::Mutex;

thread_local!(
    static CONTEXT: ContextHandle = ContextHandle::init().unwrap();
);

/// Provides thread-local geos context to the function `f`.
///
/// It is an efficient and thread-safe way of providing geos context to be used in reentrant c api.
///
/// # Example
///
/// ```ignore
/// with_context(|ctx| unsafe {
///     let ptr = GEOSGeom_createEmptyPolygon_r(ctx.as_raw());
///     GEOSGeom_destroy_r(ctx.as_raw, ptr);
/// })
/// ```
pub(crate) fn with_context<R>(f: impl FnOnce(&ContextHandle) -> R) -> R {
    CONTEXT.with(f)
}

pub type HandlerCallback = Box<dyn Fn(&str) + Send + Sync>;

macro_rules! set_callbacks {
    ($c_func:ident, $kind:ident, $callback_name:ident, $last:ident) => {
        #[allow(clippy::needless_lifetimes)]
        fn $kind(ptr: GEOSContextHandle_t, nf: *mut InnerContext) {
            #[allow(clippy::extra_unused_lifetimes)]
            unsafe extern "C" fn message_handler_func(message: *const c_char, data: *mut c_void) {
                let inner_context: &InnerContext = &*(data as *mut _);

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

set_callbacks!(
    GEOSContext_setNoticeMessageHandler_r,
    set_notif,
    notif_callback,
    last_notification
);
set_callbacks!(
    GEOSContext_setErrorMessageHandler_r,
    set_error,
    error_callback,
    last_error
);

pub(crate) struct PtrWrap<T>(pub T);

impl<T> Deref for PtrWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T> Send for PtrWrap<T> {}
unsafe impl<T> Sync for PtrWrap<T> {}

pub(crate) struct InnerContext {
    last_notification: Mutex<Option<String>>,
    last_error: Mutex<Option<String>>,
    notif_callback: Mutex<HandlerCallback>,
    error_callback: Mutex<HandlerCallback>,
}

pub struct ContextHandle {
    ptr: PtrWrap<GEOSContextHandle_t>,
    pub(crate) inner: PtrWrap<*mut InnerContext>,
}

impl ContextHandle {
    /// Creates a new `ContextHandle`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// ```
    pub fn init() -> GResult<Self> {
        let ptr = unsafe { GEOS_init_r() };
        if ptr.is_null() {
            return Err(Error::GenericError("GEOS_init_r failed".to_owned()));
        }

        let last_notification = Mutex::new(None);
        let last_error = Mutex::new(None);

        let notif_callback: Mutex<HandlerCallback> = Mutex::new(Box::new(|_| {}));
        let error_callback: Mutex<HandlerCallback> = Mutex::new(Box::new(|_| {}));

        let inner = Box::into_raw(Box::new(InnerContext {
            last_notification,
            last_error,
            notif_callback,
            error_callback,
        }));

        set_notif(ptr, inner);
        set_error(ptr, inner);

        Ok(ContextHandle {
            ptr: PtrWrap(ptr),
            inner: PtrWrap(inner),
        })
    }

    pub(crate) fn as_raw(&self) -> GEOSContextHandle_t {
        *self.ptr
    }

    pub(crate) fn get_inner(&self) -> &InnerContext {
        unsafe { &*self.inner.0 }
    }

    /// Allows to set a notice message handler.
    ///
    /// Passing [`None`] as parameter will unset this callback.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_notice_message_handler(&self, nf: Option<HandlerCallback>) {
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
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_error_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_error_message_handler(&self, ef: Option<HandlerCallback>) {
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
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
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
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
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
    /// use geos::{ContextHandle, OutputDimension};
    ///
    /// let mut context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(OutputDimension::TwoD);
    /// assert_eq!(context_handle.get_wkb_output_dimensions(), Ok(OutputDimension::TwoD));
    /// ```
    pub fn get_wkb_output_dimensions(&self) -> GResult<OutputDimension> {
        unsafe {
            let out = GEOS_getWKBOutputDims_r(self.as_raw());
            OutputDimension::try_from(out)
        }
    }

    /// Sets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandle, OutputDimension};
    ///
    /// let mut context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(OutputDimension::TwoD);
    /// assert_eq!(context_handle.get_wkb_output_dimensions(), Ok(OutputDimension::TwoD));
    /// ```
    pub fn set_wkb_output_dimensions(
        &mut self,
        dimensions: OutputDimension,
    ) -> GResult<OutputDimension> {
        unsafe {
            let out = GEOS_setWKBOutputDims_r(self.as_raw(), dimensions.into());
            OutputDimension::try_from(out)
        }
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandle, ByteOrder};
    ///
    /// let mut context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn get_wkb_byte_order(&self) -> ByteOrder {
        ByteOrder::try_from(unsafe { GEOS_getWKBByteOrder_r(self.as_raw()) })
            .expect("failed to convert to ByteOrder")
    }

    /// Sets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandle, ByteOrder};
    ///
    /// let mut context_handle = ContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn set_wkb_byte_order(&mut self, byte_order: ByteOrder) -> ByteOrder {
        ByteOrder::try_from(unsafe { GEOS_setWKBByteOrder_r(self.as_raw(), byte_order.into()) })
            .expect("failed to convert to ByteOrder")
    }
}

impl Drop for ContextHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                GEOS_finish_r(self.as_raw());
            }
            // Now we just have to clear stuff!
            let _inner: Box<InnerContext> = Box::from_raw(self.inner.0);
        }
    }
}
