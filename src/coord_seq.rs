use error::{Error, GResult};
use context_handle::PtrWrap;
use ffi::*;
use crate::{CoordDimensions, GContextHandle, AsRaw, ContextHandling, ContextInteractions};
use std::sync::Arc;

pub struct CoordSeq<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSCoordSequence>,
    pub(crate) context: Arc<GContextHandle<'a>>,
    nb_dimensions: usize,
    nb_lines: usize,
}

impl<'a> CoordSeq<'a> {
    /// Creates a new `CoordSeq`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coord_seq = CoordSeq::new(2, CoordDimensions::ThreeD)
    ///                              .expect("failed to create CoordSeq");
    ///
    /// // Then you fill the positions of your `coord_seq`:
    /// let positions: &[(f64, f64, f64)] = &[(0., 0., 0.), (1., 2., 1.)];
    /// for (pos, (x, y, z)) in positions.into_iter().enumerate() {
    ///     coord_seq.set_x(pos, *x).expect("failed to set x...");
    ///     coord_seq.set_y(pos, *y).expect("failed to set y...");
    ///     coord_seq.set_z(pos, *z).expect("failed to set z...");
    /// }
    /// assert!(coord_seq.get_z(1) == Ok(1.));
    ///
    /// // An example with 2 dimensions (and 3 lines) as well:
    /// let mut coord_seq2 = CoordSeq::new(3, CoordDimensions::TwoD)
    ///                               .expect("failed to create CoordSeq");
    /// let positions2: &[(f64, f64)] = &[(0., 0.), (1., 2.), (14., 5.)];
    /// for (pos, (x, y)) in positions2.into_iter().enumerate() {
    ///     coord_seq2.set_x(pos, *x).expect("failed to set x...");
    ///     coord_seq2.set_y(pos, *y).expect("failed to set y...");
    /// }
    /// assert!(coord_seq2.get_x(1) == Ok(1.));
    /// ```
    pub fn new(size: u32, dims: CoordDimensions) -> GResult<CoordSeq<'a>> {
        match GContextHandle::init() {
            Ok(context_handle) => {
                unsafe {
                    let ptr = GEOSCoordSeq_create_r(context_handle.as_raw(), size, dims.into());
                    CoordSeq::new_from_raw(ptr, Arc::new(context_handle), size, dims.into())
                }
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSCoordSequence,
        context: Arc<GContextHandle<'a>>,
        size: u32,
        dims: u32,
    ) -> GResult<CoordSeq<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr);
        }
        Ok(CoordSeq { ptr: PtrWrap(ptr), context, nb_dimensions: dims as _, nb_lines: size as _ })
    }

    pub fn set_x(&mut self, idx: usize, val: f64) -> GResult<()> {
        assert!(idx < self.nb_lines);

        let ret_val = unsafe {
            GEOSCoordSeq_setX_r(self.get_raw_context(), self.as_raw(), idx as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set x for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn set_y(&mut self, idx: usize, val: f64) -> GResult<()> {
        assert!(idx < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let ret_val = unsafe {
            GEOSCoordSeq_setY_r(self.get_raw_context(), self.as_raw(), idx as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set y for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn set_z(&mut self, idx: usize, val: f64) -> GResult<()> {
        assert!(idx < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let ret_val = unsafe {
            GEOSCoordSeq_setZ_r(self.get_raw_context(), self.as_raw(), idx as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set z for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn get_x(&self, idx: usize) -> GResult<f64> {
        assert!(idx < self.nb_lines);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getX_r(self.get_raw_context(), self.as_raw(), idx as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_y(&self, idx: usize) -> GResult<f64> {
        assert!(idx < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getY_r(self.get_raw_context(), self.as_raw(), idx as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_z(&self, idx: usize) -> GResult<f64> {
        assert!(idx < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ_r(self.get_raw_context(), self.as_raw(), idx as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn size(&self) -> GResult<usize> {
        let mut n = 0;
        let ret_val = unsafe {
            GEOSCoordSeq_getSize_r(self.get_raw_context(), self.as_raw(), &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(n as usize)
        }
    }

    pub fn dimensions(&self) -> GResult<CoordDimensions> {
        let mut n = 0;
        let ret_val = unsafe {
            GEOSCoordSeq_getDimensions_r(self.get_raw_context(), self.as_raw(), &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting dimensions from CoordSeq".into()))
        } else {
            Ok(CoordDimensions::from(n))
        }
    }
}

unsafe impl<'a> Send for CoordSeq<'a> {}
unsafe impl<'a> Sync for CoordSeq<'a> {}

impl<'a> Drop for CoordSeq<'a> {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        unsafe { GEOSCoordSeq_destroy_r(self.get_raw_context(), self.as_raw()) };
    }
}

impl<'a> Clone for CoordSeq<'a> {
    /// Also pass the context to the newly created `CoordSeq`.
    fn clone(&self) -> CoordSeq<'a> {
        let ptr = unsafe { GEOSCoordSeq_clone_r(self.get_raw_context(), self.as_raw()) };
        CoordSeq {
            ptr: PtrWrap(ptr),
            context: self.clone_context(),
            nb_dimensions: self.nb_dimensions,
            nb_lines: self.nb_lines,
        }
    }
}

impl<'a> ContextInteractions for CoordSeq<'a> {
    type Context = GContextHandle<'a>;

    /// Set the context handle to the `CoordSeq`.
    ///
    /// ```
    /// use geos::{ContextInteractions, CoordDimensions, CoordSeq, GContextHandle};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let mut coord_seq = CoordSeq::new(2, CoordDimensions::TwoD).expect("failed to create CoordSeq");
    /// coord_seq.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: Self::Context) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the `CoordSeq`.
    ///
    /// ```
    /// use geos::{ContextInteractions, CoordDimensions, CoordSeq};
    ///
    /// let coord_seq = CoordSeq::new(2, CoordDimensions::TwoD).expect("failed to create CoordSeq");
    /// let context = coord_seq.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &Self::Context {
        &self.context
    }
}

impl<'a> AsRaw for CoordSeq<'a> {
    type RawType = *mut GEOSCoordSequence;

    fn as_raw(&self) -> Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for CoordSeq<'a> {
    type Context = Arc<GContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<GContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
