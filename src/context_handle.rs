use crate::enums::{ByteOrder, CoordDimensions};
use crate::error::{Error, GResult};
use crate::functions::errcheck;
use geos_sys::*;
use libc::{c_char, c_void, strlen};
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ptr::NonNull;
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
pub fn with_context<R>(f: impl FnOnce(&ContextHandle) -> R) -> R {
    CONTEXT.with(f)
}

pub type HandlerCallback = Box<dyn Fn(&str) + Send + Sync>;

unsafe extern "C" fn message_handler(message: *const c_char, data: *mut c_void) {
    let inner_context: &InnerContext = &*(data.cast());

    if let Ok(callback) = inner_context.callback.lock() {
        let bytes = slice::from_raw_parts(message.cast::<u8>(), strlen(message) + 1);
        let s = CStr::from_bytes_with_nul_unchecked(bytes);
        let notif = s.to_str().expect("invalid CStr -> &str conversion");
        callback(notif);
        if let Ok(mut last) = inner_context.last.lock() {
            *last = Some(notif.to_owned());
        }
    }
}

pub struct InnerContext {
    last: Mutex<Option<String>>,
    callback: Mutex<HandlerCallback>,
}

impl InnerContext {
    fn take(&self) -> Option<String> {
        self.last.lock().map(|mut last| last.take()).unwrap_or(None)
    }
}

pub struct ContextHandle {
    ptr: NonNull<GEOSContextHandle_HS>,
    pub(crate) notice_ctx: NonNull<InnerContext>,
    pub(crate) error_ctx: NonNull<InnerContext>,
}

impl ContextHandle {
    /// Creates a new `ContextHandle`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init()?;
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn init() -> GResult<Self> {
        let ptr = unsafe { GEOS_init_r() };
        let ptr = NonNull::new(ptr).ok_or(Error::GeosError(("GEOS_init_r", None)))?;

        let error_ctx = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(InnerContext {
                last: Mutex::new(None),
                callback: Mutex::new(Box::new(|_| {})),
            })))
        };

        let notice_ctx = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(InnerContext {
                last: Mutex::new(None),
                callback: Mutex::new(Box::new(|_| {})),
            })))
        };

        unsafe {
            GEOSContext_setNoticeMessageHandler_r(
                ptr.as_ptr(),
                Some(message_handler),
                notice_ctx.as_ptr().cast(),
            );
            GEOSContext_setErrorMessageHandler_r(
                ptr.as_ptr(),
                Some(message_handler),
                error_ctx.as_ptr().cast(),
            );
        }

        Ok(Self {
            ptr,
            notice_ctx,
            error_ctx,
        })
    }

    pub(crate) const fn as_raw(&self) -> GEOSContextHandle_t {
        self.ptr.as_ptr()
    }

    pub(crate) fn get_notice_context(&self) -> &InnerContext {
        unsafe { self.notice_ctx.as_ref() }
    }

    pub(crate) fn get_error_context(&self) -> &InnerContext {
        unsafe { self.error_ctx.as_ref() }
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
    /// let context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_notice_message_handler(&self, nf: Option<HandlerCallback>) {
        if let Ok(mut callback) = self.get_notice_context().callback.lock() {
            *callback = nf.unwrap_or_else(|| Box::new(|_| {}));
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
    /// let context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_error_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_error_message_handler(&self, ef: Option<HandlerCallback>) {
        if let Ok(mut callback) = self.get_error_context().callback.lock() {
            *callback = ef.unwrap_or_else(|| Box::new(|_| {}));
        }
    }

    /// Returns the last error encountered.
    ///
    /// Please note that calling this function will remove the current last error!
    ///
    /// ```
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init()?;
    /// // make some functions calls...
    /// if let Some(last_error) = context_handle.get_last_error() {
    ///     println!("We have an error: {}", last_error);
    /// } else {
    ///     println!("No error occurred!");
    /// }
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_last_error(&self) -> Option<String> {
        self.get_error_context().take()
    }

    /// Returns the last notification encountered.
    ///
    /// Please note that calling this function will remove the current last notification!
    ///
    /// ```
    /// use geos::ContextHandle;
    ///
    /// let context_handle = ContextHandle::init()?;
    /// // make some functions calls...
    /// if let Some(last_notif) = context_handle.get_last_notification() {
    ///     println!("We have a notification: {}", last_notif);
    /// } else {
    ///     println!("No notifications!");
    /// }
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_last_notification(&self) -> Option<String> {
        self.get_notice_context().take()
    }

    /// Gets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandle, CoordDimensions};
    ///
    /// let mut context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_wkb_output_dimensions(CoordDimensions::TwoD);
    /// assert_eq!(
    ///     context_handle.get_wkb_output_dimensions()?,
    ///     CoordDimensions::TwoD
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_wkb_output_dimensions(&self) -> GResult<CoordDimensions> {
        unsafe {
            let out = errcheck!(-1, GEOS_getWKBOutputDims_r(self.as_raw()))?;
            CoordDimensions::try_from(out)
        }
    }

    /// Sets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ContextHandle, CoordDimensions};
    ///
    /// let mut context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_wkb_output_dimensions(CoordDimensions::TwoD)?;
    /// assert_eq!(
    ///     context_handle.get_wkb_output_dimensions()?,
    ///     CoordDimensions::TwoD
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_wkb_output_dimensions(
        &mut self,
        dimensions: CoordDimensions,
    ) -> GResult<CoordDimensions> {
        unsafe {
            let out = errcheck!(
                -1,
                GEOS_setWKBOutputDims_r(self.as_raw(), dimensions.into())
            )?;
            CoordDimensions::try_from(out)
        }
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ByteOrder, ContextHandle};
    ///
    /// let mut context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian)?;
    /// assert_eq!(
    ///     context_handle.get_wkb_byte_order()?,
    ///     ByteOrder::LittleEndian
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_wkb_byte_order(&self) -> GResult<ByteOrder> {
        let out = unsafe { errcheck!(-1, GEOS_getWKBByteOrder_r(self.as_raw()))? };
        ByteOrder::try_from(out)
    }

    /// Sets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{ByteOrder, ContextHandle};
    ///
    /// let mut context_handle = ContextHandle::init()?;
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian)?;
    /// assert_eq!(
    ///     context_handle.get_wkb_byte_order()?,
    ///     ByteOrder::LittleEndian
    /// );
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_wkb_byte_order(&mut self, byte_order: ByteOrder) -> GResult<ByteOrder> {
        let out =
            unsafe { errcheck!(-1, GEOS_setWKBByteOrder_r(self.as_raw(), byte_order.into()))? };
        ByteOrder::try_from(out)
    }
}

impl Drop for ContextHandle {
    fn drop(&mut self) {
        unsafe {
            GEOS_finish_r(self.as_raw());
            // Now we just have to clear stuff!
            let _ = Box::from_raw(self.error_ctx.as_ptr());
            let _ = Box::from_raw(self.notice_ctx.as_ptr());
        }
    }
}
