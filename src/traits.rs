pub trait AsRaw {
    type RawType;

    fn as_raw(&self) -> *const Self::RawType;
}

pub trait AsRawMut {
    type RawType;

    fn as_raw_mut(&mut self) -> *mut Self::RawType {
        unsafe { self.as_raw_mut_override() }
    }
    /// This method exists because in certain case, even though you don't run any mutable operation
    /// on the object, it still requires a mutable access.
    ///
    /// A good example is `GEOSWKTWriter_getOutputDimension_r` (which is very likely a bug).
    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType;
}
