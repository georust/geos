use crate::context_handle::with_context;
use crate::error::{Error, GResult};
#[cfg(feature = "v3_14_0")]
use crate::functions::predicate;
use crate::functions::{errcheck, nullcheck};
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, CoordDimensions, CoordType, Geometry, Ordinate};
use geos_sys::*;
use std::convert::TryFrom;
use std::ptr::NonNull;

#[cfg(feature = "v3_10_0")]
type AsArrayOutput = (Vec<f64>, Vec<f64>, Option<Vec<f64>>, Option<Vec<f64>>);

/// `CoordSeq` represents a list of coordinates inside a [`Geometry`].
///
/// # Example
///
/// ```
/// use geos::{CoordSeq, CoordType};
///
/// let mut coords = CoordSeq::new(1, CoordType::XY)?;
/// coords.set_x(0, 10.)?;
/// assert_eq!(coords.get_x(0)?, 10.);
/// # Ok::<(), geos::Error>(())
/// ```
pub struct CoordSeq {
    pub(crate) ptr: NonNull<GEOSCoordSequence>,
    size: usize,
    coord_type: CoordType,
}

as_raw_mut_impl!(CoordSeq, GEOSCoordSequence);

impl CoordSeq {
    /// Creates a new `CoordSeq`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coord_seq = CoordSeq::new(2, CoordType::XYZ)?;
    ///
    /// // Then you fill the positions of your `coord_seq`:
    /// let positions: &[(f64, f64, f64)] = &[(0., 0., 0.), (1., 2., 1.)];
    /// for (pos, (x, y, z)) in positions.into_iter().enumerate() {
    ///     coord_seq.set_x(pos, *x)?;
    ///     coord_seq.set_y(pos, *y)?;
    ///     coord_seq.set_z(pos, *z)?;
    /// }
    /// assert_eq!(coord_seq.get_z(1)?, 1.);
    ///
    /// // An example with 2 dimensions (and 3 lines) as well:
    /// let mut coord_seq2 = CoordSeq::new(3, CoordType::XY)?;
    /// let positions2: &[(f64, f64)] = &[(0., 0.), (1., 2.), (14., 5.)];
    /// for (pos, (x, y)) in positions2.into_iter().enumerate() {
    ///     coord_seq2.set_x(pos, *x)?;
    ///     coord_seq2.set_y(pos, *y)?;
    /// }
    /// assert_eq!(coord_seq2.get_x(1)?, 1.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn new(size: u32, coord_type: CoordType) -> GResult<Self> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSCoordSeq_createWithDimensions_r(
                ctx.as_raw(),
                size,
                coord_type.has_z().into(),
                coord_type.has_m().into(),
            ))?;
            Ok(Self::new_from_raw(ptr, size as _, coord_type))
        })
    }

    #[cfg(not(feature = "v3_14_0"))]
    pub fn new(size: u32, coord_type: CoordType) -> GResult<Self> {
        with_context(|ctx| unsafe {
            #[cfg(feature = "v3_12_0")]
            if coord_type == CoordType::XYM {
                return CoordSeq::new_from_buffer(
                    &vec![f64::NAN; 3 * size as usize],
                    size as usize,
                    coord_type,
                );
            }
            let ptr = nullcheck!(GEOSCoordSeq_create_r(ctx.as_raw(), size, coord_type.into()))?;
            Ok(Self::new_from_raw(ptr, size as _, coord_type))
        })
    }

    /// Creates a new `CoordSeq`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::CoordSeq;
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[0., 1.], &[2., 3.]])?;
    /// assert_eq!(coords.get_y(0)?, 1.);
    /// assert_eq!(coords.get_y(1)?, 3.);
    ///
    /// // Doing it from a Vec<Vec<f64>>.
    /// let positions = vec![vec![0., 1.], vec![2., 3.]];
    /// let s_positions = positions.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
    /// let coords = CoordSeq::new_from_vec(&s_positions)?;
    /// assert_eq!(coords.get_y(0)?, 1.);
    /// assert_eq!(coords.get_y(1)?, 3.);
    ///
    /// // All vectors don't have the same length, this is an error!
    /// assert!(CoordSeq::new_from_vec(&[vec![0., 1.], vec![3.]]).is_err());
    ///
    /// // An empty vector is an error as well since we can't figure out its dimensions!
    /// let x: &[f64] = &[];
    /// assert!(CoordSeq::new_from_vec(&[x]).is_err());
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn new_from_vec<T: AsRef<[f64]>>(data: &[T]) -> GResult<Self> {
        let size = data.len();

        if size == 0 {
            return Err(Error::GenericError(
                "Can't determine dimension for the CoordSeq".to_owned(),
            ));
        }

        let coord_dims = data[0].as_ref().len();
        let coord_type = CoordType::try_from(coord_dims as u32)?;
        if !data.iter().skip(1).all(|x| x.as_ref().len() == coord_dims) {
            return Err(Error::GenericError(
                "All vec entries must have the same size!".into(),
            ));
        }

        #[cfg(all(feature = "v3_12_0", not(feature = "v3_14_0")))]
        if coord_type == CoordType::XYZM {
            return Err(Error::GenericError(
                "XYZM Coordinates only supported with GEOS 3.14 onwards for this function."
                    .to_owned(),
            ));
        }

        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSCoordSeq_create_r(
                ctx.as_raw(),
                size as _,
                coord_dims as _
            ))?;
            let mut coord = Self::new_from_raw(ptr, size as _, coord_type);

            let raw_coord = coord.as_raw_mut();

            for (line, line_data) in data.iter().enumerate() {
                for (pos, elem) in line_data.as_ref().iter().enumerate() {
                    match pos {
                        0 => errcheck! { GEOSCoordSeq_setX_r(ctx.as_raw(), raw_coord, line as _, *elem) },
                        1 => errcheck! { GEOSCoordSeq_setY_r(ctx.as_raw(), raw_coord, line as _, *elem) },
                        2 => errcheck! { GEOSCoordSeq_setZ_r(ctx.as_raw(), raw_coord, line as _, *elem) },
                        #[cfg(feature = "v3_14_0")]
                        3 => errcheck! { GEOSCoordSeq_setM_r(ctx.as_raw(), raw_coord, line as _, *elem) },
                        _ => unreachable!(),
                    }
                    .map_err(|_| {
                        Error::GenericError(format!(
                            "Failed to set value at position {pos} on line {line}"
                        ))
                    })?;
                }
            }
            Ok(coord)
        })
    }

    /// Creates a new `CoordSeq` from an interleaved coordinate buffer.
    ///
    /// # Parameters
    ///
    /// - `data`: The data buffer as a contiguous f64 slice.
    /// - `size`: The number of coordinates in the provided data buffer.
    /// - `coord_type`: The coordinate dimension type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, CoordType::XY)?;
    /// assert_eq!(coords.get_y(1)?, 3.);
    /// assert_eq!(coords.get_x(2)?, 4.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn new_from_buffer(data: &[f64], size: usize, coord_type: CoordType) -> GResult<Self> {
        let coord_dims: u32 = coord_type.into();

        assert_eq!(
            data.len(),
            size * coord_dims as usize,
            "Incorrect buffer length"
        );

        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSCoordSeq_copyFromBuffer_r(
                ctx.as_raw(),
                data.as_ptr(),
                size as _,
                coord_type.has_z().into(),
                coord_type.has_m().into(),
            ))?;
            Ok(Self::new_from_raw(ptr, size as _, coord_type))
        })
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
    /// let coords = CoordSeq::new_from_arrays(&x, &y, None, None)?;
    /// assert_eq!(coords.get_y(1)?, 3.);
    /// assert_eq!(coords.get_x(2)?, 4.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn new_from_arrays(
        x: &[f64],
        y: &[f64],
        z: Option<&[f64]>,
        m: Option<&[f64]>,
    ) -> GResult<Self> {
        assert_eq!(x.len(), y.len(), "Arrays have different lengths.");

        let z_ptr = if let Some(z) = z {
            assert_eq!(x.len(), z.len(), "Arrays have different lengths.");
            z.as_ptr()
        } else {
            std::ptr::null()
        };
        let m_ptr = if let Some(m) = m {
            assert_eq!(x.len(), m.len(), "Arrays have different lengths.");
            m.as_ptr()
        } else {
            std::ptr::null()
        };

        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSCoordSeq_copyFromArrays_r(
                ctx.as_raw(),
                x.as_ptr(),
                y.as_ptr(),
                z_ptr,
                m_ptr,
                x.len() as _,
            ))?;
            let has_z = !z_ptr.is_null();
            let has_m = !m_ptr.is_null();
            Ok(Self::new_from_raw(
                ptr,
                x.len() as u32,
                CoordType::try_from((has_z, has_m))?,
            ))
        })
    }

    pub(crate) const unsafe fn new_from_raw(
        ptr: NonNull<GEOSCoordSequence>,
        size: u32,
        coord_type: CoordType,
    ) -> Self {
        Self {
            ptr,
            size: size as _,
            coord_type,
        }
    }

    /// Sets the X position value at the given `line`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XY)?;
    /// coords.set_x(0, 10.)?;
    /// assert_eq!(coords.get_x(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_x(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.size);

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_setX_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                line as _,
                val
            ))?;
            Ok(())
        })
    }

    /// Sets the Y position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have at least two dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XY)?;
    /// coords.set_y(0, 10.)?;
    /// assert_eq!(coords.get_y(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_y(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.size);

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_setY_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                line as _,
                val
            ))?;
            Ok(())
        })
    }

    /// Sets the Z position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have three dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZ)?;
    /// coords.set_z(0, 10.)?;
    /// assert_eq!(coords.get_z(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_z(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.size);
        assert!(self.coord_type.has_z());

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_setZ_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                line as _,
                val
            ))?;
            Ok(())
        })
    }

    /// Sets the M position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have four dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZM)?;
    /// coords.set_m(0, 10.)?;
    /// assert_eq!(coords.get_m(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn set_m(&mut self, line: usize, val: f64) -> GResult<()> {
        assert!(line < self.size);
        assert!(self.coord_type.has_m());

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_setM_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                line as _,
                val
            ))?;
            Ok(())
        })
    }

    /// Sets the value at the given `ordinate` (aka position).
    ///
    /// Note: your `CoordSeq` object must have enough dimensions to set at the given `ordinate`!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType, Ordinate};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZ)?;
    /// coords.set_ordinate(0, Ordinate::Z, 10.)?;
    /// assert_eq!(coords.get_z(0)?, 10.);
    /// assert_eq!(coords.get_ordinate(0, Ordinate::Z)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn set_ordinate(&mut self, line: usize, ordinate: Ordinate, val: f64) -> GResult<()> {
        let ordinate: u32 = ordinate.into();
        assert!(line < self.size);
        assert!(ordinate <= self.coord_type.into());

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_setOrdinate_r(
                ctx.as_raw(),
                self.as_raw_mut(),
                line as _,
                ordinate,
                val
            ))?;
            Ok(())
        })
    }

    /// Gets the X position value at the given `line`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XY)?;
    /// coords.set_x(0, 10.)?;
    /// assert_eq!(coords.get_x(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_x(&self, line: usize) -> GResult<f64> {
        assert!(line < self.size);

        with_context(|ctx| unsafe {
            let mut n = 0.0;
            errcheck!(GEOSCoordSeq_getX_r(
                ctx.as_raw(),
                self.as_raw(),
                line as _,
                &mut n
            ))?;
            Ok(n)
        })
    }

    /// Gets the Y position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have at least two dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XY)?;
    /// coords.set_y(0, 10.)?;
    /// assert_eq!(coords.get_y(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_y(&self, line: usize) -> GResult<f64> {
        assert!(line < self.size);

        with_context(|ctx| unsafe {
            let mut n = 0.0;
            errcheck!(GEOSCoordSeq_getY_r(
                ctx.as_raw(),
                self.as_raw(),
                line as _,
                &mut n
            ))?;
            Ok(n)
        })
    }

    /// Gets the Z position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have three dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZ)?;
    /// coords.set_z(0, 10.)?;
    /// assert_eq!(coords.get_z(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_z(&self, line: usize) -> GResult<f64> {
        assert!(line < self.size);
        assert!(self.coord_type.has_z());

        with_context(|ctx| unsafe {
            let mut n = 0.0;
            errcheck!(GEOSCoordSeq_getZ_r(
                ctx.as_raw(),
                self.as_raw(),
                line as _,
                &mut n
            ))?;
            Ok(n)
        })
    }

    /// Gets the M position value at the given `line`.
    ///
    /// Note: your `CoordSeq` object must have four dimensions!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZM)?;
    /// coords.set_m(0, 10.)?;
    /// assert_eq!(coords.get_m(0)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_14_0")]
    pub fn get_m(&self, line: usize) -> GResult<f64> {
        assert!(line < self.size);
        assert!(self.coord_type.has_m());

        with_context(|ctx| unsafe {
            let mut n = 0.0;
            errcheck!(GEOSCoordSeq_getM_r(
                ctx.as_raw(),
                self.as_raw(),
                line as _,
                &mut n
            ))?;
            Ok(n)
        })
    }

    /// Gets the entire `CoordSeq` object as an interleaved buffer.
    ///
    /// # Parameters
    ///
    /// - `coord_type`: Optionally, the number of dimensions to include in the output buffer. If `None`,
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
    /// use geos::{CoordSeq, CoordType};
    ///
    /// // Create a two-dimensional `CoordSeq` from a buffer with three points
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, CoordType::XY)?;
    ///
    /// // Return an output buffer, inferring the number of coordinates (2)
    /// let output_buffer = coords.as_buffer(None)?;
    ///
    /// let expected_output_buffer = vec![0., 1., 2., 3., 4., 5.];
    /// assert_eq!(output_buffer, expected_output_buffer);
    /// # Ok::<(), geos::Error>(())
    /// ```
    ///
    /// You can also force GEOS to return a 3D buffer from 2D coordinates
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, CoordType::XY)?;
    ///
    /// // Return an output buffer, forcing it to be 3-dimensional
    /// let output_buffer = coords.as_buffer(Some(CoordType::XYZ))?;
    /// let expected_output_buffer = vec![0., 1., f64::NAN, 2., 3., f64::NAN, 4., 5., f64::NAN];
    /// assert_eq!(output_buffer[0], expected_output_buffer[0]);
    /// assert_eq!(output_buffer[3], expected_output_buffer[3]);
    /// # Ok::<(), geos::Error>(())
    /// ```
    ///
    /// You can also force GEOS to return a 2D buffer from 3D coordinates
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let buffer = vec![0., 1., 100., 2., 3., 200.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 2, CoordType::XYZ)?;
    ///
    /// // Return an output buffer, forcing it to be 2-dimensional
    /// let output_buffer = coords.as_buffer(Some(CoordType::XY))?;
    /// assert_eq!(output_buffer, vec![0., 1., 2., 3.]);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_buffer(&self, coord_type: Option<CoordType>) -> GResult<Vec<f64>> {
        let size = self.size;
        let coord_type = coord_type.unwrap_or(self.coord_type);
        let coord_dims: u32 = coord_type.into();

        with_context(|ctx| unsafe {
            let mut output_buffer = vec![0.; size * coord_dims as usize];
            errcheck!(GEOSCoordSeq_copyToBuffer_r(
                ctx.as_raw(),
                self.as_raw(),
                output_buffer.as_mut_ptr(),
                coord_type.has_z().into(),
                coord_type.has_m().into(),
            ))?;
            Ok(output_buffer)
        })
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
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let buffer = vec![0., 1., 2., 3., 4., 5.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, CoordType::XY)?;
    ///
    /// let output_arrays = coords.as_arrays()?;
    ///
    /// assert_eq!(output_arrays.0, vec![0., 2., 4.]);
    /// assert_eq!(output_arrays.1, vec![1., 3., 5.]);
    /// assert_eq!(output_arrays.2, None);
    /// assert_eq!(output_arrays.3, None);
    /// # Ok::<(), geos::Error>(())
    /// ```
    ///
    /// If the `CoordSeq` contains three dimensions, the third dimension will be returned as the
    /// third value in the tuple:
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let buffer = vec![0., 1., 100., 3., 4., 200., 6., 7., 300.];
    /// let coords = CoordSeq::new_from_buffer(&buffer, 3, CoordType::XYZ)?;
    ///
    /// let output_arrays = coords.as_arrays()?;
    ///
    /// assert_eq!(output_arrays.0, vec![0., 3., 6.]);
    /// assert_eq!(output_arrays.1, vec![1., 4., 7.]);
    /// assert_eq!(output_arrays.2, Some(vec![100., 200., 300.]));
    /// assert_eq!(output_arrays.3, None);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn as_arrays(&self) -> GResult<AsArrayOutput> {
        let size = self.size;
        let mut x = vec![0.; size];
        let mut y = vec![0.; size];
        let mut z = (self.coord_type.has_z()).then(|| vec![0.; size]);
        let mut m = (self.coord_type.has_m()).then(|| vec![0.; size]);

        with_context(|ctx| unsafe {
            errcheck!(GEOSCoordSeq_copyToArrays_r(
                ctx.as_raw(),
                self.as_raw(),
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut().map_or(std::ptr::null_mut(), Vec::as_mut_ptr),
                m.as_mut().map_or(std::ptr::null_mut(), Vec::as_mut_ptr),
            ))
        })?;

        Ok((x, y, z, m))
    }

    /// Gets the value at the given `ordinate` (aka position).
    ///
    /// Note: your `CoordSeq` object must have enough dimensions to access the given `ordinate`!
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType, Ordinate};
    ///
    /// let mut coords = CoordSeq::new(1, CoordType::XYZ)?;
    /// coords.set_ordinate(0, Ordinate::Z, 10.)?;
    /// assert_eq!(coords.get_z(0)?, 10.);
    /// assert_eq!(coords.get_ordinate(0, Ordinate::Z)?, 10.);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn get_ordinate(&self, line: usize, ordinate: Ordinate) -> GResult<f64> {
        let ordinate: u32 = ordinate.into();
        assert!(line < self.size);
        assert!(ordinate <= self.coord_type.into());

        with_context(|ctx| unsafe {
            let mut val = 0.0;
            errcheck!(GEOSCoordSeq_getOrdinate_r(
                ctx.as_raw(),
                self.as_raw(),
                line as _,
                ordinate as _,
                &mut val
            ))?;
            Ok(val)
        })
    }

    /// Returns the number of lines of the `CoordSeq` object.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordSeq, CoordType};
    ///
    /// let coords = CoordSeq::new(2, CoordType::XYZ)?;
    /// assert_eq!(coords.size()?, 2);
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3., 4.], &[5., 6.], &[7., 8.]])?;
    /// assert_eq!(coords.size()?, 4);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn size(&self) -> GResult<usize> {
        with_context(|ctx| unsafe {
            let mut size = 0;
            errcheck!(GEOSCoordSeq_getSize_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut size
            ))?;
            Ok(size as _)
        })
    }

    /// Returns the number of dimensions of the `CoordSeq` object.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, CoordType};
    ///
    /// let coords = CoordSeq::new(2, CoordType::XYZ)?;
    /// assert_eq!(coords.dimensions()?, CoordDimensions::ThreeD);
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3., 4.]])?;
    /// assert_eq!(coords.dimensions()?, CoordDimensions::TwoD);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn dimensions(&self) -> GResult<CoordDimensions> {
        with_context(|ctx| unsafe {
            let mut dims = 0;
            errcheck!(GEOSCoordSeq_getDimensions_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut dims
            ))?;
            CoordDimensions::try_from(dims as i32)
        })
    }

    /// Returns `true` if the geometry has a counter-clockwise orientation.
    ///
    /// Available using the `v3_7_0` feature.
    #[cfg(feature = "v3_7_0")]
    pub fn is_ccw(&self) -> GResult<bool> {
        with_context(|ctx| unsafe {
            let mut is_ccw = 0;
            errcheck!(GEOSCoordSeq_isCCW_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut is_ccw
            ))?;
            Ok(is_ccw == 1)
        })
    }

    /// Returns `true` if `self` has a Z coordinate.
    #[cfg(feature = "v3_14_0")]
    pub fn has_z(&self) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSCoordSeq_hasZ_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
            ))
        })
    }

    /// Returns `true` if `self` has a M coordinate.
    #[cfg(feature = "v3_14_0")]
    pub fn has_m(&self) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSCoordSeq_hasM_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
            ))
        })
    }

    /// Creates a point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.]])?;
    ///
    /// let geom = Geometry::create_point(coords)?;
    ///
    /// assert_eq!(geom.to_wkt()?, "POINT (1 2)");
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn create_point(self) -> GResult<Geometry> {
        Geometry::create_point(self)
    }

    /// Creates a line string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3., 4.]])?;
    ///
    /// let geom = Geometry::create_line_string(coords)?;
    ///
    /// assert_eq!(geom.to_wkt()?, "LINESTRING (1 2, 3 4)");
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn create_line_string(self) -> GResult<Geometry> {
        Geometry::create_line_string(self)
    }

    /// Creates a linear ring geometry.
    pub fn create_linear_ring(self) -> GResult<Geometry> {
        Geometry::create_linear_ring(self)
    }
}

unsafe impl Send for CoordSeq {}
unsafe impl Sync for CoordSeq {}

impl Drop for CoordSeq {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSCoordSeq_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

impl Clone for CoordSeq {
    fn clone(&self) -> Self {
        let ptr = with_context(|ctx| unsafe {
            nullcheck!(GEOSCoordSeq_clone_r(ctx.as_raw(), self.as_raw()))
                .expect("GEOSCoordSeq_clone_r failed")
        });
        Self {
            ptr,
            size: self.size,
            coord_type: self.coord_type,
        }
    }
}
