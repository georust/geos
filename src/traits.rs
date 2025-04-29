use std::ops::Deref;

pub(crate) struct PtrWrap<T>(pub T);

impl<T> Deref for PtrWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait AsRaw {
    type RawType;

    fn as_raw(&self) -> *const Self::RawType;
}

pub trait AsRawMut: AsRaw {
    fn as_raw_mut(&mut self) -> *mut Self::RawType {
        unsafe { self.as_raw_mut_override() }
    }

    /// This method exists because in certain case, even though you don't run any mutable operation
    /// on the object, it still requires a mutable access.
    ///
    /// A good example is `GEOSWKTWriter_getOutputDimension_r` (which is very likely a bug).
    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType;
}

macro_rules! as_raw_impl {
    ($type_name:ty, $geos_type_name:ty) => {
        impl AsRaw for $type_name {
            type RawType = $geos_type_name;

            fn as_raw(&self) -> *const Self::RawType {
                *self.ptr
            }
        }
    };
}

macro_rules! as_raw_mut_impl {
    ($type_name:ty, $geos_type_name:ty) => {
        $crate::traits::as_raw_impl!($type_name, $geos_type_name);

        impl AsRawMut for $type_name {
            unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
                *self.ptr
            }
        }
    };
}

pub(crate) use as_raw_impl;
pub(crate) use as_raw_mut_impl;
