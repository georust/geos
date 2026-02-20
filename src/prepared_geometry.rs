use crate::context_handle::with_context;
use crate::functions::*;
use crate::traits::as_raw_impl;
use crate::{AsRaw, GResult, Geom};
use geos_sys::*;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// `PreparedGeometry` is an interface which prepares [`Geometry`](crate::Geometry) for greater performance
/// on repeated calls.
///
/// # Example
///
/// ```
/// use geos::{Geom, Geometry};
///
/// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
/// let mut prepared_geom = geom1.to_prepared_geom()?;
/// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
///
/// assert_eq!(prepared_geom.contains(&geom2)?, true);
/// # Ok::<(), geos::Error>(())
/// ```
pub struct PreparedGeometry<'a> {
    ptr: NonNull<GEOSPreparedGeometry>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> PreparedGeometry<'a> {
    /// Creates a new `PreparedGeometry` from a [`Geometry`](crate::Geometry).
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, PreparedGeometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
    /// let prepared_geom = PreparedGeometry::new(&geom1);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn new<G: Geom>(g: &'a G) -> GResult<Self> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSPrepare_r(ctx.as_raw(), g.as_raw()))?;
            Ok(PreparedGeometry {
                ptr,
                phantom: PhantomData,
            })
        })
    }

    /// Returns `true` if no points of the other geometry is outside the exterior of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
    /// let mut prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    ///
    /// assert_eq!(prepared_geom.contains(&geom2)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn contains<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedContains_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if every point of the `other` geometry is inside self's interior.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
    /// let mut prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
    ///
    /// assert_eq!(prepared_geom.contains_properly(&geom2)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn contains_properly<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedContainsProperly_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if no point of `self` is outside of `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)")?;
    /// let little_geom = geom.buffer(10., 8)?;
    /// let big_geom = geom.buffer(20., 8)?;
    ///
    /// let prepared_little_geom = little_geom.to_prepared_geom()?;
    /// let prepared_big_geom = big_geom.to_prepared_geom()?;
    ///
    /// assert_eq!(prepared_little_geom.covered_by(&big_geom)?, true);
    /// assert_eq!(prepared_big_geom.covered_by(&little_geom)?, false);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn covered_by<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedCoveredBy_r(
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
    /// let geom = Geometry::new_from_wkt("POINT (1 2)")?;
    /// let little_geom = geom.buffer(10., 8)?;
    /// let big_geom = geom.buffer(20., 8)?;
    ///
    /// let prepared_little_geom = little_geom.to_prepared_geom()?;
    /// let prepared_big_geom = big_geom.to_prepared_geom()?;
    ///
    /// assert_eq!(prepared_little_geom.covers(&big_geom)?, false);
    /// assert_eq!(prepared_big_geom.covers(&little_geom)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn covers<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedCovers_r(
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
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(1 1,2 2)")?;
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 1,1 2)")?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    ///
    /// assert_eq!(prepared_geom.crosses(&geom2)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn crosses<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedCrosses_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
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
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)")?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)")?;
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)")?;
    ///
    /// assert_eq!(prepared_geom.disjoint(&geom2)?, true);
    /// assert_eq!(prepared_geom.disjoint(&geom3)?, false);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn disjoint<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedDisjoint_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
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
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)")?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)")?;
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)")?;
    ///
    /// assert_eq!(prepared_geom.intersects(&geom2)?, false);
    /// assert_eq!(prepared_geom.intersects(&geom3)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn intersects<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedIntersects_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if `self` spatially overlaps `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 0.5)")?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(1 0, 1 1, 3 5)")?;
    ///
    /// assert_eq!(prepared_geom.overlaps(&geom2)?, false);
    ///
    /// let geom1 = geom1.buffer(3., 8)?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = geom2.buffer(0.5, 8)?;
    ///
    /// assert_eq!(prepared_geom.overlaps(&geom2)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn overlaps<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedOverlaps_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
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
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(0 0, 1 1, 0 2)")?;
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// let geom2 = Geometry::new_from_wkt("POINT(1 1)")?;
    ///
    /// assert_eq!(prepared_geom.touches(&geom2)?, false);
    ///
    /// let geom2 = Geometry::new_from_wkt("POINT(0 2)")?;
    ///
    /// assert_eq!(prepared_geom.touches(&geom2)?, true);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn touches<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedTouches_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if `self` is completely inside `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom = Geometry::new_from_wkt("POINT(50 50)")?;
    /// let small_geom = geom.buffer(20., 8)?;
    /// let big_geom = geom.buffer(40., 8)?;
    ///
    /// let small_prepared_geom = small_geom.to_prepared_geom()?;
    /// let big_prepared_geom = big_geom.to_prepared_geom()?;
    ///
    /// assert_eq!(small_prepared_geom.within(&small_geom)?, true);
    /// assert_eq!(small_prepared_geom.within(&big_geom)?, true);
    /// assert_eq!(big_prepared_geom.within(&small_geom)?, false);
    /// # Ok::<(), geos::Error>(())
    /// ```
    pub fn within<G: Geom>(&self, other: &G) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedWithin_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw()
            ))
        })
    }

    /// Returns `true` if the distance between `self` and `other` is shorter than `distance`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geom, Geometry};
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)")?;
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)")?;
    /// let geom3 = Geometry::new_from_wkt("POINT (3 2)")?;
    ///
    /// let prepared_geom = geom1.to_prepared_geom()?;
    /// assert_eq!(prepared_geom.dwithin(&geom2, 1.0)?, true);
    /// assert_eq!(prepared_geom.dwithin(&geom3, 1.0)?, false);
    /// # Ok::<(), geos::Error>(())
    /// ```
    #[cfg(feature = "v3_10_0")]
    pub fn dwithin<G: Geom>(&self, other: &G, distance: f64) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedDistanceWithin_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                distance
            ))
        })
    }

    #[cfg(feature = "v3_12_0")]
    pub fn contains_xy(&self, x: f64, y: f64) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedContainsXY_r(ctx.as_raw(), self.as_raw(), x, y))
        })
    }

    #[cfg(feature = "v3_12_0")]
    pub fn intersects_xy(&self, x: f64, y: f64) -> GResult<bool> {
        with_context(|ctx| unsafe {
            predicate!(GEOSPreparedIntersectsXY_r(
                ctx.as_raw(),
                self.as_raw(),
                x,
                y
            ))
        })
    }

    #[cfg(feature = "v3_9_0")]
    pub fn distance<G: Geom>(&self, other: &G) -> GResult<f64> {
        with_context(|ctx| unsafe {
            let mut distance = 0.0;
            errcheck!(GEOSPreparedDistance_r(
                ctx.as_raw(),
                self.as_raw(),
                other.as_raw(),
                &mut distance
            ))?;
            Ok(distance)
        })
    }
}

unsafe impl Send for PreparedGeometry<'_> {}
unsafe impl Sync for PreparedGeometry<'_> {}

impl Drop for PreparedGeometry<'_> {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSPreparedGeom_destroy_r(ctx.as_raw(), self.as_raw()) });
    }
}

as_raw_impl!(PreparedGeometry<'_>, GEOSPreparedGeometry);

/// Tests to ensure that the lifetime is correctly set.
///
/// ```compile_fail
/// use geos::{Geom, Geometry, PreparedGeometry};
///
/// pub struct Boo {
///     #[allow(dead_code)]
///     geom: Geometry<'static>,
///     pub prep: PreparedGeometry<'static>,
/// }
/// let boo = {
///     let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
///     let prep = geom.to_prepared_geom()?;
///      Boo { geom, prep }
/// };
/// let pt = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
/// assert!(boo.prep.contains(&pt)?);
/// ```
///
/// ```compile_fail
/// use geos::{Geom, Geometry, PreparedGeometry};
///
/// pub struct Boo {
///     pub prep: PreparedGeometry<'static>,
/// }
///
/// let boo = {
///     let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")?;
///     let prep = geom1.to_prepared_geom()?
///
///     Boo { prep }
/// };
///
/// let pt = Geometry::new_from_wkt("POINT (2.5 2.5)")?;
///
/// assert!(boo.prep.contains(&pt)?);
/// ```
#[cfg(doctest)]
pub mod lifetime_checks {}

/// This code is not supposed to compile since `PreparedGeometry` is not supposed to outlive
/// `Geometry`!
///
/// ```compile_fail
/// let prep_geom = {
///     let geom = crate::Geometry::new_from_wkt("POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))")?;
///     geom.to_prepared_geom()?
/// };
///
/// let pt = crate::Geometry::new_from_wkt("POINT(2 2)")?;
/// _ = prep_geom.contains(&pt);
/// ```
#[cfg(doctest)]
mod lifetime_prepared_geom_sigsegv {}
