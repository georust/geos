use error::{Error, GResult};
use context_handle::PtrWrap;
use geos_sys::*;
use crate::{
    AsRaw,
    ContextHandling,
    ContextInteractions,
    CoordDimensions,
    ContextHandle,
    Geometry,
    Ordinate,
};
use crate::enums::TryFrom;
use std::sync::Arc;

/// `CoordSeq` represents a list of coordinates inside a [`Geometry`].
///
/// # Example
///
/// ```
/// use geos::{CoordDimensions, CoordSeq};
///
/// let mut coords = CoordSeq::new(1, CoordDimensions::OneD)
///                           .expect("failed to create CoordSeq");
/// coords.set_x(0, 10.);
/// assert!(coords.get_x(0) == Ok(10.));
/// ```
pub struct CoordSeq<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSCoordSequence>,
    pub(crate) context: Arc<ContextHandle<'a>>,
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
        match ContextHandle::init_e(Some("CoordSeq::new")) {
            Ok(context_handle) => {
                unsafe {
                    let ptr = GEOSCoordSeq_create_r(context_handle.as_raw(), size, dims.into());
                    CoordSeq::new_from_raw(ptr, Arc::new(context_handle), size, dims.into(), "new")
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Creates a new `CoordSeq`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[0., 1.], &[2., 3.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.get_y(1) == Ok(3.));
    ///
    /// // Doing it from a Vec<Vec<f64>>.
    /// let positions = vec![vec![0., 1.], vec![2., 3.]];
    /// let s_positions = positions.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
    /// let coords = CoordSeq::new_from_vec(&s_positions)
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.get_y(1) == Ok(3.));
    ///
    /// // All vectors don't have the same length, this is an error!
    /// assert!(CoordSeq::new_from_vec(&[vec![0., 1.], vec![3.]]).is_err());
    ///
    /// // An empty vector is an error as well since we can't figure out its dimensions!
    /// let x: &[f64] = &[];
    /// assert!(CoordSeq::new_from_vec(&[x]).is_err());
    /// ```
    pub fn new_from_vec<T: AsRef<[f64]>>(data: &[T]) -> GResult<CoordSeq<'a>> {
        let size = data.len();

        if size > 0 {
            let dims = data[0].as_ref().len();
            if let Err(e) = CoordDimensions::try_from(dims as _) {
                return Err(Error::GenericError(e.to_owned()));
            }
            if !data.iter().skip(1).all(|x| x.as_ref().len() == dims) {
                return Err(Error::GenericError("All vec entries must have the same size!".into()));
            }
            match ContextHandle::init_e(Some("CoordSeq::new_from_vec")) {
                Ok(context_handle) => {
                    unsafe {
                        let ptr = GEOSCoordSeq_create_r(context_handle.as_raw(),
                                                        size as _,
                                                        dims as _);
                        CoordSeq::new_from_raw(ptr, Arc::new(context_handle), size as _, dims as _,
                                               "new_from_vec")
                    }
                }
                Err(e) => return Err(e),
            }.and_then(|coord| {
                let raw_context = coord.get_raw_context();
                let raw_coord = coord.as_raw();

                let funcs = [GEOSCoordSeq_setX_r,
                             GEOSCoordSeq_setY_r,
                             GEOSCoordSeq_setZ_r];

                for (line, line_data) in data.iter().enumerate() {
                    for (pos, elem) in line_data.as_ref().iter().enumerate() {
                        unsafe {
                            if funcs[pos](raw_context, raw_coord, line as _, *elem) == 0 {
                                let err = format!("Failed to set value at position {} on \
                                                   line {}", pos, line);
                                return Err(Error::GenericError(err));
                            }
                        }
                    }
                }
                Ok(coord)
            })
        } else {
            Err(Error::GenericError("Can't determine dimension for the CoordSeq".to_owned()))
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSCoordSequence,
        context: Arc<ContextHandle<'a>>,
        size: u32,
        dims: u32,
        caller: &str,
    ) -> GResult<CoordSeq<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr(format!("CoordSeq::{}", caller)));
        }
        Ok(CoordSeq { ptr: PtrWrap(ptr), context, nb_dimensions: dims as _, nb_lines: size as _ })
    }

    /// Sets the X position value at the given `line`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::OneD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_x(0, 10.);
    /// assert!(coords.get_x(0) == Ok(10.));
    /// ```
    pub fn set_x(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);

        let ret_val = unsafe {
            GEOSCoordSeq_setX_r(self.get_raw_context(), self.as_raw(), line as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set x for coord".into()))
        } else {
            Ok(())
        }
    }

    /// Sets the Y position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have at least two dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::TwoD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_y(0, 10.);
    /// assert!(coords.get_y(0) == Ok(10.));
    /// ```
    pub fn set_y(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let ret_val = unsafe {
            GEOSCoordSeq_setY_r(self.get_raw_context(), self.as_raw(), line as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set y for coord".into()))
        } else {
            Ok(())
        }
    }

    /// Sets the Z position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have three dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::ThreeD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_z(0, 10.);
    /// assert!(coords.get_z(0) == Ok(10.));
    /// ```
    pub fn set_z(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let ret_val = unsafe {
            GEOSCoordSeq_setZ_r(self.get_raw_context(), self.as_raw(), line as _, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set z for coord".into()))
        } else {
            Ok(())
        }
    }

    /// Sets the value at the given `ordinate` (aka position).
    ///
    /// Note: your `CoordSeq` object must have enough dimensions to set at the given `ordinate`!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Ordinate};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::ThreeD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_ordinate(0, Ordinate::Z, 10.);
    /// assert!(coords.get_z(0) == Ok(10.));
    /// assert!(coords.get_ordinate(0, Ordinate::Z) == 10.);
    /// ```
    pub fn set_ordinate(&mut self, line: usize, ordinate: Ordinate, val: f64) -> GResult<()> {
        let ordinate = ordinate.into();
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions > ordinate);

        let ret_val = unsafe {
            GEOSCoordSeq_setOrdinate_r(self.get_raw_context(), self.as_raw(), line as _,
                                       ordinate, val)
        };
        if ret_val == 0 {
            Err(Error::GeosError(format!("impossible to set value for ordinate {}", ordinate)))
        } else {
            Ok(())
        }
    }

    /// Gets the X position value at the given `line`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::OneD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_x(0, 10.);
    /// assert!(coords.get_x(0) == Ok(10.));
    /// ```
    pub fn get_x(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getX_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("failed to get coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    /// Gets the Y position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have at least two dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::TwoD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_y(0, 10.);
    /// assert!(coords.get_y(0) == Ok(10.));
    /// ```
    pub fn get_y(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getY_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("failed to get coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    /// Gets the Z position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have three dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::ThreeD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_z(0, 10.);
    /// assert!(coords.get_z(0) == Ok(10.));
    /// ```
    pub fn get_z(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError("failed to get coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    /// Gets the value at the given `ordinate` (aka position).
    ///
    /// Note: your `CoordSeq` object must have enough dimensions to access the given `ordinate`!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Ordinate};
    ///
    /// let mut coords = CoordSeq::new(1, CoordDimensions::ThreeD)
    ///                           .expect("failed to create CoordSeq");
    /// coords.set_ordinate(0, Ordinate::Z, 10.);
    /// assert!(coords.get_z(0) == Ok(10.));
    /// assert!(coords.get_ordinate(0, Ordinate::Z) == 10.);
    /// ```
    pub fn get_ordinate(&self, line: usize, ordinate: Ordinate) -> f64 {
        let ordinate = ordinate.into();
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions > ordinate);

        unsafe {
            GEOSCoordSeq_getOrdinate_r(self.get_raw_context(), self.as_raw(), line as _, ordinate)
        }
    }

    /// Returns the number of lines of the `CoordSeq` object.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let coords = CoordSeq::new(2, CoordDimensions::ThreeD)
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.size() == Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.size() == Ok(4));
    /// ```
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

    /// Returns the number of lines of the `CoordSeq` object.
    ///
    /// Note: This is an alias to the [`size`](#method.size) method.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let coords = CoordSeq::new(2, CoordDimensions::ThreeD)
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.number_of_lines() == Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.number_of_lines() == Ok(4));
    /// ```
    pub fn number_of_lines(&self) -> GResult<usize> {
        self.size()
    }

    /// Returns the number of dimensions of the `CoordSeq` object.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq};
    ///
    /// let coords = CoordSeq::new(2, CoordDimensions::OneD)
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.dimensions() == Ok(CoordDimensions::OneD));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3. ,4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert!(coords.dimensions() == Ok(CoordDimensions::TwoD));
    /// ```
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

    /// Returns `true` if the geometry has a counter-clockwise orientation.
    #[cfg(feature = "v3_7_0")]
    pub fn is_ccw(&self) -> GResult<bool> {
        unsafe {
            let mut is_ccw = 0;
            if GEOSCoordSeq_isCCW_r(self.get_raw_context(), self.as_raw(), &mut is_ccw) != 1 {
                Err(Error::GenericError("GEOSCoordSeq_isCCW_r failed".to_owned()))
            } else {
                Ok(is_ccw == 1)
            }
        }
    }

    pub fn create_point(self) -> GResult<Geometry<'a>> {
        Geometry::create_point(self)
    }

    pub fn create_line_string(self) -> GResult<Geometry<'a>> {
        Geometry::create_line_string(self)
    }

    pub fn create_linear_ring(self) -> GResult<Geometry<'a>> {
        Geometry::create_linear_ring(self)
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
        let context = self.clone_context();
        let ptr = unsafe { GEOSCoordSeq_clone_r(context.as_raw(), self.as_raw()) };
        if ptr.is_null() {
            panic!("Couldn't clone CoordSeq...");
        }
        CoordSeq {
            ptr: PtrWrap(ptr),
            context,
            nb_dimensions: self.nb_dimensions,
            nb_lines: self.nb_lines,
        }
    }
}

impl<'a> ContextInteractions<'a> for CoordSeq<'a> {
    /// Set the context handle to the `CoordSeq`.
    ///
    /// ```
    /// use geos::{ContextInteractions, CoordDimensions, CoordSeq, ContextHandle};
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let mut coord_seq = CoordSeq::new(2, CoordDimensions::TwoD).expect("failed to create CoordSeq");
    /// coord_seq.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
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
    fn get_context_handle(&self) -> &ContextHandle<'a> {
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
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
