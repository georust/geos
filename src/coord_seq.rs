use crate::context_handle::PtrWrap;
use crate::error::{Error, GResult};
use crate::{
    AsRaw, AsRawMut, ContextHandle, ContextHandling, ContextInteractions, CoordDimensions,
    Geometry, Ordinate,
};
use geos_sys::*;
use std::convert::TryFrom;
use std::sync::Arc;

#[cfg(feature = "v3_10_0")]
type AsArrayOutput = (Vec<f64>, Vec<f64>, Option<Vec<f64>>, Option<Vec<f64>>);

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
/// assert_eq!(coords.get_x(0), Ok(10.));
/// ```
pub struct CoordSeq<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSCoordSequence>,
    pub(crate) context: Arc<ContextHandle<'a>>,
    nb_dimensions: usize,
    nb_lines: usize,
}

/// Representation of an immutable GEOS coordinate sequence. Since it's only a view over another
/// GEOS geometry's data, only immutable operations are implemented on it.
pub struct ConstCoordSeq<'a, 'b> {
    pub(crate) ptr: PtrWrap<*const GEOSCoordSequence>,
    pub(crate) original: &'b Geometry<'a>,
    // pub(crate) context: ContextHandle<'a>,
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
    /// assert_eq!(coord_seq.get_z(1), Ok(1.));
    ///
    /// // An example with 2 dimensions (and 3 lines) as well:
    /// let mut coord_seq2 = CoordSeq::new(3, CoordDimensions::TwoD)
    ///                               .expect("failed to create CoordSeq");
    /// let positions2: &[(f64, f64)] = &[(0., 0.), (1., 2.), (14., 5.)];
    /// for (pos, (x, y)) in positions2.into_iter().enumerate() {
    ///     coord_seq2.set_x(pos, *x).expect("failed to set x...");
    ///     coord_seq2.set_y(pos, *y).expect("failed to set y...");
    /// }
    /// assert_eq!(coord_seq2.get_x(1), Ok(1.));
    /// ```
    pub fn new(size: u32, dims: CoordDimensions) -> GResult<CoordSeq<'a>> {
        let context_handle = ContextHandle::init_e(Some("CoordSeq::new"))?;
        unsafe {
            let ptr = GEOSCoordSeq_create_r(context_handle.as_raw(), size, dims.into());
            CoordSeq::new_from_raw(ptr, Arc::new(context_handle), size, dims.into(), "new")
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
    /// assert_eq!(coords.get_y(1), Ok(3.));
    ///
    /// // Doing it from a Vec<Vec<f64>>.
    /// let positions = vec![vec![0., 1.], vec![2., 3.]];
    /// let s_positions = positions.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
    /// let coords = CoordSeq::new_from_vec(&s_positions)
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.get_y(1), Ok(3.));
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
            if let Err(e) = CoordDimensions::try_from(dims as u32) {
                return Err(Error::GenericError(e.to_owned()));
            }
            if !data.iter().skip(1).all(|x| x.as_ref().len() == dims) {
                return Err(Error::GenericError(
                    "All vec entries must have the same size!".into(),
                ));
            }
            match ContextHandle::init_e(Some("CoordSeq::new_from_vec")) {
                Ok(context_handle) => unsafe {
                    let ptr = GEOSCoordSeq_create_r(context_handle.as_raw(), size as _, dims as _);
                    CoordSeq::new_from_raw(
                        ptr,
                        Arc::new(context_handle),
                        size as _,
                        dims as _,
                        "new_from_vec",
                    )
                },
                Err(e) => return Err(e),
            }
            .and_then(|mut coord| {
                let raw_context = coord.get_raw_context();
                let raw_coord = coord.as_raw_mut();

                let funcs = [
                    GEOSCoordSeq_setX_r,
                    GEOSCoordSeq_setY_r,
                    GEOSCoordSeq_setZ_r,
                ];

                for (line, line_data) in data.iter().enumerate() {
                    for (pos, elem) in line_data.as_ref().iter().enumerate() {
                        unsafe {
                            if funcs[pos](raw_context, raw_coord, line as _, *elem) == 0 {
                                let err = format!(
                                    "Failed to set value at position {pos} on line {line}",
                                );
                                return Err(Error::GenericError(err));
                            }
                        }
                    }
                }
                Ok(coord)
            })
        } else {
            Err(Error::GenericError(
                "Can't determine dimension for the CoordSeq".to_owned(),
            ))
        }
    }

    /// Creates a new `CoordSeq` from an interleaved coordinate buffer.
    ///
    /// # Parameters
    ///
    /// - `data`: The data buffer as a contiguous f64 slice.
    /// - `size`: The number of coordinates in the provided data buffer.
    /// - `has_z`: Whether the data buffer contains `z` coordinates. If `false`, the coordinate
    ///   buffer must be either `XY` or `XYM`. If `true`, the coordinate buffer must be either
    ///   `XYZ` or `XYZM`.
    /// - `has_m`: Whether the data buffer contains `m` coordinates. If `false`, the coordinate
    ///   buffer must be either `XY` or `XYZ`. If `true`, the coordinate buffer must be either
    ///   `XYM` or `XYZM`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.get_y(1), Ok(3.));
    /// assert_eq!(coords.get_x(2), Ok(4.));
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn new_from_buffer(
        data: &[f64],
        size: usize,
        has_z: bool,
        has_m: bool,
    ) -> GResult<CoordSeq<'a>> {
        let mut dims: u32 = 2;
        if has_z {
            dims += 1;
        }
        if has_m {
            dims += 1;
        }

        assert_eq!(data.len(), size * dims as usize, "Incorrect buffer length");

        let context_handle = ContextHandle::init_e(Some("CoordSeq::new_from_buffer"))?;
        unsafe {
            let ptr = GEOSCoordSeq_copyFromBuffer_r(
                context_handle.as_raw(),
                data.as_ptr(),
                size as _,
                has_z as _,
                has_m as _,
            );
            CoordSeq::new_from_raw(
                ptr,
                Arc::new(context_handle),
                size as _,
                dims,
                "new_from_buffer",
            )
        }
    }

    /// Creates a new `CoordSeq` from separated coordinate buffers.
    ///
    /// # Parameters
    ///
    /// - `x`: A slice of x coordinates.
    /// - `y`: A slice of y coordinates.
    /// - `z`: An optional slice of z coordinates.
    /// - `m`: An optional slice of m coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let x = vec![0., 2., 4.];
    /// let y = vec![1., 3., 5.];
    /// let coords = CoordSeq::new_from_arrays(&x, &y, None, None)
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.get_y(1), Ok(3.));
    /// assert_eq!(coords.get_x(2), Ok(4.));
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn new_from_arrays(
        x: &[f64],
        y: &[f64],
        z: Option<&[f64]>,
        m: Option<&[f64]>,
    ) -> GResult<CoordSeq<'a>> {
        assert_eq!(x.len(), y.len(), "Arrays have different lengths.");

        let mut dims: u32 = 2;
        let z_ptr = if let Some(z) = z {
            assert_eq!(x.len(), z.len(), "Arrays have different lengths.");
            dims += 1;
            z.as_ptr()
        } else {
            std::ptr::null()
        };
        let m_ptr = if let Some(m) = m {
            assert_eq!(x.len(), m.len(), "Arrays have different lengths.");
            dims += 1;
            m.as_ptr()
        } else {
            std::ptr::null()
        };

        let context_handle = ContextHandle::init_e(Some("CoordSeq::new_from_arrays"))?;
        unsafe {
            let ptr = GEOSCoordSeq_copyFromArrays_r(
                context_handle.as_raw(),
                x.as_ptr(),
                y.as_ptr(),
                z_ptr,
                m_ptr,
                x.len() as _,
            );
            CoordSeq::new_from_raw(
                ptr,
                Arc::new(context_handle),
                x.len() as u32,
                dims,
                "new_from_buffer",
            )
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
            let extra = if let Some(x) = context.get_last_error() {
                format!("\nLast error: {x}")
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "CoordSeq::{caller}{extra}",
            )));
        }
        Ok(CoordSeq {
            ptr: PtrWrap(ptr),
            context,
            nb_dimensions: dims as _,
            nb_lines: size as _,
        })
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
    /// assert_eq!(coords.get_x(0), Ok(10.));
    /// ```
    pub fn set_x(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);

        let ret_val = unsafe {
            GEOSCoordSeq_setX_r(self.get_raw_context(), self.as_raw_mut(), line as _, val)
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
    /// assert_eq!(coords.get_y(0), Ok(10.));
    /// ```
    pub fn set_y(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let ret_val = unsafe {
            GEOSCoordSeq_setY_r(self.get_raw_context(), self.as_raw_mut(), line as _, val)
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// ```
    pub fn set_z(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let ret_val = unsafe {
            GEOSCoordSeq_setZ_r(self.get_raw_context(), self.as_raw_mut(), line as _, val)
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// assert_eq!(coords.get_ordinate(0, Ordinate::Z), Ok(10.));
    /// ```
    pub fn set_ordinate(&mut self, line: usize, ordinate: Ordinate, val: f64) -> GResult<()> {
        let ordinate: u32 = ordinate.into();
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions > ordinate as _);

        let ret_val = unsafe {
            GEOSCoordSeq_setOrdinate_r(
                self.get_raw_context(),
                self.as_raw_mut(),
                line as _,
                ordinate,
                val,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError(format!(
                "impossible to set value for ordinate {ordinate}",
            )))
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
    /// assert_eq!(coords.get_x(0), Ok(10.));
    /// ```
    pub fn get_x(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getX_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
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
    /// assert_eq!(coords.get_y(0), Ok(10.));
    /// ```
    pub fn get_y(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getY_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// ```
    pub fn get_z(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
        }
    }

    /// Gets the entire `CoordSeq` object as an interleaved buffer.
    ///
    /// # Parameters:
    ///
    /// - `dims`: Optionally, the number of dimensions to include in the output buffer. If `None`,
    ///   will be inferred from the number of dimensions on the geometry. A user may want to
    ///   override this for performance because GEOS always stores coordinates internally with 3 or
    ///   4 dimensions. So copying 2-dimensional GEOS coordinates to a 3-dimensional output buffer
    ///   is slightly faster (a straight `memcpy`) than copying 2-dimensionalal GEOS coordinates to
    ///   a 2-dimensional output buffer (iterating over every coordinate and copying only the XY
    ///   values).
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// // Create a two-dimensional `CoordSeq` from a buffer with three points
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, inferring the number of coordinates (2)
    /// let output_buffer = coords.as_buffer(None)
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., 2., 3., 4., 5.];
    /// assert_eq!(output_buffer, expected_output_buffer);
    /// ```
    ///
    /// You can also force GEOS to return a 3D buffer from 2D coordinates
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, forcing it to be 3-dimensional
    /// let output_buffer = coords.as_buffer(Some(3))
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., f64::NAN, 2., 3., f64::NAN, 4., 5., f64::NAN];
    /// assert_eq!(output_buffer[0], expected_output_buffer[0]);
    /// assert_eq!(output_buffer[3], expected_output_buffer[3]);
    /// ```
    ///
    /// You can also force GEOS to return a 2D buffer from 3D coordinates
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 100., 2., 3., 200.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 2, true, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, forcing it to be 2-dimensional
    /// let output_buffer = coords.as_buffer(Some(2))
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., 2., 3.];
    /// assert_eq!(output_buffer, expected_output_buffer);
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_buffer(&self, dims: Option<usize>) -> GResult<Vec<f64>> {
        let size = self.nb_lines;
        let dims = dims.unwrap_or(self.nb_dimensions);

        let has_z = dims >= 3;
        let has_m = dims >= 4;

        let mut output_buffer = vec![0.; size * dims];
        unsafe {
            GEOSCoordSeq_copyToBuffer_r(
                self.get_raw_context(),
                self.as_raw(),
                output_buffer.as_mut_ptr(),
                has_z as _,
                has_m as _,
            );
        }

        Ok(output_buffer)
    }

    /// Gets the entire `CoordSeq` object as individual coordinate arrays.
    ///
    /// Returns a tuple with four vectors. The first and second vectors correspond to `x` and `y`
    /// coordinates. The third corresponds to an optional third dimension if it exists (`z` or
    /// `m`). The fourth corresponds to an optional fourth dimension if it exists (`m` in the case
    /// of an `XYZM` geometry).
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let output_arrays = coords.as_arrays()
    ///                           .expect("failed to get arrays");
    ///
    /// assert_eq!(output_arrays.0, vec![0., 2., 4.]);
    /// assert_eq!(output_arrays.1, vec![1., 3., 5.]);
    /// assert_eq!(output_arrays.2, None);
    /// assert_eq!(output_arrays.3, None);
    /// ```
    ///
    /// If the `CoordSeq` contains three dimensions, the third dimension will be returned as the
    /// third value in the tuple:
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 100., 3., 4., 200., 6., 7., 300.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, true, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let output_arrays = coords.as_arrays()
    ///                           .expect("failed to get arrays");
    ///
    /// assert_eq!(output_arrays.0, vec![0., 3., 6.]);
    /// assert_eq!(output_arrays.1, vec![1., 4., 7.]);
    /// assert_eq!(output_arrays.2, Some(vec![100., 200., 300.]));
    /// assert_eq!(output_arrays.3, None);
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_arrays(&self) -> GResult<AsArrayOutput> {
        let size = self.nb_lines;
        let mut x = vec![0.; size];
        let mut y = vec![0.; size];
        let mut z = if self.nb_dimensions == 3 {
            Some(vec![0.; size])
        } else {
            None
        };
        let mut m = if self.nb_dimensions == 4 {
            Some(vec![0.; size])
        } else {
            None
        };

        unsafe {
            GEOSCoordSeq_copyToArrays_r(
                self.get_raw_context(),
                self.as_raw(),
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut()
                    .map(|arr| arr.as_mut_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                m.as_mut()
                    .map(|arr| arr.as_mut_ptr())
                    .unwrap_or(std::ptr::null_mut()),
            );
        }

        Ok((x, y, z, m))
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// assert_eq!(coords.get_ordinate(0, Ordinate::Z), Ok(10.));
    /// ```
    pub fn get_ordinate(&self, line: usize, ordinate: Ordinate) -> GResult<f64> {
        let ordinate: u32 = ordinate.into();
        let mut val = 0f64;
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions > ordinate as _);

        if unsafe {
            GEOSCoordSeq_getOrdinate_r(
                self.get_raw_context(),
                self.as_raw(),
                line as _,
                ordinate,
                &mut val,
            )
        } != 1
        {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(val)
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
    /// assert_eq!(coords.size(), Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.size(), Ok(4));
    /// ```
    pub fn size(&self) -> GResult<usize> {
        let mut n = 0;
        let ret_val =
            unsafe { GEOSCoordSeq_getSize_r(self.get_raw_context(), self.as_raw(), &mut n) };
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
    /// assert_eq!(coords.number_of_lines(), Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.number_of_lines(), Ok(4));
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
    /// assert_eq!(coords.dimensions(), Ok(CoordDimensions::OneD));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3. ,4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.dimensions(), Ok(CoordDimensions::TwoD));
    /// ```
    pub fn dimensions(&self) -> GResult<CoordDimensions> {
        let mut n = 0;
        let ret_val =
            unsafe { GEOSCoordSeq_getDimensions_r(self.get_raw_context(), self.as_raw(), &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting dimensions from CoordSeq".into()))
        } else {
            Ok(CoordDimensions::try_from(n).expect("Failed to convert to CoordDimensions"))
        }
    }

    /// Returns `true` if the geometry has a counter-clockwise orientation.
    ///
    /// Available using the `v3_7_0` feature.
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn is_ccw(&self) -> GResult<bool> {
        unsafe {
            let mut is_ccw = 0;
            if GEOSCoordSeq_isCCW_r(self.get_raw_context(), self.as_raw(), &mut is_ccw) != 1 {
                Err(Error::GenericError(
                    "GEOSCoordSeq_isCCW_r failed".to_owned(),
                ))
            } else {
                Ok(is_ccw == 1)
            }
        }
    }

    /// Creates a point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_point(coords).expect("Failed to create point");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "POINT (1.0000000000000000 2.0000000000000000)");
    /// ```
    pub fn create_point(self) -> GResult<Geometry<'a>> {
        Geometry::create_point(self)
    }

    /// Creates a line string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3., 4.]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_line_string(coords).expect("Failed to create line string");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "LINESTRING (1.0000000000000000 2.0000000000000000, \
    ///                         3.0000000000000000 4.0000000000000000)");
    /// ```
    pub fn create_line_string(self) -> GResult<Geometry<'a>> {
        Geometry::create_line_string(self)
    }

    /// Creates a linear ring geometry.
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
        unsafe { GEOSCoordSeq_destroy_r(self.get_raw_context(), self.as_raw_mut()) };
    }
}

impl<'a> Clone for CoordSeq<'a> {
    /// Also pass the context to the newly created `CoordSeq`.
    fn clone(&self) -> CoordSeq<'a> {
        let ptr = unsafe { GEOSCoordSeq_clone_r(self.get_raw_context(), self.as_raw()) };
        if ptr.is_null() {
            panic!("Couldn't clone CoordSeq...");
        }
        CoordSeq {
            ptr: PtrWrap(ptr),
            context: self.clone_context(),
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
    type RawType = GEOSCoordSequence;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl<'a> AsRawMut for CoordSeq<'a> {
    type RawType = GEOSCoordSequence;

    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
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

impl<'a, 'b> ConstCoordSeq<'a, 'b> {
    pub(crate) unsafe fn new_from_raw(
        ptr: *const GEOSCoordSequence,
        original: &'b Geometry<'a>,
        size: u32,
        dims: u32,
        caller: &str,
    ) -> GResult<ConstCoordSeq<'a, 'b>> {
        if ptr.is_null() {
            let extra = if let Some(x) = original.context.get_last_error() {
                format!("\nLast error: {x}")
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "CoordSeq::{caller}{extra}",
            )));
        }
        Ok(ConstCoordSeq {
            ptr: PtrWrap(ptr),
            original,
            nb_dimensions: dims as _,
            nb_lines: size as _,
        })
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
    /// assert_eq!(coords.get_x(0), Ok(10.));
    /// ```
    pub fn get_x(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getX_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
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
    /// assert_eq!(coords.get_y(0), Ok(10.));
    /// ```
    pub fn get_y(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 2);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getY_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// ```
    pub fn get_z(&self, line: usize) -> GResult<f64> {
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions >= 3);

        let mut n = 0.;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ_r(self.get_raw_context(), self.as_raw(), line as _, &mut n)
        };
        if ret_val == 0 {
            Err(Error::GeosError(
                "failed to get coordinates from CoordSeq".into(),
            ))
        } else {
            Ok(n as _)
        }
    }

    /// Gets the entire `CoordSeq` object as an interleaved buffer.
    ///
    /// # Parameters:
    ///
    /// - `dims`: Optionally, the number of dimensions to include in the output buffer. If `None`,
    ///   will be inferred from the number of dimensions on the geometry. A user may want to
    ///   override this for performance because GEOS always stores coordinates internally with 3 or
    ///   4 dimensions. So copying 2-dimensional GEOS coordinates to a 3-dimensional output buffer
    ///   is slightly faster (a straight `memcpy`) than copying 2-dimensionalal GEOS coordinates to
    ///   a 2-dimensional output buffer (iterating over every coordinate and copying only the XY
    ///   values).
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// // Create a two-dimensional `CoordSeq` from a buffer with three points
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, inferring the number of coordinates (2)
    /// let output_buffer = coords.as_buffer(None)
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., 2., 3., 4., 5.];
    /// assert_eq!(output_buffer, expected_output_buffer);
    /// ```
    ///
    /// You can also force GEOS to return a 3D buffer from 2D coordinates
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, forcing it to be 3-dimensional
    /// let output_buffer = coords.as_buffer(Some(3))
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., f64::NAN, 2., 3., f64::NAN, 4., 5., f64::NAN];
    /// assert_eq!(output_buffer[0], expected_output_buffer[0]);
    /// assert_eq!(output_buffer[3], expected_output_buffer[3]);
    /// ```
    ///
    /// You can also force GEOS to return a 2D buffer from 3D coordinates
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 100., 2., 3., 200.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 2, true, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// // Return an output buffer, forcing it to be 2-dimensional
    /// let output_buffer = coords.as_buffer(Some(2))
    ///                           .expect("failed to get buffer");
    ///
    /// let expected_output_buffer = vec![0., 1., 2., 3.];
    /// assert_eq!(output_buffer, expected_output_buffer);
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_buffer(&self, dims: Option<usize>) -> GResult<Vec<f64>> {
        let size = self.nb_lines;
        let dims = dims.unwrap_or(self.nb_dimensions);

        let has_z = dims >= 3;
        let has_m = dims >= 4;

        let mut output_buffer = vec![0.; size * dims];
        unsafe {
            GEOSCoordSeq_copyToBuffer_r(
                self.get_raw_context(),
                self.as_raw(),
                output_buffer.as_mut_ptr(),
                has_z as _,
                has_m as _,
            );
        }

        Ok(output_buffer)
    }

    /// Gets the entire `CoordSeq` object as individual coordinate arrays.
    ///
    /// Returns a tuple with four vectors. The first and second vectors correspond to `x` and `y`
    /// coordinates. The third corresponds to an optional third dimension if it exists (`z` or
    /// `m`). The fourth corresponds to an optional fourth dimension if it exists (`m` in the case
    /// of an `XYZM` geometry).
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, false, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let output_arrays = coords.as_arrays()
    ///                           .expect("failed to get arrays");
    ///
    /// assert_eq!(output_arrays.0, vec![0., 2., 4.]);
    /// assert_eq!(output_arrays.1, vec![1., 3., 5.]);
    /// assert_eq!(output_arrays.2, None);
    /// assert_eq!(output_arrays.3, None);
    /// ```
    ///
    /// If the `CoordSeq` contains three dimensions, the third dimension will be returned as the
    /// third value in the tuple:
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let buffer = vec![0., 1., 100., 3., 4., 200., 6., 7., 300.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, true, false)
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let output_arrays = coords.as_arrays()
    ///                           .expect("failed to get arrays");
    ///
    /// assert_eq!(output_arrays.0, vec![0., 3., 6.]);
    /// assert_eq!(output_arrays.1, vec![1., 4., 7.]);
    /// assert_eq!(output_arrays.2, Some(vec![100., 200., 300.]));
    /// assert_eq!(output_arrays.3, None);
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_arrays(&self) -> GResult<AsArrayOutput> {
        let size = self.nb_lines;
        let mut x = vec![0.; size];
        let mut y = vec![0.; size];
        let mut z = if self.nb_dimensions == 3 {
            Some(vec![0.; size])
        } else {
            None
        };
        let mut m = if self.nb_dimensions == 4 {
            Some(vec![0.; size])
        } else {
            None
        };

        unsafe {
            GEOSCoordSeq_copyToArrays_r(
                self.get_raw_context(),
                self.as_raw(),
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut()
                    .map(|arr| arr.as_mut_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                m.as_mut()
                    .map(|arr| arr.as_mut_ptr())
                    .unwrap_or(std::ptr::null_mut()),
            );
        }

        Ok((x, y, z, m))
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
    /// assert_eq!(coords.get_z(0), Ok(10.));
    /// assert_eq!(coords.get_ordinate(0, Ordinate::Z), Ok(10.));
    /// ```
    pub fn get_ordinate(&self, line: usize, ordinate: Ordinate) -> GResult<f64> {
        let ordinate: u32 = ordinate.into();
        let mut val = 0f64;
        assert!(line < self.nb_lines);
        assert!(self.nb_dimensions > ordinate as _);

        if unsafe {
            GEOSCoordSeq_getOrdinate_r(
                self.get_raw_context(),
                self.as_raw(),
                line as _,
                ordinate,
                &mut val,
            )
        } != 1
        {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(val)
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
    /// assert_eq!(coords.size(), Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.size(), Ok(4));
    /// ```
    pub fn size(&self) -> GResult<usize> {
        let mut n = 0;
        let ret_val =
            unsafe { GEOSCoordSeq_getSize_r(self.get_raw_context(), self.as_raw(), &mut n) };
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
    /// assert_eq!(coords.number_of_lines(), Ok(2));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1f64], &[2.], &[3.], &[4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.number_of_lines(), Ok(4));
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
    /// assert_eq!(coords.dimensions(), Ok(CoordDimensions::OneD));
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3. ,4.]])
    ///                       .expect("failed to create CoordSeq");
    /// assert_eq!(coords.dimensions(), Ok(CoordDimensions::TwoD));
    /// ```
    pub fn dimensions(&self) -> GResult<CoordDimensions> {
        let mut n = 0;
        let ret_val =
            unsafe { GEOSCoordSeq_getDimensions_r(self.get_raw_context(), self.as_raw(), &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting dimensions from CoordSeq".into()))
        } else {
            Ok(CoordDimensions::try_from(n).expect("Failed to convert to CoordDimensions"))
        }
    }

    /// Returns `true` if the geometry has a counter-clockwise orientation.
    ///
    /// Available using the `v3_7_0` feature.
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn is_ccw(&self) -> GResult<bool> {
        unsafe {
            let mut is_ccw = 0;
            if GEOSCoordSeq_isCCW_r(self.get_raw_context(), self.as_raw(), &mut is_ccw) != 1 {
                Err(Error::GenericError(
                    "GEOSCoordSeq_isCCW_r failed".to_owned(),
                ))
            } else {
                Ok(is_ccw == 1)
            }
        }
    }
}

unsafe impl<'a, 'b> Send for ConstCoordSeq<'a, 'b> {}
unsafe impl<'a, 'b> Sync for ConstCoordSeq<'a, 'b> {}

// Assuming that the parent will drop the coord seq?

// impl<'a> Drop for ConstCoordSeq<'a> {
//     fn drop(&mut self) {
//         if self.ptr.is_null() {
//             return;
//         }
//         unsafe { GEOSCoordSeq_destroy_r(self.get_raw_context(), self.as_raw_mut()) };
//     }
// }

// impl<'a> Clone for CoordSeq<'a> {
//     /// Also pass the context to the newly created `CoordSeq`.
//     fn clone(&self) -> CoordSeq<'a> {
//         let ptr = unsafe { GEOSCoordSeq_clone_r(self.get_raw_context(), self.as_raw()) };
//         if ptr.is_null() {
//             panic!("Couldn't clone CoordSeq...");
//         }
//         CoordSeq {
//             ptr: PtrWrap(ptr),
//             context: self.clone_context(),
//             nb_dimensions: self.nb_dimensions,
//             nb_lines: self.nb_lines,
//         }
//     }
// }

// impl<'a, 'b> ContextInteractions<'a> for ConstCoordSeq<'a, 'b> {
//     /// Get the context handle of the `CoordSeq`.
//     ///
//     /// ```
//     /// use geos::{ContextInteractions, CoordDimensions, CoordSeq};
//     ///
//     /// let coord_seq = CoordSeq::new(2, CoordDimensions::TwoD).expect("failed to create CoordSeq");
//     /// let context = coord_seq.get_context_handle();
//     /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
//     /// ```
//     fn get_context_handle(&self) -> &ContextHandle<'a> {
//         &self.context
//     }
// }

impl<'a, 'b> AsRaw for ConstCoordSeq<'a, 'b> {
    type RawType = GEOSCoordSequence;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl<'a, 'b> ContextHandling for ConstCoordSeq<'a, 'b> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.original.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.original.context)
    }
}
