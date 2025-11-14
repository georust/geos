use crate::context_handle::with_context;
use crate::enums::*;
use crate::error::{Error, GResult};
use crate::functions::*;
use crate::traits::{as_raw_impl, as_raw_mut_impl};
#[cfg(any(feature = "v3_10_0", feature = "dox"))]
use crate::GeoJSONWriter;
#[cfg(any(feature = "v3_10_0", feature = "dox"))]
use crate::MakeValidParams;
#[cfg(any(feature = "v3_6_0", feature = "dox"))]
use crate::Precision;
use crate::{AsRaw, AsRawMut, BufferParams, CoordSeq, PreparedGeometry, WKTWriter};
use geos_sys::*;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::{self, str};

/// Representation of a GEOS geometry.
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry};
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 3.5)")
///                           .expect("Invalid geometry");
/// assert_eq!(point_geom.get_x(), Ok(2.5));
/// assert_eq!(point_geom.get_y(), Ok(3.5));
/// ```
pub struct Geometry {
    pub(crate) ptr: NonNull<GEOSGeometry>,
}

// Representation of a GEOS geometry. Since it's only a view over another GEOS geometry data,
/// only not mutable operations are implemented on it.
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry};
///
/// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
///                                            (1 1, 2 1, 2 5, 1 5, 1 1),\
///                                            (8 5, 8 4, 9 4, 9 5, 8 5))")
///                    .expect("Invalid geometry");
/// let point_geom = geom
///     .get_interior_ring_n(0)
///     .expect("failed to get const geometry");
/// ```
pub struct ConstGeometry<'a> {
    pub(crate) ptr: NonNull<GEOSGeometry>,
    phantom: PhantomData<&'a Geometry>,
}

unsafe impl Send for Geometry {}
unsafe impl Sync for Geometry {}

unsafe impl Send for ConstGeometry<'_> {}
unsafe impl Sync for ConstGeometry<'_> {}

impl Geom for Geometry {}
impl Geom for ConstGeometry<'_> {}

pub trait Geom: AsRaw<RawType = GEOSGeometry> + Sized + Send + Sync {
    /// Returns the type of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_type(), Ok("Polygon".to_owned()));
    /// ```
    fn get_type(&self) -> GResult<String> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomType_r(ctx.as_raw(), self.as_raw()))?;
            managed_string(ptr, ctx)
        })
    }

    fn geometry_type(&self) -> GResult<GeometryTypes> {
        with_context(|ctx| unsafe {
            let geom_type = errcheck!(-1, GEOSGeomTypeId_r(ctx.as_raw(), self.as_raw()))?;
            GeometryTypes::try_from(geom_type)
        })
    }

    /// Checks if the geometry is valid.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert!(geom.is_valid() == Ok(false));
    /// ```
    fn is_valid(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSisValid_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns an explanation on why the geometry is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// // Bowtie polygon with self-intersection
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 2 2, 2 0, 0 2, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(
    ///     geom.is_valid_reason(),
    ///     Ok("Self-intersection[1 1]".to_owned()),
    /// );
    /// ```
    fn is_valid_reason(&self) -> GResult<String> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSisValidReason_r(ctx.as_raw(), self.as_raw()))?;
            managed_string(ptr, ctx)
        })
    }

    /// Get the underlying geos CoordSeq object from the geometry
    ///
    /// Note: this clones the underlying CoordSeq to avoid double free
    /// (because CoordSeq handles the object ptr and the CoordSeq is still owned by the geos
    /// geometry) if this method's performance becomes a bottleneck, feel free to open an issue,
    /// we could skip this clone with cleaner code.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (2 3)")
    ///                     .expect("Invalid geometry");
    /// let coord_seq = geom.get_coord_seq().expect("get_coord_seq failed");
    ///
    /// assert_eq!(coord_seq.get_x(0), Ok(2.));
    /// assert_eq!(coord_seq.get_y(0), Ok(3.));
    /// ```
    fn get_coord_seq(&self) -> GResult<CoordSeq> {
        with_context(|ctx| unsafe {
            let coords = nullcheck!(GEOSGeom_getCoordSeq_r(ctx.as_raw(), self.as_raw()))?;
            let coords = nullcheck!(GEOSCoordSeq_clone_r(ctx.as_raw(), coords.as_ptr()))?;
            let mut size = 0;
            let mut dims = 0;

            errcheck!(GEOSCoordSeq_getSize_r(
                ctx.as_raw(),
                coords.as_ptr(),
                &mut size
            ))?;
            errcheck!(GEOSCoordSeq_getDimensions_r(
                ctx.as_raw(),
                coords.as_ptr(),
                &mut dims
            ))?;

            Ok(CoordSeq::new_from_raw(coords, size, dims))
        })
    }

    /// Returns the area of the geometry. Units are specified by the SRID of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// assert_eq!(geom1.area(), Ok(60.));
    /// ```
    fn area(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut n = 0.;
            errcheck!(GEOSArea_r(ctx.as_raw(), self.as_raw(), &mut n))?;
            Ok(n)
        })
    }

    /// Returns a WKT representation of the geometry. It defaults to 2 dimensions output. Use
    /// [`WKTWriter`] type directly if you want more control.
    ///
    /// # Examples
    ///
    /// ```
    /// use geos::{Geom, Geometry, OutputDimension, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     point_geom.to_wkt().unwrap(),
    ///     "POINT (2.5000000000000000 2.5000000000000000)",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     point_geom.to_wkt().unwrap(),
    ///     "POINT (2.5 2.5)",
    /// );
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 3)")
    ///                           .expect("Invalid geometry");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     point_geom.to_wkt().unwrap(),
    ///     "POINT (2.5000000000000000 2.5000000000000000)",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     point_geom.to_wkt().unwrap(),
    ///     "POINT Z (2.5 2.5 3)",
    /// );
    /// ```
    fn to_wkt(&self) -> GResult<String> {
        WKTWriter::new()?.write(self)
    }

    /// Returns a WKT representation of the geometry with the given `precision`. It is a wrapper
    /// around [`WKTWriter::set_rounding_precision`].
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.53 2.54)").expect("Invalid geometry");
    /// assert_eq!(point_geom.to_wkt_precision(2).unwrap(), "POINT (2.53 2.54)");
    ///
    /// // It is a wrapper around:
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    /// writer.set_rounding_precision(2);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.53 2.54)");
    /// ```
    fn to_wkt_precision(&self, precision: u32) -> GResult<String> {
        with_context(|ctx| unsafe {
            let writer = nullcheck!(GEOSWKTWriter_create_r(ctx.as_raw()))?.as_ptr();
            GEOSWKTWriter_setRoundingPrecision_r(ctx.as_raw(), writer, precision as _);
            let c_result = nullcheck!(GEOSWKTWriter_write_r(ctx.as_raw(), writer, self.as_raw()))?;
            GEOSWKTWriter_destroy_r(ctx.as_raw(), writer);
            managed_string(c_result, ctx)
        })
    }

    /// Returns `true` if the geometry is a ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let circle = Geometry::new_from_wkt("LINESTRING(0 0, 0 1, 1 1, 0 0)").expect("Invalid geometry");
    /// assert_eq!(circle.is_ring(), Ok(true));
    /// ```
    fn is_ring(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSisRing_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns `true` if `self` shares any portion of space with `other`. So if any of this is
    /// `true`:
    ///
    /// * `self` overlaps `other`
    /// * `self` touches `other`
    /// * `self` is within `other`
    ///
    /// Then `intersects` will return `true` as well.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.intersects(&geom2), Ok(false));
    /// assert_eq!(geom1.intersects(&geom3), Ok(true));
    /// ```
    fn intersects<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSIntersects_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if `self` and `other` have at least one interior into each other.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(1 1,2 2)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 1,1 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.crosses(&geom2), Ok(true));
    /// ```
    fn crosses<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSCrosses_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if `self` doesn't:
    ///
    /// * Overlap `other`
    /// * Touch `other`
    /// * Is within `other`
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.disjoint(&geom2), Ok(true));
    /// assert_eq!(geom1.disjoint(&geom3), Ok(false));
    /// ```
    fn disjoint<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSDisjoint_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if the only points in common between `self` and `other` lie in the union of
    /// the boundaries of `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(0 0, 1 1, 0 2)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT(1 1)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.touches(&geom2), Ok(false));
    ///
    /// let geom2 = Geometry::new_from_wkt("POINT(0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.touches(&geom2), Ok(true));
    /// ```
    fn touches<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSTouches_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if `self` spatially overlaps `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 0.5)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(1 0, 1 1, 3 5)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.overlaps(&geom2), Ok(false));
    ///
    /// let geom1 = geom1.buffer(3., 8).expect("buffer failed");
    /// let geom2 = geom2.buffer(0.5, 8).expect("buffer failed");
    ///
    /// assert_eq!(geom1.overlaps(&geom2), Ok(true));
    /// ```
    fn overlaps<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSOverlaps_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if `self` is completely inside `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(50 50)").expect("invalid geometry");
    /// let small_geom = geom.buffer(20., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(40., 8).expect("buffer failed");
    ///
    /// assert_eq!(small_geom.within(&small_geom), Ok(true));
    /// assert_eq!(small_geom.within(&big_geom), Ok(true));
    /// assert_eq!(big_geom.within(&small_geom), Ok(false));
    /// ```
    fn within<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSWithin_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Checks if the two [`Geometry`] objects are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1.equals(&geom2) == Ok(false));
    /// assert!(geom1.equals(&geom3) == Ok(true));
    /// ```
    ///
    /// Note that you can also use method through the `PartialEq` trait:
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1 != geom2);
    /// assert!(geom1 == geom3);
    /// ```
    fn equals<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSEquals_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Checks if the two [`Geometry`] objects are exactly equal.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.equals_exact(&geom2, 0.1), Ok(false));
    /// assert_eq!(geom1.equals_exact(&geom3, 0.1), Ok(true));
    /// ```
    fn equals_exact<G: Geom>(&self, other: &G, precision: f64) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSEqualsExact_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                precision
            ))
        })
    }

    #[cfg(any(feature = "v3_12_0", feature = "dox"))]
    fn equals_identical<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSEqualsIdentical_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if no point of `other` is outside of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// assert_eq!(little_geom.covers(&big_geom), Ok(false));
    /// assert_eq!(big_geom.covers(&little_geom), Ok(true));
    /// ```
    fn covers<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSCovers_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if no point of `self` is outside of `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// assert_eq!(little_geom.covered_by(&big_geom), Ok(true));
    /// assert_eq!(big_geom.covered_by(&little_geom), Ok(false));
    /// ```
    fn covered_by<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSCoveredBy_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    /// Returns `true` if no points of the `other` geometry is outside the exterior of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.contains(&geom2), Ok(true));
    /// ```
    fn contains<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSContains_r(ctx.as_raw(), self.as_raw(), other.as_raw()))
        })
    }

    fn relate<G: Geom>(&self, other: &G) -> GResult<String> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSRelate_r(ctx.as_raw(), self.as_raw(), other.as_raw()))?;
            managed_string(ptr, ctx)
        })
    }

    fn relate_pattern<G: Geom>(&self, other: &G, pattern: &str) -> GResult<bool> {
        with_context(|ctx| unsafe {
            let pattern = CString::new(pattern)
                .map_err(|e| Error::GenericError(format!("Conversion to CString failed: {e}")))?;
            predicate!(GEOSRelatePattern_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                pattern.as_ptr()
            ))
        })
    }

    /// Returns a geometry which represents all points whose distance from `self` is less than or
    /// equal to distance.
    ///
    /// You can find nice examples about this in [postgis](https://postgis.net/docs/ST_Buffer.html)
    /// documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let buffer_geom = geom.buffer(50., 2).expect("buffer failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((51.0 3.0, 36.4 -32.4, 1.0 -47.0, -34.4 -32.4, -49.0 3.0, -34.4 38.4, \
    ///                       1.0 53.0, 36.4 38.4, 51.0 3.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((51 3, 36.4 -32.4, 1 -47, -34.4 -32.4, -49 3, -34.4 38.4, 1 53, 36.4 \
    ///                       38.4, 51 3))");
    /// ```
    fn buffer(&self, width: f64, quadsegs: i32) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBuffer_r(
                ctx.as_raw(),
                self.as_raw(),
                width,
                quadsegs as _
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a geometry which represents all points whose distance from `self` is less than or
    /// equal to distance.
    ///
    /// The explicit `buffer_params` argument passing is more efficient than
    /// the otherwise identical (except for the `single_sided` option is missing)
    /// [`buffer_with_style`](crate::Geom::buffer_with_style) method when the same [`BufferParams`]
    /// is reused.
    ///
    /// You can find nice examples and details about the [`BufferParams`] options in this
    /// [postgis](https://postgis.net/docs/ST_Buffer.html) documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{BufferParams, CapStyle, Geom, Geometry, JoinStyle};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let params = BufferParams::builder()
    ///     .end_cap_style(CapStyle::Round)
    ///     .join_style(JoinStyle::Round)
    ///     .mitre_limit(5.0)
    ///     .quadrant_segments(8)
    ///     .single_sided(false)
    ///     .build()
    ///     .expect("build BufferParams");
    /// let buffer_geom = geom.buffer_with_params(2., &params).expect("buffer_with_params failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((3.0 3.0, 3.0 2.6, 2.8 2.2, 2.7 1.9, 2.4 1.6, 2.1 1.3, 1.8 1.2, 1.4 1.0, \
    ///              1.0 1.0, 0.6 1.0, 0.2 1.2, -0.1 1.3, -0.4 1.6, -0.7 1.9, -0.8 2.2, -1.0 2.6, \
    ///              -1.0 3.0, -1.0 3.4, -0.8 3.8, -0.7 4.1, -0.4 4.4, -0.1 4.7, 0.2 4.8, 0.6 5.0, \
    ///              1.0 5.0, 1.4 5.0, 1.8 4.8, 2.1 4.7, 2.4 4.4, 2.7 4.1, 2.8 3.8, 3.0 3.4, \
    ///              3.0 3.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((3 3, 3 2.6, 2.8 2.2, 2.7 1.9, 2.4 1.6, 2.1 1.3, 1.8 1.2, 1.4 1, 1 1, \
    ///              0.6 1, 0.2 1.2, -0.1 1.3, -0.4 1.6, -0.7 1.9, -0.8 2.2, -1 2.6, -1 3, -1 3.4, \
    ///              -0.8 3.8, -0.7 4.1, -0.4 4.4, -0.1 4.7, 0.2 4.8, 0.6 5, 1 5, 1.4 5, 1.8 4.8, \
    ///              2.1 4.7, 2.4 4.4, 2.7 4.1, 2.8 3.8, 3 3.4, 3 3))");
    /// ```
    fn buffer_with_params(&self, width: f64, buffer_params: &BufferParams) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBufferWithParams_r(
                ctx.as_raw(),
                self.as_raw(),
                buffer_params.as_raw(),
                width
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a geometry which represents all points whose distance from `self` is less than or
    /// equal to distance.
    ///
    /// If the same paramters are used many times it's more efficient to use the
    /// [`buffer_with_params`](crate::Geom::buffer_with_params) operation.
    ///
    /// You can find nice examples and details about the options in this
    /// [postgis](https://postgis.net/docs/ST_Buffer.html) documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CapStyle, Geom, Geometry, JoinStyle};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let buffer_geom = geom.buffer_with_style(
    ///     2., 8, CapStyle::Round, JoinStyle::Round, 5.
    /// ).expect("buffer_with_style failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((3.0 3.0, 3.0 2.6, 2.8 2.2, 2.7 1.9, 2.4 1.6, 2.1 1.3, 1.8 1.2, \
    ///             1.4 1.0, 1.0 1.0, 0.6 1.0, 0.2 1.2, -0.1 1.3, -0.4 1.6, -0.7 1.9, -0.8 2.2, \
    ///             -1.0 2.6, -1.0 3.0, -1.0 3.4, -0.8 3.8, -0.7 4.1, -0.4 4.4, -0.1 4.7, 0.2 4.8, \
    ///             0.6 5.0, 1.0 5.0, 1.4 5.0, 1.8 4.8, 2.1 4.7, 2.4 4.4, 2.7 4.1, 2.8 3.8, \
    ///             3.0 3.4, 3.0 3.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((3 3, 3 2.6, 2.8 2.2, 2.7 1.9, 2.4 1.6, 2.1 1.3, 1.8 1.2, 1.4 1, 1 1, \
    ///             0.6 1, 0.2 1.2, -0.1 1.3, -0.4 1.6, -0.7 1.9, -0.8 2.2, -1 2.6, -1 3, -1 3.4, \
    ///             -0.8 3.8, -0.7 4.1, -0.4 4.4, -0.1 4.7, 0.2 4.8, 0.6 5, 1 5, 1.4 5, 1.8 4.8, \
    ///             2.1 4.7, 2.4 4.4, 2.7 4.1, 2.8 3.8, 3 3.4, 3 3))");
    /// ```
    fn buffer_with_style(
        &self,
        width: f64,
        quadsegs: i32,
        end_cap_style: CapStyle,
        join_style: JoinStyle,
        mitre_limit: f64,
    ) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBufferWithStyle_r(
                ctx.as_raw(),
                self.as_raw(),
                width,
                quadsegs,
                end_cap_style.into(),
                join_style.into(),
                mitre_limit
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns `true` if the given geometry is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_polygon().expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON EMPTY").expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(false));
    /// ```
    fn is_empty(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSisEmpty_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns true if the given geometry has no anomalous geometric points, such as self
    /// intersection or self tangency.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_simple(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 1,2 2,2 3.5,1 3,1 2,2 1)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_simple(), Ok(false));
    /// ```
    fn is_simple(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSisSimple_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns a geometry which represents part of `self` that doesn't intersect with `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(50 100, 50 200)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(50 50, 50 150)").expect("Invalid geometry");
    ///
    /// let difference_geom = geom1.difference(&geom2).expect("envelope failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(difference_geom.to_wkt_precision(1).unwrap(),
    ///            "LINESTRING (50.0 150.0, 50.0 200.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(difference_geom.to_wkt().unwrap(), "LINESTRING (50 150, 50 200)");
    /// ```
    fn difference<G: Geom>(&self, other: &G) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSDifference_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_9_0", feature = "dox"))]
    fn difference_prec<G: Geom>(&self, other: &G, grid_size: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSDifferencePrec_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                grid_size
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the minimum bouding box of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let envelope_geom = geom.envelope().expect("envelope failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(envelope_geom.to_wkt_precision(1).unwrap(), "POINT (1.0 3.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(envelope_geom.to_wkt().unwrap(), "POINT (1 3)");
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 1 3)").expect("Invalid geometry");
    /// let envelope_geom = geom.envelope().expect("envelope failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(envelope_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((0.0 0.0, 1.0 0.0, 1.0 3.0, 0.0 3.0, 0.0 0.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(envelope_geom.to_wkt().unwrap(), "POLYGON ((0 0, 1 0, 1 3, 0 3, 0 0))");
    /// ```
    fn envelope(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSEnvelope_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a geometry which represents the parts of `self` and `other` that don't intersect.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(50 100, 50 200)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(50 50, 50 150)").expect("Invalid geometry");
    ///
    /// let sym_diff_geom = geom1.sym_difference(&geom2).expect("sym_difference failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     sym_diff_geom.to_wkt_precision(1).unwrap(),
    ///     "MULTILINESTRING ((50.0 150.0, 50.0 200.0), (50.0 50.0, 50.0 100.0))",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     sym_diff_geom.to_wkt().unwrap(),
    ///     "MULTILINESTRING ((50 150, 50 200), (50 50, 50 100))",
    /// );
    /// ```
    fn sym_difference<G: Geom>(&self, other: &G) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSSymDifference_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_9_0", feature = "dox"))]
    fn sym_difference_prec<G: Geom>(&self, other: &G, grid_size: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSSymDifferencePrec_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                grid_size
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Aggregates the given geometry with another one.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT(3 4)").expect("Invalid geometry");
    ///
    /// let union_geom = geom1.union(&geom2).expect("union failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(union_geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 2.0, 3.0 4.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(union_geom.to_wkt().unwrap(), "MULTIPOINT ((1 2), (3 4))");
    /// ```
    fn union<G: Geom>(&self, other: &G) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSUnion_r(ctx.as_raw(), self.as_raw(), other.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_9_0", feature = "dox"))]
    fn union_prec<G: Geom>(&self, other: &G, grid_size: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSUnionPrec_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                grid_size
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the geometric center or (equivalently) the center of mass of the given geometry as
    /// a point.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("MULTIPOINT(-1 0, -1 2, -1 3, -1 4, -1 7, 0 1, 0 3, 1 1)")
    ///                     .expect("Invalid geometry");
    /// let centroid = geom.get_centroid().expect("failed to get centroid");
    ///
    /// assert_eq!(centroid.to_wkt_precision(1).unwrap(), "POINT (-0.5 2.6)");
    /// ```
    fn get_centroid(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGetCentroid_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_UnaryUnion.html):
    ///
    /// > Unlike ST_Union, ST_UnaryUnion does dissolve boundaries between components of a
    /// > multipolygon (invalid) and does perform union between the components of a
    /// > geometrycollection. Each components of the input geometry is assumed to be valid, so you
    /// > won't get a valid multipolygon out of a bow-tie polygon (invalid).
    /// >
    /// > You may use this function to node a set of linestrings. You may mix ST_UnaryUnion with
    /// > ST_Collect to fine-tune how many geometries at once you want to dissolve to be nice on
    /// > both memory size and CPU time, finding the balance between ST_Union and ST_MemUnion.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((1 1, 2 1, 2 5, 1 5, 1 1))")
    ///                      .expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipolygon(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// let mut union_geom = geom.unary_union().expect("unary_union failed");
    /// union_geom.normalize().expect("normalize failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     union_geom.to_wkt_precision(1).unwrap(),
    ///     "POLYGON ((0.0 0.0, 0.0 6.0, 10.0 6.0, 10.0 0.0, 0.0 0.0))",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     union_geom.to_wkt().unwrap(),
    ///     "POLYGON ((0 0, 0 6, 10 6, 10 0, 0 0))",
    /// );
    /// ```
    fn unary_union(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSUnaryUnion_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_9_0", feature = "dox"))]
    fn unary_union_prec(&self, grid_size: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSUnaryUnionPrec_r(ctx.as_raw(), self.as_raw(), grid_size))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_8_0", feature = "dox"))]
    fn coverage_union(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSCoverageUnion_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_12_0", feature = "dox"))]
    fn disjoint_subset_union(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSDisjointSubsetUnion_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Create a voronoi diagram.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let input = Geometry::new_from_wkt("MULTIPOINT(2 2, 4 2)")
    ///                   .expect("Invalid geometry");
    /// let mut expected = Geometry::new_from_wkt(
    ///     "GEOMETRYCOLLECTION (POLYGON ((0 0, 0 4, 3 4, 3 0, 0 0)), POLYGON ((6 4, 6 0, 3 0, 3 4, 6 4)))")
    ///     .expect("Invalid geometry");
    ///
    /// let mut voronoi = input.voronoi(None::<&Geometry>, 0., false)
    ///                        .expect("voronoi failed");
    ///
    /// expected.normalize().expect("normalize failed");
    /// voronoi.normalize().expect("normalize failed");
    ///
    /// assert_eq!(expected.equals(&voronoi), Ok(true));
    /// ```
    fn voronoi<G: Geom>(
        &self,
        envelope: Option<&G>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSVoronoiDiagram_r(
                ctx.as_raw(),
                self.as_raw(),
                envelope.map_or(std::ptr::null_mut(), |e| e.as_raw()),
                tolerance,
                only_edges.into(),
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a geometry representing the intersection between `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let mut geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("Invalid geometry");
    /// let mut geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("Invalid geometry");
    ///
    /// let intersection_geom = geom1.intersection(&geom2).expect("intersection failed");
    ///
    /// // No intersection.
    /// assert_eq!(intersection_geom.is_empty(), Ok(true));
    ///
    /// // We slighty change the linestring so we have an intersection:
    /// let mut geom2 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("Invalid geometry");
    ///
    /// let intersection_geom = geom1.intersection(&geom2).expect("intersection failed");
    ///
    /// // Intersection!
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(intersection_geom.to_wkt_precision(1).unwrap(), "POINT (0.0 0.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(intersection_geom.to_wkt().unwrap(), "POINT (0 0)");
    /// ```
    fn intersection<G: Geom>(&self, other: &G) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSIntersection_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_9_0", feature = "dox"))]
    fn intersection_prec<G: Geom>(&self, other: &G, grid_size: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSIntersectionPrec_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                grid_size
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_ConvexHull.html):
    ///
    /// > The convex hull of a geometry represents the minimum convex geometry that encloses all
    /// > geometries within the set.
    /// >
    /// > One can think of the convex hull as the geometry you get by wrapping an elastic band
    /// > around a set of geometries. This is different from a concave hull which is analogous to
    /// > shrink-wrapping your geometries.
    /// >
    /// > It is usually used with MULTI and Geometry Collections. Although it is not an aggregate -
    /// > you can use it in conjunction with ST_Collect to get the convex hull of a set of points.
    /// > ST_ConvexHull(ST_Collect(somepointfield)).
    /// >
    /// > It is often used to determine an affected area based on a set of point observations.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let mut geom1 = Geometry::new_from_wkt("MULTILINESTRING((100 190,10 8),
    ///                                                         (150 10, 20 30))")
    ///                          .expect("Invalid geometry");
    /// let mut geom2 = Geometry::new_from_wkt("MULTIPOINT(50 5, 150 30, 50 10, 10 10)")
    ///                          .expect("Invalid geometry");
    ///
    /// let geom = geom1.union(&geom2).expect("union failed");
    /// let convex_hull_geom = geom.convex_hull().expect("convex_hull failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(convex_hull_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((50.0 5.0, 10.0 8.0, 10.0 10.0, 100.0 190.0, 150.0 30.0, 150.0 10.0, 50.0 5.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(convex_hull_geom.to_wkt().unwrap(),
    ///            "POLYGON ((50 5, 10 8, 10 10, 100 190, 150 30, 150 10, 50 5))");
    /// ```
    fn convex_hull(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSConvexHull_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the closure of the combinatorial boundary of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 1,0 0, -1 1)").expect("Invalid geometry");
    /// let boundary_geom = geom.boundary().expect("boundary failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(boundary_geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 1.0, -1.0 1.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(boundary_geom.to_wkt().unwrap(), "MULTIPOINT ((1 1), (-1 1))");
    /// ```
    fn boundary(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBoundary_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns `true` if `self` has a Z coordinate.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2 3)").expect("Invalid geometry");
    /// assert_eq!(geom.has_z(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2)").expect("Invalid geometry");
    /// assert_eq!(geom.has_z(), Ok(false));
    /// ```
    fn has_z(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSHasZ_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns `true` if `self` has a M coordinate.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2 3 4)").expect("Invalid geometry");
    /// assert_eq!(geom.has_m(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2 3)").expect("Invalid geometry");
    /// assert_eq!(geom.has_m(), Ok(false));
    /// ```
    #[cfg(any(feature = "v3_12_0", feature = "dox"))]
    fn has_m(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSHasM_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns `true` if start and end point are coincident.
    ///
    /// Only works on `LineString`, `LinearRing`, `CircularString`, `MultiLineString` and `MultiCurve`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 1 1)").expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(false));
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 0 1, 1 1, 0 0)").expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("MULTILINESTRING((0 0, 0 1, 1 1, 0 0),(0 0, 1 1))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(false));
    /// ```
    fn is_closed(&self) -> GResult<bool> {
        with_context(|ctx| unsafe { predicate!(GEOSisClosed_r(ctx.as_raw(), self.as_raw())) })
    }

    /// Returns the length of `self`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(743238 2967416,743238 2967450)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(
    ///     geom.length().map(|x| format!("{:.2}", x)).unwrap(),
    ///     "34.00",
    /// );
    /// ```
    fn length(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut length = 0.0;
            errcheck!(GEOSLength_r(ctx.as_raw(), self.as_raw(), &mut length))?;
            Ok(length)
        })
    }

    /// Returns the distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    fn distance<G: Geom>(&self, other: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSDistance_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance
            ))?;
            Ok(distance)
        })
    }

    /// Returns `true` if the distance between `self` and `other` is shorter than `distance`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (3 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.dwithin(&geom2, 1.0), Ok(true));
    /// assert_eq!(geom1.dwithin(&geom3, 1.0), Ok(false));
    /// ```
    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    fn dwithin<G: Geom>(&self, other: &G, distance: f64) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSDistanceWithin_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance
            ))
        })
    }

    /// Returns the indexed distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.distance_indexed(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn distance_indexed<G: Geom>(&self, other: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSDistanceIndexed_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance
            ))?;
            Ok(distance)
        })
    }

    /// Returns the hausdorff distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.hausdorff_distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    fn hausdorff_distance<G: Geom>(&self, other: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSHausdorffDistance_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance
            ))?;
            Ok(distance)
        })
    }

    /// Returns the hausdorff distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.hausdorff_distance_densify(&geom2, 1.).map(|x| format!("{:.2}", x))
    ///                                                        .unwrap(), "1.00");
    /// ```
    fn hausdorff_distance_densify<G: Geom>(&self, other: &G, distance_frac: f64) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSHausdorffDistanceDensify_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance
            ))?;
            Ok(distance)
        })
    }

    /// Returns the frechet distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (0 0, 100 0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (0 0, 50 50, 100 0)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.frechet_distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "70.71");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn frechet_distance<G: Geom>(&self, other: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSFrechetDistance_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance
            ))?;
            Ok(distance)
        })
    }

    /// Returns the frechet distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (0 0, 100 0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (0 0, 50 50, 100 0)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.frechet_distance_densify(&geom2, 1.).map(|x| format!("{:.2}", x))
    ///                                                      .unwrap(), "70.71");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn frechet_distance_densify<G: Geom>(&self, other: &G, distance_frac: f64) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSFrechetDistanceDensify_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance,
            ))?;
            Ok(distance)
        })
    }

    /// Returns the length of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4, 5 6)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_length().map(|x| format!("{:.2}", x)).unwrap(), "5.66");
    /// ```
    fn get_length(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut length = 0.0;
            errcheck!(GEOSGeomGetLength_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut length
            ))?;
            Ok(length)
        })
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_Snap.html):
    ///
    /// >  Snaps the vertices and segments of a geometry another Geometry's vertices. A snap
    /// > distance tolerance is used to control where snapping is performed. The result geometry is
    /// > the input geometry with the vertices snapped. If no snapping occurs then the input
    /// > geometry is returned unchanged.
    /// >
    /// > Snapping one geometry to another can improve robustness for overlay operations by
    /// > eliminating nearly-coincident edges (which cause problems during noding and intersection
    /// > calculation).
    /// >
    /// > Too much snapping can result in invalid topology being created, so the number and location
    /// > of snapped vertices is decided using heuristics to determine when it is safe to snap. This
    /// > can result in some potential snaps being omitted, however.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("MULTIPOLYGON(((26 125, 26 200, 126 200, 126 125, 26 125),
    ///                                                   (51 150, 101 150, 76 175, 51 150)),
    ///                                                  ((151 100, 151 200, 176 175, 151 100)))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(5 107, 54 84, 101 100)")
    ///                      .expect("Invalid geometry");
    ///
    /// let distance = geom1.distance(&geom2).expect("distance failed");
    /// let snap_geom = geom1.snap(&geom2, distance * 1.25).expect("snap failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(snap_geom.to_wkt_precision(1).unwrap(),
    ///            "MULTIPOLYGON (((5.0 107.0, 26.0 200.0, 126.0 200.0, 126.0 125.0, 101.0 100.0, 54.0 84.0, 5.0 107.0), \
    ///                            (51.0 150.0, 101.0 150.0, 76.0 175.0, 51.0 150.0)), \
    ///                           ((151.0 100.0, 151.0 200.0, 176.0 175.0, 151.0 100.0)))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(snap_geom.to_wkt().unwrap(),
    ///            "MULTIPOLYGON (((5 107, 26 200, 126 200, 126 125, 101 100, 54 84, 5 107), \
    ///                            (51 150, 101 150, 76 175, 51 150)), \
    ///                           ((151 100, 151 200, 176 175, 151 100)))");
    /// ```
    fn snap<G: Geom>(&self, other: &G, tolerance: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSSnap_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                tolerance,
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns unique points of `self`.
    fn extract_unique_points(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_extractUniquePoints_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    fn nearest_points<G: Geom>(&self, other: &G) -> GResult<CoordSeq> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSNearestPoints_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
            ))?;
            let mut size = 0;
            let mut dims = 0;

            errcheck!(GEOSCoordSeq_getSize_r(
                ctx.as_raw(),
                ptr.as_ptr(),
                &mut size
            ))?;
            errcheck!(GEOSCoordSeq_getDimensions_r(
                ctx.as_raw(),
                ptr.as_ptr(),
                &mut dims
            ))?;
            Ok(CoordSeq::new_from_raw(ptr, size, dims))
        })
    }

    /// Returns the X position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_x() == Ok(1.5));
    /// ```
    fn get_x(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut x = 0.;
            errcheck!(GEOSGeomGetX_r(ctx.as_raw(), self.as_raw(), &mut x))?;
            Ok(x)
        })
    }

    /// Returns the Y position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_y() == Ok(2.5));
    /// ```
    fn get_y(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut x = 0.;
            errcheck!(GEOSGeomGetY_r(ctx.as_raw(), self.as_raw(), &mut x))?;
            Ok(x)
        })
    }

    /// Returns the Z position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_z() == Ok(4.0));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn get_z(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut x = 0.;
            errcheck!(GEOSGeomGetZ_r(ctx.as_raw(), self.as_raw(), &mut x))?;
            Ok(x)
        })
    }

    /// Returns the M position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// Available using the `v3_12_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0 5.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_m() == Ok(5.0));
    /// ```
    #[cfg(any(feature = "v3_12_0", feature = "dox"))]
    fn get_m(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut m = 0.;
            errcheck!(GEOSGeomGetM_r(ctx.as_raw(), self.as_raw(), &mut m))?;
            Ok(m)
        })
    }

    /// Returns the nth point of the given geometry.
    ///
    /// The given `Geometry` must be a `LineString`, `LinearRing` or `CircularString` otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4, 5 6)")
    ///                     .expect("Invalid geometry");
    /// let nth_point = geom.get_point_n(1).expect("get_point_n failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(nth_point.to_wkt_precision(1).unwrap(), "POINT (3.0 4.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(nth_point.to_wkt().unwrap(), "POINT (3 4)");
    /// ```
    fn get_point_n(&self, n: usize) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomGetPointN_r(ctx.as_raw(), self.as_raw(), n as _))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the start point of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, `LinearRing` or `CircularString` otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    /// let start_point = geom.get_start_point().expect("get_start_point failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(start_point.to_wkt_precision(1).unwrap(), "POINT (1.0 2.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(start_point.to_wkt().unwrap(), "POINT (1 2)");
    /// ```
    fn get_start_point(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomGetStartPoint_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the end point of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, `LinearRing` or `CircularString` otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    /// let end_point = geom.get_end_point().expect("get_end_point failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(end_point.to_wkt_precision(1).unwrap(), "POINT (3.0 4.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(end_point.to_wkt().unwrap(), "POINT (3 4)");
    /// ```
    fn get_end_point(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomGetEndPoint_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the number of points of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, `LinearRing` or `CircularString` otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_points(), Ok(2));
    /// ```
    fn get_num_points(&self) -> GResult<usize> {
        with_context(|ctx| unsafe {
            let ret = errcheck!(-1, GEOSGeomGetNumPoints_r(ctx.as_raw(), self.as_raw()))?;
            Ok(ret as _)
        })
    }

    /// Returns the number of interior rings.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                            (1 1, 2 1, 2 5, 1 5, 1 1),\
    ///                                            (8 5, 8 4, 9 4, 9 5, 8 5))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_interior_rings(), Ok(2));
    /// ```
    fn get_num_interior_rings(&self) -> GResult<usize> {
        with_context(|ctx| unsafe {
            let ret = errcheck!(-1, GEOSGetNumInteriorRings_r(ctx.as_raw(), self.as_raw()))?;
            Ok(ret as _)
        })
    }

    /// Returns the number of coordinates inside `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_coordinates(), Ok(5));
    /// ```
    fn get_num_coordinates(&self) -> GResult<usize> {
        with_context(|ctx| unsafe {
            let ret = errcheck!(-1, GEOSGetNumCoordinates_r(ctx.as_raw(), self.as_raw()))?;
            Ok(ret as _)
        })
    }

    /// Returns the number of dimensions used in `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_dimensions(), Ok(2));
    /// ```
    fn get_num_dimensions(&self) -> GResult<i32> {
        with_context(|ctx| unsafe {
            // Need to skip errcheck as 0 is a valid return values
            // TODO: file a bug report to GEOS
            let ret = GEOSGeom_getDimensions_r(ctx.as_raw(), self.as_raw());
            Ok(ret as _)
        })
    }

    /// Return in which coordinate dimension the geometry is.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_coordinate_dimension() == Ok(CoordDimensions::ThreeD));
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_coordinate_dimension() == Ok(CoordDimensions::TwoD));
    /// ```
    fn get_coordinate_dimension(&self) -> GResult<CoordDimensions> {
        with_context(|ctx| unsafe {
            let ret = errcheck!(GEOSGeom_getCoordinateDimension_r(
                ctx.as_raw(),
                self.as_raw(),
            ))?;
            CoordDimensions::try_from(ret as u32)
        })
    }

    /// This function attempts to return a valid representation of `self`.
    ///
    /// Available using the `v3_8_0` feature.
    #[cfg(any(feature = "v3_8_0", feature = "dox"))]
    fn make_valid(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMakeValid_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Attempts to return a valid representation of `self` using the provided parameters.
    ///
    /// This allows control over the validation method and whether to preserve collapsed geometries.
    ///
    /// Available using the `v3_10_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, MakeValidMethod, MakeValidParams};
    ///
    /// // Bow-tie polygon (self-intersecting, invalid)
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 0 1, 1 0, 0 0))")
    ///     .expect("Invalid geometry");
    ///
    /// let params = MakeValidParams::builder()
    ///     .method(MakeValidMethod::Structure)
    ///     .keep_collapsed(false)
    ///     .build()
    ///     .expect("Failed to create params");
    ///
    /// let valid_geom = geom.make_valid_with_params(&params)
    ///     .expect("make_valid_with_params failed");
    ///
    /// assert!(valid_geom.is_valid().unwrap());
    /// ```
    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    fn make_valid_with_params(&self, params: &MakeValidParams) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMakeValidWithParams_r(
                ctx.as_raw(),
                self.as_raw(),
                params.as_raw(),
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the number of geometries.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(77.29 29.07,77.42 29.26,77.27 29.31,77.29 29.07)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_num_geometries(), Ok(1));
    ///
    /// let geom = Geometry::new_from_wkt("GEOMETRYCOLLECTION(MULTIPOINT(-2 3 , -2 2),\
    ///                                                       LINESTRING(5 5 ,10 10),\
    ///                                                       POLYGON((-7 4.2,-7.1 5,-7.1 4.3,-7 4.2)))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_num_geometries(), Ok(3));
    /// ```
    fn get_num_geometries(&self) -> GResult<usize> {
        with_context(|ctx| unsafe {
            let ret = errcheck!(-1, GEOSGetNumGeometries_r(ctx.as_raw(), self.as_raw()))?;
            Ok(ret as _)
        })
    }

    /// Get SRID of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let mut point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)")
    ///                               .expect("Invalid geometry");
    /// point_geom.set_srid(4326);
    /// assert_eq!(point_geom.get_srid(), Ok(4326));
    /// ```
    fn get_srid(&self) -> GResult<libc::c_int> {
        with_context(|ctx| unsafe {
            // No need to wrap this one, as 0 is a valid SRID
            let ret = GEOSGetSRID_r(ctx.as_raw(), self.as_raw());
            Ok(ret as _)
        })
    }

    /// Returns the precision of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert_eq!(point_geom.get_precision().map(|x| format!("{:.2}", x)).unwrap(), "0.00");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn get_precision(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            errcheck!(-1.0, GEOSGeom_getPrecision_r(ctx.as_raw(), self.as_raw()))
        })
    }

    /// Returns the precision of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, Precision};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    ///
    /// point_geom.set_precision(1., Precision::KeepCollapsed);
    /// assert_eq!(point_geom.get_precision().map(|x| format!("{:.2}", x)).unwrap(), "0.00");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn set_precision(&self, grid_size: f64, flags: Precision) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_setPrecision_r(
                ctx.as_raw(),
                self.as_raw(),
                grid_size,
                flags.into(),
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the biggest X of the geometry.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_max(), Ok(5.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn get_x_max(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut value = 0.0;
            errcheck!(GEOSGeom_getXMax_r(ctx.as_raw(), self.as_raw(), &mut value))?;
            Ok(value)
        })
    }

    /// Returns the smallest X of the geometry.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_min(), Ok(1.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn get_x_min(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut value = 0.0;
            errcheck!(GEOSGeom_getXMin_r(ctx.as_raw(), self.as_raw(), &mut value))?;
            Ok(value)
        })
    }

    /// Returns the biggest Y of the geometry.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_max(), Ok(6.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn get_y_max(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut value = 0.0;
            errcheck!(GEOSGeom_getYMax_r(ctx.as_raw(), self.as_raw(), &mut value))?;
            Ok(value)
        })
    }

    /// Returns the smallest Y of the geometry.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_min(), Ok(3.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    fn get_y_min(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut value = 0.0;
            errcheck!(GEOSGeom_getYMin_r(ctx.as_raw(), self.as_raw(), &mut value))?;
            Ok(value)
        })
    }

    /// Returns the smallest distance by which a vertex of `self` could be moved to produce an
    /// invalid geometry.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(geom.minimum_clearance().map(|x| format!("{:.8}", x)).unwrap(), "5.00000000");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn minimum_clearance(&self) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut value = 0.0;
            predicate!(GEOSMinimumClearance_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut value
            ))?;
            Ok(value)
        })
    }

    /// Returns the two-point `LineString` spanning of `self`'s minimum clearance.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON ((0 0, 1 0, 1 1, 0.5 3.2e-4, 0 0))")
    ///                     .expect("Invalid WKT");
    /// let line = geom.minimum_clearance_line().expect("minimum_clearance_line failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(line.to_wkt_precision(1).unwrap(), "LINESTRING (0.5 0.0, 0.5 0.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(line.to_wkt().unwrap(), "LINESTRING (0.5 0.00032, 0.5 0)");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn minimum_clearance_line(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMinimumClearanceLine_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the minimum rotated rectangle inside of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn minimum_rotated_rectangle(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMinimumRotatedRectangle_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the minimum width inside of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    fn minimum_width(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMinimumWidth_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a [delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation)
    /// around the vertices of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((175 150, 20 40, 50 60, 125 100, 175 150))")
    ///                      .expect("Invalid WKT");
    /// let geom2 = Geometry::new_from_wkt("POINT(110 170)").expect("Invalid WKT");
    /// let geom2 = geom2.buffer(20., 8).expect("buffer failed");
    ///
    /// let geom = geom1.union(&geom2).expect("union failed");
    ///
    /// let final_geom = geom.delaunay_triangulation(0.001, false).expect("delaunay_triangulation failed");
    /// ```
    fn delaunay_triangulation(&self, tolerance: f64, only_edges: bool) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSDelaunayTriangulation_r(
                ctx.as_raw(),
                self.as_raw(),
                tolerance,
                only_edges.into(),
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    fn interpolate(&self, d: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSInterpolate_r(ctx.as_raw(), self.as_raw(), d))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    fn interpolate_normalized(&self, d: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSInterpolateNormalized_r(ctx.as_raw(), self.as_raw(), d))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    fn project<G: Geom>(&self, p: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            errcheck!(-1.0, GEOSProject_r(ctx.as_raw(), self.as_raw(), p.as_raw()))
        })
    }

    fn project_normalized<G: Geom>(&self, p: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            errcheck!(
                -1.0,
                GEOSProjectNormalized_r(ctx.as_raw(), self.as_raw(), p.as_raw())
            )
        })
    }

    fn node(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSNode_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    ///  Return an offset line at a given distance and side from an input line. All points of the
    /// returned geometries are not further than the given distance from the input geometry.
    ///
    /// ### Parameters description:
    ///
    /// #### width
    ///
    /// * If `width` is positive, the offset will be at the left side of the input line and retain
    ///   the same direction.
    /// * If `width` is negative, it'll be at the right side and in the opposite direction.
    ///
    /// #### quadrant_segments
    ///
    /// * If `quadrant_segments` is >= 1, joins are round, and `quadrant_segments` indicates the
    ///   number of segments to use to approximate a quarter-circle.
    /// * If `quadrant_segments` == 0, joins are bevelled (flat).
    /// * If `quadrant_segments` < 0, joins are mitred, and the value of `quadrant_segments`
    ///   indicates the mitre ration limit as `mitre_limit = |quadrant_segments|`
    ///
    /// #### mitre_limit
    ///
    /// The mitre ratio is the ratio of the distance from the corner to the end of the mitred offset
    /// corner. When two line segments meet at a sharp angle, a miter join will extend far beyond
    /// the original geometry (and in the extreme case will be infinitely far). To prevent
    /// unreasonable geometry, the mitre limit allows controlling the maximum length of the join
    /// corner. Corners with a ratio which exceed the limit will be beveled.
    fn offset_curve(
        &self,
        width: f64,
        quadrant_segments: i32,
        join_style: JoinStyle,
        mitre_limit: f64,
    ) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSOffsetCurve_r(
                ctx.as_raw(),
                self.as_raw(),
                width,
                quadrant_segments,
                join_style.into(),
                mitre_limit,
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    fn point_on_surface(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSPointOnSurface_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns, in the tuple elements order:
    ///
    /// 1. The polygonized geometry.
    /// 2. The cuts geometries collection.
    /// 3. The dangles geometries collection.
    /// 4. The invalid geometries collection.
    #[allow(clippy::type_complexity)]
    fn polygonize_full(
        &self,
    ) -> GResult<(
        Geometry,
        Option<Geometry>,
        Option<Geometry>,
        Option<Geometry>,
    )> {
        let mut cuts: *mut GEOSGeometry = ::std::ptr::null_mut();
        let mut dangles: *mut GEOSGeometry = ::std::ptr::null_mut();
        let mut invalids: *mut GEOSGeometry = ::std::ptr::null_mut();

        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSPolygonize_full_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut cuts,
                &mut dangles,
                &mut invalids,
            ))?;
            let geom = Geometry::new_from_raw(ptr);

            let cuts = NonNull::new(cuts).map(Geometry::new_from_raw);
            let dangles = NonNull::new(dangles).map(Geometry::new_from_raw);
            let invalids = NonNull::new(invalids).map(Geometry::new_from_raw);
            Ok((geom, cuts, dangles, invalids))
        })
    }

    fn shared_paths<G: Geom>(&self, other: &G) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSSharedPaths_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Converts a [`Geometry`] to the HEX format. For more control over the generated output,
    /// use the [`WKBWriter`](crate::WKBWriter) type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to WKB failed");
    /// ```
    fn to_hex(&self) -> GResult<Vec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomToHEX_buf_r(ctx.as_raw(), self.as_raw(), &mut size))?;
            Ok(managed_vec(ptr, size, ctx))
        })
    }

    /// Converts a [`Geometry`] to the WKB format. For more control over the generated output,
    /// use the [`WKBWriter`](crate::WKBWriter) type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    /// ```
    fn to_wkb(&self) -> GResult<Vec<u8>> {
        let mut size = 0;
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomToWKB_buf_r(ctx.as_raw(), self.as_raw(), &mut size))?;
            Ok(managed_vec(ptr, size, ctx))
        })
    }

    /// Converts a [`Geometry`] to the GeoJSON format. For more control over the generated output,
    /// use the [`GeoJSONWriter`](crate::GeoJSONWriter) type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// assert_eq!(
    ///     point_geom.to_geojson().unwrap(),
    ///     r#"{"type":"Point","coordinates":[2.5,2.5]}"#,
    /// );
    /// ```
    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    fn to_geojson(&self) -> GResult<String> {
        GeoJSONWriter::new()?.write(self)
    }

    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    fn to_geojson_formatted(&self, indent: i32) -> GResult<String> {
        GeoJSONWriter::new()?.write_formatted(self, indent)
    }

    /// Creates a new [`PreparedGeometry`] from the current `Geometry`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                           .expect("Invalid geometry");
    /// let prepared_geom = point_geom.to_prepared_geom().expect("failed to create prepared geom");
    /// ```
    fn to_prepared_geom(&self) -> GResult<PreparedGeometry<'_>> {
        PreparedGeometry::new(self)
    }

    fn clone(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_clone_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns the 1-based nth geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("MULTIPOINT(1 1, 2 2, 3 3, 4 4)")
    ///                     .expect("Invalid geometry");
    /// let point_nb3 = geom
    ///     .get_geometry_n(2)
    ///     .expect("failed to get third point");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     point_nb3.to_wkt().unwrap(),
    ///     "POINT (3.0000000000000000 3.0000000000000000)",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(point_nb3.to_wkt().unwrap(), "POINT (3 3)");
    /// ```
    fn get_geometry_n(&self, n: usize) -> GResult<ConstGeometry<'_>> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGetGeometryN_r(ctx.as_raw(), self.as_raw(), n as _))?;
            Ok(ConstGeometry::new_from_raw(ptr))
        })
    }

    /// Returns the nth interior ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                            (1 1, 2 1, 2 5, 1 5, 1 1),\
    ///                                            (8 5, 8 4, 9 4, 9 5, 8 5))")
    ///                     .expect("Invalid geometry");
    /// let interior = geom
    ///     .get_interior_ring_n(0)
    ///     .expect("failed to get interior ring");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(interior.to_wkt().unwrap(),
    ///            "LINEARRING (1.0000000000000000 1.0000000000000000, \
    ///                         2.0000000000000000 1.0000000000000000, \
    ///                         2.0000000000000000 5.0000000000000000, \
    ///                         1.0000000000000000 5.0000000000000000, \
    ///                         1.0000000000000000 1.0000000000000000)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(interior.to_wkt().unwrap(), "LINEARRING (1 1, 2 1, 2 5, 1 5, 1 1)");
    /// ```
    fn get_interior_ring_n(&self, n: usize) -> GResult<ConstGeometry<'_>> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGetInteriorRingN_r(ctx.as_raw(), self.as_raw(), n as _))?;
            Ok(ConstGeometry::new_from_raw(ptr))
        })
    }

    /// Returns the exterior ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                               (1 1, 2 1, 2 5, 1 5, 1 1))")
    ///                           .expect("Invalid geometry");
    ///
    /// let exterior = point_geom
    ///     .get_exterior_ring()
    ///     .expect("failed to get exterior ring");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(exterior.to_wkt().unwrap(),
    ///            "LINEARRING (0.0000000000000000 0.0000000000000000, \
    ///                         10.0000000000000000 0.0000000000000000, \
    ///                         10.0000000000000000 6.0000000000000000, \
    ///                         0.0000000000000000 6.0000000000000000, \
    ///                         0.0000000000000000 0.0000000000000000)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(exterior.to_wkt().unwrap(), "LINEARRING (0 0, 10 0, 10 6, 0 6, 0 0)");
    /// ```
    fn get_exterior_ring(&self) -> GResult<ConstGeometry<'_>> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGetExteriorRing_r(ctx.as_raw(), self.as_raw()))?;
            Ok(ConstGeometry::new_from_raw(ptr))
        })
    }

    /// Apply XY coordinate transform callback to all coordinates in a copy of input geometry.
    /// If the callback returns an error, the function will return an Err.
    /// Z and M values, if present, are not modified by this function.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1.5 2.5)").expect("Invalid geometry");
    /// let transformed = geom.transform_xy(|x, y| {
    ///     Ok::<_, geos::Error>((x + 1.0, y + 2.0))
    /// }).expect("transform failed");
    /// assert_eq!(transformed.to_wkt_precision(1).unwrap(), "POINT (2.5 4.5)");
    /// ```
    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    fn transform_xy<F: Fn(f64, f64) -> Result<(f64, f64), E>, E: From<Error>>(
        &self,
        on_transform_point: F,
    ) -> Result<Geometry, E> {
        let mut trampoline = Trampoline::new(on_transform_point);

        with_context(|ctx| unsafe {
            let ptr = GEOSGeom_transformXY_r(
                ctx.as_raw(),
                self.as_raw(),
                trampoline.get_callback(),
                trampoline.as_mut_void(),
            );
            if let Some(ptr) = NonNull::new(ptr) {
                Ok(Geometry::new_from_raw(ptr))
            } else if let Some(err) = trampoline.err {
                Err(err)
            } else {
                Err(Error::GeosError(("GEOSGeom_transformXY_r", ctx.get_last_error())).into())
            }
        })
    }

    fn clip_by_rect(&self, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSClipByRect_r(
                ctx.as_raw(),
                self.as_raw(),
                xmin,
                ymin,
                xmax,
                ymax
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    fn densify(&self, tolerance: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSDensify_r(ctx.as_raw(), self.as_raw(), tolerance))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    fn remove_repeated_points(&self, tolerance: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSRemoveRepeatedPoints_r(
                ctx.as_raw(),
                self.as_raw(),
                tolerance
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    fn concave_hull(&self, ratio: f64, allow_holes: bool) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSConcaveHull_r(
                ctx.as_raw(),
                self.as_raw(),
                ratio,
                allow_holes.into()
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    fn get_extent(&self) -> GResult<Vec<f64>> {
        with_context(|ctx| unsafe {
            let mut xmin: f64 = 0.;
            let mut ymin: f64 = 0.;
            let mut xmax: f64 = 0.;
            let mut ymax: f64 = 0.;
            errcheck!(GEOSGeom_getExtent_r(
                ctx.as_raw(),
                self.as_raw(),
                &mut xmin,
                &mut ymin,
                &mut xmax,
                &mut ymax
            ))?;
            Ok(vec![xmin, ymin, xmax, ymax])
        })
    }

    #[cfg(any(feature = "v3_12_0", feature = "dox"))]
    fn line_substring(&self, start_fraction: f64, end_fraction: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSLineSubstring_r(
                ctx.as_raw(),
                self.as_raw(),
                start_fraction,
                end_fraction as _,
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }
}

/// Trampoline function helper function to get the trampoline function from the closure.
#[cfg(feature = "v3_11_0")]
struct Trampoline<F, E: From<Error>> {
    closure: F,
    err: Option<E>,
}

#[cfg(feature = "v3_11_0")]
impl<F, E: From<Error>> Trampoline<F, E> {
    fn new(closure: F) -> Self {
        Self { closure, err: None }
    }

    fn as_mut_void(&mut self) -> *mut libc::c_void {
        self as *mut _ as *mut _
    }
}

#[cfg(feature = "v3_11_0")]
impl<F: FnMut(f64, f64) -> Result<(f64, f64), E>, E: From<Error>> Trampoline<F, E> {
    fn get_callback(&self) -> GEOSTransformXYCallback {
        unsafe extern "C" fn transform_trampoline<F, E>(
            x: *mut libc::c_double,
            y: *mut libc::c_double,
            user_data: *mut libc::c_void,
        ) -> libc::c_int
        where
            F: FnMut(f64, f64) -> Result<(f64, f64), E>,
            E: From<Error>,
        {
            let trampoline = &mut *user_data.cast::<Trampoline<F, E>>();
            let closure = &mut trampoline.closure;
            match closure(*x, *y) {
                Ok((new_x, new_y)) => {
                    *x = new_x;
                    *y = new_y;
                    1
                }
                Err(error) => {
                    trampoline.err = Some(error);
                    0
                }
            }
        }

        Some(transform_trampoline::<F, E>)
    }
}

impl<'a> ConstGeometry<'a> {
    pub(crate) fn new_from_raw(ptr: NonNull<GEOSGeometry>) -> ConstGeometry<'a> {
        ConstGeometry {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl Geometry {
    pub(crate) fn new_from_raw(ptr: NonNull<GEOSGeometry>) -> Geometry {
        Geometry { ptr }
    }

    /// Creates a `Geometry` from the WKT format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// ```
    pub fn new_from_wkt(wkt: &str) -> GResult<Geometry> {
        with_context(|ctx| match CString::new(wkt) {
            Ok(c_str) => unsafe {
                let reader = nullcheck!(GEOSWKTReader_create_r(ctx.as_raw()))?.as_ptr();
                let ptr = nullcheck!(GEOSWKTReader_read_r(ctx.as_raw(), reader, c_str.as_ptr()))?;
                GEOSWKTReader_destroy_r(ctx.as_raw(), reader);
                Ok(Geometry::new_from_raw(ptr))
            },
            Err(e) => Err(Error::GenericError(format!(
                "Conversion to CString failed with {e}",
            ))),
        })
    }

    /// Create a new [`Geometry`] from the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to HEX failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = Geometry::new_from_hex(hex_buf.as_ref())
    ///                      .expect("conversion from HEX failed");
    /// assert_eq!(point_geom.equals(&new_geom), Ok(true));
    /// ```
    pub fn new_from_hex(hex: &[u8]) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomFromHEX_buf_r(ctx.as_raw(), hex.as_ptr(), hex.len()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Create a new [`Geometry`] from the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = Geometry::new_from_wkb(wkb_buf.as_ref())
    ///                      .expect("conversion from WKB failed");
    /// assert_eq!(point_geom.equals(&new_geom), Ok(true));
    /// ```
    pub fn new_from_wkb(wkb: &[u8]) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeomFromWKB_buf_r(ctx.as_raw(), wkb.as_ptr(), wkb.len()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates a `Geometry` from the `GeoJSON` format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_geojson(r#"{"type": "Point", "coordinates": [2.5, 2.5]}"#).expect("Invalid geometry");
    /// ```
    #[cfg(any(feature = "v3_10_0", feature = "dox"))]
    pub fn new_from_geojson(wkt: &str) -> GResult<Geometry> {
        with_context(|ctx| match CString::new(wkt) {
            Ok(c_str) => unsafe {
                let reader = nullcheck!(GEOSGeoJSONReader_create_r(ctx.as_raw()))?;
                let ptr = nullcheck!(GEOSGeoJSONReader_readGeometry_r(
                    ctx.as_raw(),
                    reader.as_ptr(),
                    c_str.as_ptr()
                ))?;
                GEOSGeoJSONReader_destroy_r(ctx.as_raw(), reader.as_ptr());
                Ok(Geometry::new_from_raw(ptr))
            },
            Err(e) => Err(Error::GenericError(format!(
                "Conversion to CString failed: {e}",
            ))),
        })
    }

    /// Creates an areal geometry formed by the constituent linework of given geometry.
    ///
    /// You can find new illustrations on [postgis](https://postgis.net/docs/ST_BuildArea.html)
    /// documentation.
    ///
    /// Available using the `v3_8_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("GEOMETRYCOLLECTION( \
    ///                                     POLYGON((0 0, 4 0, 4 4, 0 4, 0 0)), \
    ///                                     POLYGON((1 1, 1 3, 3 3, 3 1, 1 1)))")
    ///             .expect("Invalid geometry");
    ///
    /// let build_area_geom = geom.build_area().expect("build_area failed");
    /// // Square polygon with square hole.
    /// assert_eq!(build_area_geom.to_wkt_precision(0).unwrap(),
    ///            "POLYGON ((0 0, 0 4, 4 4, 4 0, 0 0), (1 1, 3 1, 3 3, 1 3, 1 1))");
    /// ```
    #[cfg(any(feature = "v3_8_0", feature = "dox"))]
    pub fn build_area(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBuildArea_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Description from [postgis](https://postgis.net/docs/ST_Polygonize.html):
    ///
    /// > Creates a GeometryCollection containing possible polygons formed from the constituent
    /// > linework of a set of geometries.
    ///
    /// # Example:
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((-71.040878 42.285678,\
    ///                                              -71.040943 42.2856,\
    ///                                              -71.04096 42.285752,\
    ///                                              -71.040878 42.285678))")
    ///                      .expect("Failed to create geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((-71.17166 42.353675,\
    ///                                              -71.172026 42.354044,\
    ///                                              -71.17239 42.354358,\
    ///                                              -71.171794 42.354971,\
    ///                                              -71.170511 42.354855,\
    ///                                              -71.17112 42.354238,\
    ///                                              -71.17166 42.353675))")
    ///                      .expect("Failed to create geometry");
    ///
    /// let polygonized = Geometry::polygonize(&[geom1, geom2]).expect("polygonize failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(polygonized.to_wkt().unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((-71.0408780000000064 42.2856779999999972, \
    ///                                           -71.0409429999999986 42.2856000000000023, \
    ///                                           -71.0409599999999983 42.2857520000000022, \
    ///                                           -71.0408780000000064 42.2856779999999972)), \
    ///                                 POLYGON ((-71.1716600000000028 42.3536750000000026, \
    ///                                           -71.1720260000000025 42.3540440000000018, \
    ///                                           -71.1723899999999929 42.3543579999999977, \
    ///                                           -71.1717940000000056 42.3549709999999990, \
    ///                                           -71.1705110000000047 42.3548550000000006, \
    ///                                           -71.1711200000000019 42.3542380000000023, \
    ///                                           -71.1716600000000028 42.3536750000000026)))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(polygonized.to_wkt().unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((-71.040878 42.285678, \
    ///                                           -71.040943 42.2856, \
    ///                                           -71.04096 42.285752, \
    ///                                           -71.040878 42.285678)), \
    ///                                 POLYGON ((-71.17166 42.353675, \
    ///                                           -71.172026 42.354044, \
    ///                                           -71.17239 42.354358, \
    ///                                           -71.171794 42.354971, \
    ///                                           -71.170511 42.354855, \
    ///                                           -71.17112 42.354238, \
    ///                                           -71.17166 42.353675)))");
    /// ```
    pub fn polygonize<T: Borrow<Geometry>>(geometries: &[T]) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let geoms = geometries
                .iter()
                .map(|g| g.borrow().as_raw().cast())
                .collect::<Vec<_>>();
            let ptr = nullcheck!(GEOSPolygonize_r(
                ctx.as_raw(),
                geoms.as_ptr(),
                geoms.len() as _
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    pub fn polygonizer_get_cut_edges<T: Borrow<Geometry>>(
        &self,
        geometries: &[T],
    ) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let geoms = geometries
                .iter()
                .map(|g| g.borrow().as_raw() as *const _)
                .collect::<Vec<_>>();
            let ptr = nullcheck!(GEOSPolygonizer_getCutEdges_r(
                ctx.as_raw(),
                geoms.as_ptr(),
                geoms.len() as _
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Merges `Multi Line String` geometry into a (set of) `Line String`.
    ///
    /// ### Warning
    ///
    /// If you use this function on something else than a `Multi Line String` or a
    /// `Line String`, it'll return an empty `Geometry collection`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let lines = Geometry::new_from_wkt("MULTILINESTRING((-29 -27,-30 -29.7,-36 -31,-45 -33),\
    ///                                                  (-45 -33,-46 -32))")
    ///                      .expect("Invalid geometry");
    /// let lines_merged = lines.line_merge().expect("line merge failed");
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     lines_merged.to_wkt_precision(1).unwrap(),
    ///     "LINESTRING (-29.0 -27.0, -30.0 -29.7, -36.0 -31.0, -45.0 -33.0, -46.0 -32.0)",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     lines_merged.to_wkt().unwrap(),
    ///     "LINESTRING (-29 -27, -30 -29.7, -36 -31, -45 -33, -46 -32)",
    /// );
    /// ```
    pub fn line_merge(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSLineMerge_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    pub fn line_merge_directed(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSLineMergeDirected_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Reverses the order of the vertexes.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 10,1 2)")
    ///                     .expect("invalid geometry");
    /// let reversed_line = line.reverse().expect("reverse failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(
    ///     reversed_line.to_wkt_precision(1).unwrap(),
    ///     "LINESTRING (1.0 2.0, 1.0 10.0)",
    /// );
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(
    ///     reversed_line.to_wkt().unwrap(),
    ///     "LINESTRING (1 2, 1 10)",
    /// );
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn reverse(&self) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSReverse_r(ctx.as_raw(), self.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a simplified version of the given geometry.
    pub fn simplify(&self, tolerance: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSSimplify_r(ctx.as_raw(), self.as_raw(), tolerance))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Returns a simplified version of the given geometry. It will avoid creating invalid derived
    /// geometries.
    pub fn topology_preserve_simplify(&self, tolerance: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSTopologyPreserveSimplify_r(
                ctx.as_raw(),
                self.as_raw(),
                tolerance
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Set SRID of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let mut point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)")
    ///                               .expect("Invalid geometry");
    /// point_geom.set_srid(4326);
    /// assert_eq!(point_geom.get_srid(), Ok(4326));
    /// ```
    pub fn set_srid(&mut self, srid: libc::c_int) {
        with_context(|ctx| unsafe { GEOSSetSRID_r(ctx.as_raw(), self.as_raw_mut(), srid) });
    }

    /// Normalizes `self` in its normalized/canonical form. May reorder vertices in polygon rings,
    /// rings in a polygon, elements in a multi-geometry complex.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let mut geom = Geometry::new_from_wkt(
    ///     "GEOMETRYCOLLECTION(POINT(2 3), MULTILINESTRING((0 0, 1 1),(2 2, 3 3)))",
    /// ).expect("Invalid geometry");
    ///
    /// geom.normalize().expect("normalize failed");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "GEOMETRYCOLLECTION (MULTILINESTRING ((2.0 2.0, 3.0 3.0), (0.0 0.0, 1.0 1.0)), \
    ///                                 POINT (2.0 3.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "GEOMETRYCOLLECTION (MULTILINESTRING ((2 2, 3 3), (0 0, 1 1)), POINT (2 3))");
    /// ```
    pub fn normalize(&mut self) -> GResult<()> {
        with_context(|ctx| unsafe {
            errcheck!(-1, GEOSNormalize_r(ctx.as_raw(), self.as_raw_mut()))?;
            Ok(())
        })
    }

    /// Creates an empty polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_polygon().expect("Failed to build empty polygon");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "POLYGON EMPTY");
    /// ```
    pub fn create_empty_polygon() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyPolygon_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_point().expect("Failed to build empty point");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "POINT EMPTY");
    /// ```
    pub fn create_empty_point() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyPoint_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty line string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_line_string().expect("Failed to build empty line string");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "LINESTRING EMPTY");
    /// ```
    pub fn create_empty_line_string() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyLineString_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty circular string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_circular_string().expect("Failed to build empty circular string");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "CIRCULARSTRING EMPTY");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_empty_circular_string() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyCircularString_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty compound curve geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_compound_curve().expect("Failed to build empty compound curve");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "COMPOUNDCURVE EMPTY");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_empty_compound_curve() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyCompoundCurve_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty curve polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_empty_curve_polygon().expect("Failed to build empty curve polygon");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "CURVEPOLYGON EMPTY");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_empty_curve_polygon() -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyCurvePolygon_r(ctx.as_raw()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates an empty collection.
    ///
    /// The `type_` must be one of:
    ///
    /// * [`GeometryTypes::GeometryCollection`]
    /// * [`GeometryTypes::MultiPoint`]
    /// * [`GeometryTypes::MultiLineString`]
    /// * [`GeometryTypes::MultiPolygon`]
    /// * [`GeometryTypes::MultiCurve`]
    /// * [`GeometryTypes::MultiSurface`]
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry, GeometryTypes};
    ///
    /// let geom = Geometry::create_empty_collection(GeometryTypes::MultiPolygon)
    ///                     .expect("Failed to build empty collection");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "MULTIPOLYGON EMPTY");
    /// ```
    pub fn create_empty_collection(type_: GeometryTypes) -> GResult<Geometry> {
        if !type_.is_collection() {
            return Err(Error::GenericError(format!(
                "invalid collection type: {type_:?}"
            )));
        }

        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createEmptyCollection_r(ctx.as_raw(), type_.into()))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates a polygon formed by the given shell and array of holes.
    ///
    /// ### Note
    ///
    /// `exterior` must be a `LinearRing`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("LINEARRING(75.15 29.53,77.1 29.2,77.6 29.5, 75.15 29.53)")
    ///                     .expect("Invalid geometry");
    /// let polygon_geom = Geometry::create_polygon(geom, vec![])
    ///                             .expect("create_polygon failed");
    ///
    /// assert_eq!(
    ///     polygon_geom.to_wkt_precision(1).unwrap(),
    ///     "POLYGON ((75.2 29.5, 77.1 29.2, 77.6 29.5, 75.2 29.5))",
    /// );
    /// ```
    pub fn create_polygon(
        mut exterior: Geometry,
        mut interiors: Vec<Geometry>,
    ) -> GResult<Geometry> {
        let nb_interiors = interiors.len();
        let polygon = with_context(|ctx| unsafe {
            let mut geoms: Vec<*mut GEOSGeometry> =
                interiors.iter_mut().map(AsRawMut::as_raw_mut).collect();
            let ptr = nullcheck!(GEOSGeom_createPolygon_r(
                ctx.as_raw(),
                exterior.as_raw_mut(),
                geoms.as_mut_ptr().cast(),
                nb_interiors as _,
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        });

        // We transfered the ownership of the ptr to the new Geometry,
        // so the old ones need to forget their c ptr to avoid double free.
        std::mem::forget(exterior);
        for interior in interiors {
            std::mem::forget(interior);
        }

        polygon
    }

    /// Create a geometry collection.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.0 4.0)").expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_geometry_collection(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((0.0 0.0, 10.0 0.0, 10.0 6.0, 0.0 6.0, 0.0 0.0)), \
    ///                                 POINT (3.0 4.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((0 0, 10 0, 10 6, 0 6, 0 0)), POINT (3 4))");
    /// ```
    pub fn create_geometry_collection(geoms: Vec<Geometry>) -> GResult<Geometry> {
        create_multi_geom(geoms, GeometryTypes::GeometryCollection)
    }

    /// Create a multi polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((3 3, 10 3, 10 6, 3 6, 3 3))")
    ///                      .expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipolygon(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "MULTIPOLYGON (((0.0 0.0, 10.0 0.0, 10.0 6.0, 0.0 6.0, 0.0 0.0)), \
    ///                           ((3.0 3.0, 10.0 3.0, 10.0 6.0, 3.0 6.0, 3.0 3.0)))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "MULTIPOLYGON (((0 0, 10 0, 10 6, 0 6, 0 0)), \
    ///                           ((3 3, 10 3, 10 6, 3 6, 3 3)))");
    /// ```
    pub fn create_multipolygon(polygons: Vec<Geometry>) -> GResult<Geometry> {
        if !check_same_geometry_type(&polygons, GeometryTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_owned(),
            ));
        }
        create_multi_geom(polygons, GeometryTypes::MultiPolygon)
    }

    /// Create a multiline string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (1.0 2.0, 3.0 4.0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (5.0 6.0, 7.0 8.0)").expect("invalid geometry");
    ///
    /// let geom = Geometry::create_multiline_string(vec![geom1, geom2])
    ///                     .expect("Failed to build multiline string");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "MULTILINESTRING ((1.0 2.0, 3.0 4.0), (5.0 6.0, 7.0 8.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "MULTILINESTRING ((1 2, 3 4), (5 6, 7 8))");
    /// ```
    pub fn create_multiline_string(linestrings: Vec<Geometry>) -> GResult<Geometry> {
        if !check_same_geometry_type(&linestrings, GeometryTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_owned(),
            ));
        }
        create_multi_geom(linestrings, GeometryTypes::MultiLineString)
    }

    /// Creates a multi point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1.0 2.0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.0 4.0)").expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipoint(vec![geom1, geom2])
    ///                     .expect("Failed to build multipoint");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 2.0, 3.0 4.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(), "MULTIPOINT ((1 2), (3 4))");
    /// ```
    pub fn create_multipoint(points: Vec<Geometry>) -> GResult<Geometry> {
        if !check_same_geometry_type(&points, GeometryTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_owned(),
            ));
        }
        create_multi_geom(points, GeometryTypes::MultiPoint)
    }

    /// Create a multicurve geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (1.0 2.0, 3.0 4.0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("CIRCULARSTRING (5.0 6.0, 7.0 8.0, 9.0 10.0)").expect("invalid geometry");
    ///
    /// let geom = Geometry::create_multicurve(vec![geom1, geom2])
    ///                     .expect("Failed to build multicurve");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "MULTICURVE ((1 2, 3 4), CIRCULARSTRING (5 6, 7 8, 9 10))");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_multicurve(curves: Vec<Geometry>) -> GResult<Geometry> {
        if !curves
            .iter()
            .all(|g| g.geometry_type().map(|t| t.is_curve()) == Ok(true))
        {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString or CircularString"
                    .to_owned(),
            ));
        }
        create_multi_geom(curves, GeometryTypes::MultiCurve)
    }

    /// Create a multisurface geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("CURVEPOLYGON(CIRCULARSTRING(1 3, 3 5, 4 7, 7 3, 1 3))").expect("invalid geometry");
    ///
    /// let geom = Geometry::create_multisurface(vec![geom1, geom2])
    ///                     .expect("Failed to build multisurface");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(),
    ///            "MULTISURFACE (((0 0, 1 1, 1 2, 1 1, 0 0)), CURVEPOLYGON (CIRCULARSTRING (1 3, 3 5, 4 7, 7 3, 1 3)))");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_multisurface(surfaces: Vec<Geometry>) -> GResult<Geometry> {
        if !surfaces
            .iter()
            .all(|g| g.geometry_type().map(|t| t.is_surface()) == Ok(true))
        {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon or CurvePolygon".to_owned(),
            ));
        }
        create_multi_geom(surfaces, GeometryTypes::MultiSurface)
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
    /// let geom = Geometry::create_point(coords).expect("Failed to create a point");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "POINT (1.0 2.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(), "POINT (1 2)");
    /// ```
    pub fn create_point(mut s: CoordSeq) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createPoint_r(ctx.as_raw(), s.as_raw_mut()));
            std::mem::forget(s);
            Ok(Geometry::new_from_raw(ptr?))
        })
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
    /// let geom = Geometry::create_line_string(coords).expect("Failed to create a line string");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "LINESTRING (1.0 2.0, 3.0 4.0)");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(), "LINESTRING (1 2, 3 4)");
    /// ```
    pub fn create_line_string(mut s: CoordSeq) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createLineString_r(ctx.as_raw(), s.as_raw_mut()));
            std::mem::forget(s);
            Ok(Geometry::new_from_raw(ptr?))
        })
    }

    /// Creates a linear ring geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[75.15, 29.53],
    ///                                       &[77.1, 29.2],
    ///                                       &[77.6, 29.5],
    ///                                       &[75.15, 29.53]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_linear_ring(coords)
    ///                     .expect("Failed to create a linear ring");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "LINEARRING (75.2 29.5, 77.1 29.2, 77.6 29.5, 75.2 29.5)");
    /// ```
    pub fn create_linear_ring(mut s: CoordSeq) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createLinearRing_r(ctx.as_raw(), s.as_raw_mut()));
            std::mem::forget(s);
            Ok(Geometry::new_from_raw(ptr?))
        })
    }

    /// Creates a rectangular polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::create_rectangle(0., 0., 1., 1.).expect("Failed to create a rectangle");
    ///
    /// #[cfg(not(feature = "v3_12_0"))]
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "POLYGON ((0.0 0.0, 1.0 0.0, 1.0 1.0, 0.0 1.0, 0.0 0.0))");
    /// #[cfg(feature = "v3_12_0")]
    /// assert_eq!(geom.to_wkt().unwrap(), "POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0))");
    /// ```
    #[cfg(any(feature = "v3_11_0", feature = "dox"))]
    pub fn create_rectangle(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createRectangle_r(
                ctx.as_raw(),
                xmin,
                ymin,
                xmax,
                ymax
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    }

    /// Creates a circular string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geom, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[75.15, 29.53],
    ///                                       &[77.2, 29.1],
    ///                                       &[77.6, 29.5],
    ///                                       &[75.15, 29.53]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_circular_string(coords)
    ///                     .expect("Failed to create a circular string");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "CIRCULARSTRING (75.2 29.5, 77.2 29.1, 77.6 29.5, 75.2 29.5)");
    /// ```
    #[cfg(any(feature = "v3_13_0", feature = "dox"))]
    pub fn create_circular_string(mut s: CoordSeq) -> GResult<Geometry> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createCircularString_r(
                ctx.as_raw(),
                s.as_raw_mut()
            ));
            std::mem::forget(s);
            Ok(Geometry::new_from_raw(ptr?))
        })
    }
}

impl<G: Geom> PartialEq<G> for ConstGeometry<'_> {
    fn eq(&self, other: &G) -> bool {
        self.equals(other).unwrap_or(false)
    }
}

impl<G: Geom> PartialEq<G> for Geometry {
    fn eq(&self, other: &G) -> bool {
        self.equals(other).unwrap_or(false)
    }
}

impl Clone for Geometry {
    fn clone(&self) -> Geometry {
        Geom::clone(self).expect("cloning failed")
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSGeom_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

as_raw_mut_impl!(Geometry, GEOSGeometry);
as_raw_impl!(ConstGeometry<'_>, GEOSGeometry);

#[cfg(test)]
mod test {
    #[cfg(feature = "v3_11_0")]
    use super::*;

    #[test]
    #[cfg(feature = "v3_11_0")]
    fn transform_point_geometry() {
        let geom = Geometry::new_from_wkt("POINT (1.5 2.5)").expect("Invalid geometry");
        let transformed = geom
            .transform_xy(|x, y| {
                assert_eq!(x, 1.5);
                assert_eq!(y, 2.5);

                Ok::<_, Error>((3.5, 4.5))
            })
            .expect("transform failed");

        let expected_geom = Geometry::new_from_wkt("POINT (3.5 4.5)").expect("Invalid geometry");
        assert_eq!(expected_geom.equals(&transformed), Ok(true));
    }

    #[test]
    #[cfg(feature = "v3_11_0")]
    fn transform_point_geometry_closure_error() {
        let geom = Geometry::new_from_wkt("POINT (1.5 2.5)").expect("Invalid geometry");
        match geom.transform_xy(|_x, _y| Err(Error::ImpossibleOperation("foo".into()))) {
            Ok(_) => panic!("transform_xy should have failed"),
            Err(e) => assert_eq!(e, Error::ImpossibleOperation("foo".into())),
        };
    }

    #[test]
    #[cfg(feature = "v3_11_0")]
    fn transform_polygon_geometry() {
        let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
            .expect("Invalid geometry");
        let transformed = geom
            .transform_xy(|x, y| Ok::<_, crate::Error>((x + 1.0, y + 2.0)))
            .expect("transform failed");

        let expected_geom = Geometry::new_from_wkt("POLYGON((1 2, 11 2, 11 8, 1 8, 1 2))")
            .expect("Invalid geometry");
        assert_eq!(expected_geom.equals(&transformed), Ok(true));
    }
}
