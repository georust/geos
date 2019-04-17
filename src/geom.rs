use crate::{CoordSeq, GContextHandle, AsRaw, ContextHandling, ContextInteractions, PreparedGGeom};
use context_handle::PtrWrap;
use enums::*;
use error::{Error, GResult, PredicateType};
use geos_sys::*;
use functions::*;
use std::ffi::CString;
use std::{self, str};
use c_vec::CVec;
use std::sync::Arc;

pub struct GGeom<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSGeometry>,
    context: Arc<GContextHandle<'a>>,
}

impl<'a> GGeom<'a> {
    /// Create a new [`GGeom`] from the WKT format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt_s("POINT (2.5 2.5)".to_owned())
    ///                        .expect("Invalid geometry");
    /// ```
    pub fn new_from_wkt_s(wkt: String) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context) => {
                let wkt = match CString::new(wkt) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(Error::GenericError(format!("Conversion to CString failed: {}",
                                                               e)));
                    }
                };
                unsafe {
                    let ptr = GEOSGeomFromWKT_r(context.as_raw(), wkt.as_ptr());
                    GGeom::new_from_raw(ptr, Arc::new(context))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Same as [`new_from_wkt_s`] except it internally uses a reader instead of just using the
    /// given string.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// ```
    pub fn new_from_wkt(wkt: &str) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context_handle) => {
                match CString::new(wkt) {
                    Ok(c_str) => {
                        unsafe {
                            let reader = GEOSWKTReader_create_r(context_handle.as_raw());
                            let ptr = GEOSWKTReader_read_r(context_handle.as_raw(), reader,
                                                           c_str.as_ptr());
                            GEOSWKTReader_destroy_r(context_handle.as_raw(), reader);
                            GGeom::new_from_raw(ptr, Arc::new(context_handle))
                        }
                    }
                    Err(e) => {
                        Err(Error::GenericError(format!("Conversion to CString failed: {}", e)))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`GGeom`] from the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to HEX failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = GGeom::new_from_hex(hex_buf.as_ref())
    ///                      .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_hex(hex: &[u8]) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromHEX_buf_r(context.as_raw(), hex.as_ptr(), hex.len());
                    GGeom::new_from_raw(ptr, Arc::new(context))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`GGeom`] from the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = GGeom::new_from_wkb(wkb_buf.as_ref())
    ///                      .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_wkb(wkb: &[u8]) -> GResult<GGeom<'a>> {
        match GContextHandle::init() {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromWKB_buf_r(context.as_raw(), wkb.as_ptr(), wkb.len());
                    GGeom::new_from_raw(ptr, Arc::new(context))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Converts a [`GGeom`] to the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to WKB failed");
    /// ```
    pub fn to_hex(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Converts a [`GGeom`] to the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    /// ```
    pub fn to_wkb(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Creates a new [`PreparedGGeom`] from the current `GGeom`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let prepared_geom = point_geom.to_prepared_geom().expect("failed to create prepared geom");
    /// ```
    pub fn to_prepared_geom(&self) -> GResult<PreparedGGeom<'a>> {
        PreparedGGeom::new(self)
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSGeometry,
        context: Arc<GContextHandle<'a>>,
    ) -> GResult<GGeom<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr);
        }
        Ok(GGeom { ptr: PtrWrap(ptr), context })
    }

    pub fn is_valid(&self) -> bool {
        unsafe { GEOSisValid_r(self.get_raw_context(), self.as_raw()) == 1 }
    }

    /// Get the underlying geos CoordSeq object from the geometry
    ///
    /// Note: this clones the underlying CoordSeq to avoid double free
    /// (because CoordSeq handles the object ptr and the CoordSeq is still owned by the geos
    /// geometry) if this method's performance becomes a bottleneck, feel free to open an issue,
    /// we could skip this clone with cleaner code
    pub fn get_coord_seq(&self) -> GResult<CoordSeq<'a>> {
        let type_geom = self.geometry_type();
        match type_geom {
            GGeomTypes::Point | GGeomTypes::LineString | GGeomTypes::LinearRing => unsafe {
                let coord = GEOSGeom_getCoordSeq_r(self.get_raw_context(), self.as_raw());
                let t = GEOSCoordSeq_clone_r(self.get_raw_context(), coord);
                let mut size = 0;
                let mut dims = 0;

                if GEOSCoordSeq_getSize_r(self.get_raw_context(), coord, &mut size) == 0 {
                    return Err(Error::GenericError("GEOSCoordSeq_getSize_r failed".to_owned()));
                }
                if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), coord, &mut dims) == 0 {
                    return Err(Error::GenericError("GEOSCoordSeq_getDimensions_r failed".to_owned()));
                }
                CoordSeq::new_from_raw(t, self.clone_context(), size, dims)
            },
            _ => Err(Error::ImpossibleOperation(
                "Geometry must be a Point, LineString or LinearRing to extract it's coordinates"
                    .into(),
            )),
        }
    }

    pub fn geometry_type(&self) -> GGeomTypes {
        let type_geom = unsafe { GEOSGeomTypeId_r(self.get_raw_context(), self.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        let mut n = 0.;

        let res = unsafe { GEOSArea_r(self.get_raw_context(), self.as_raw(), &mut n) };
        if res != 1 {
            Err(Error::GeosError(format!("area failed with code {}", res)))
        } else {
            Ok(n as f64)
        }
    }

    pub fn to_wkt(&self) -> String {
        unsafe {
            let ptr = GEOSGeomToWKT_r(self.get_raw_context(), self.as_raw());
            managed_string(ptr, &self.context)
        }
    }

    pub fn to_wkt_precision(&self, precision: Option<u32>) -> String {
        unsafe {
            let writer = GEOSWKTWriter_create_r(self.get_raw_context());
            if let Some(x) = precision {
                GEOSWKTWriter_setRoundingPrecision_r(self.get_raw_context(), writer, x as _)
            };
            let c_result = GEOSWKTWriter_write_r(self.get_raw_context(), writer, self.as_raw());
            GEOSWKTWriter_destroy_r(self.get_raw_context(), writer);
            managed_string(c_result, &self.context)
        }
    }

    pub fn is_ring(&self) -> GResult<bool> {
        let rv = unsafe { GEOSisRing_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn intersects<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn crosses<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn disjoint<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn touches<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn overlaps<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn within<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    /// Checks if the two [`GGeom`] objects are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let geom1 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = GGeom::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1.equals(&geom2) == Ok(false));
    /// assert!(geom1.equals(&geom3) == Ok(true));
    /// ```
    ///
    /// Note that you can also use method through the `PartialEq` trait:
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let geom1 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = GGeom::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1 != geom2);
    /// assert!(geom1 == geom3);
    /// ```
    pub fn equals<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn equals_exact<'b>(&self, g2: &GGeom<'b>, precision: f64) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact_r(self.get_raw_context(), self.as_raw(), g2.as_raw(), precision)
        };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn covers<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn covered_by<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn contains<'b>(&self, g2: &GGeom<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<GGeom<'a>> {
        assert!(quadsegs > 0);
        unsafe {
            let ptr = GEOSBuffer_r(
                self.get_raw_context(),
                self.as_raw(),
                width,
                quadsegs as _,
            );
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn is_empty(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn difference<'b>(&self, g2: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSDifference_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn envelope(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSEnvelope_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn sym_difference<'b>(&self, g2: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSSymDifference_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn union(&self, g2: &GGeom<'a>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSUnion_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn get_centroid(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGetCentroid_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn create_polygon<'b>(mut exterior: GGeom<'a>, mut interiors: Vec<GGeom<'b>>) -> GResult<GGeom<'a>> {
        let context_handle = exterior.clone_context();
        let nb_interiors = interiors.len();
        let res = unsafe {
            let mut geoms: Vec<*mut GEOSGeometry> = interiors.iter_mut().map(|g| g.as_raw()).collect();
            let ptr = GEOSGeom_createPolygon_r(
                context_handle.as_raw(),
                exterior.as_raw(),
                geoms.as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_interiors as _,
            );
            GGeom::new_from_raw(ptr, context_handle)
        };

        // We transfered the ownership of the ptr to the new GGeom,
        // so the old ones need to forget their c ptr to avoid double free.
        exterior.ptr = PtrWrap(::std::ptr::null_mut());
        for i in interiors.iter_mut() {
            i.ptr = PtrWrap(::std::ptr::null_mut());
        }

        res
    }

    pub fn create_geometrycollection(geoms: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        create_multi_geom(geoms, GGeomTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&polygons, GGeomTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_owned(),
            ));
        }
        create_multi_geom(polygons, GGeomTypes::MultiPolygon)
    }

    pub fn create_multilinestring(linestrings: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&linestrings, GGeomTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_owned(),
            ));
        }
        create_multi_geom(linestrings, GGeomTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&points, GGeomTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_owned(),
            ));
        }
        create_multi_geom(points, GGeomTypes::MultiPoint)
    }

    pub fn create_point(mut s: CoordSeq<'a>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGeom_createPoint_r(s.get_raw_context(), s.as_raw());
            let res = GGeom::new_from_raw(ptr, s.clone_context());
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn create_line_string(mut s: CoordSeq<'a>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLineString_r(s.get_raw_context(), s.as_raw());
            let res = GGeom::new_from_raw(ptr, s.clone_context());
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn create_linear_ring(mut s: CoordSeq<'a>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLinearRing_r(s.get_raw_context(), s.as_raw());
            let res = GGeom::new_from_raw(ptr, s.clone_context());
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn unary_union(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSUnaryUnion_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn voronoi<'b>(
        &self,
        envelope: Option<&GGeom<'b>>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<GGeom<'a>> {
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram_r(
                self.get_raw_context(),
                self.as_raw(),
                envelope
                    .map(|e| e.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                tolerance,
                only_edges as _,
            );
            Self::new_from_raw(raw_voronoi, self.clone_context())
        }
    }

    pub fn normalize(&mut self) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    pub fn intersection<'b>(&self, other: &GGeom<'b>) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSIntersection_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn convex_hull(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSConvexHull_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn boundary(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSBoundary_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn has_z(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn is_closed(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisClosed_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn distance_indexed<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistanceIndexed_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance_densify<'b>(&self, other: &GGeom<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance<'b>(&self, other: &GGeom<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance_densify<'b>(&self, other: &GGeom<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn get_length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn snap<'b>(&self, other: &GGeom<'b>, tolerance: f64) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSSnap_r(self.get_raw_context(), self.as_raw(), other.as_raw(), tolerance);
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn extract_unique_points(&self) -> GResult<GGeom<'a>> {
        unsafe {
            let ptr = GEOSGeom_extractUniquePoints_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    pub fn nearest_points<'b>(&self, other: &GGeom<'b>) -> GResult<CoordSeq<'a>> {
        unsafe {
            let ptr = GEOSNearestPoints_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
            );
            let mut size = 0;
            let mut dims = 0;

            if GEOSCoordSeq_getSize_r(self.get_raw_context(), ptr, &mut size) == 0 {
                return Err(Error::GenericError("GEOSCoordSeq_getSize_r failed".to_owned()));
            }
            if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), ptr, &mut dims) == 0 {
                return Err(Error::GenericError("GEOSCoordSeq_getDimensions_r failed".to_owned()));
            }
            CoordSeq::new_from_raw(ptr, self.clone_context(), size, dims)
        }
    }

    /// Returns the X position. The given `GGeom` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_x() == Ok(1.5));
    /// ```
    pub fn get_x(&self) -> GResult<f64> {
        if self.geometry_type() != GGeomTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut x = 0.;
        unsafe {
            if GEOSGeomGetX_r(self.get_raw_context(), self.as_raw(), &mut x) != 0 {
                Ok(x)
            } else {
                Err(Error::GenericError("GEOSGeomGetX_r failed".to_owned()))
            }
        }
    }

    /// Returns the Y position. The given `GGeom` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_y() == Ok(2.5));
    /// ```
    pub fn get_y(&self) -> GResult<f64> {
        if self.geometry_type() != GGeomTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut y = 0.;
        unsafe {
            if GEOSGeomGetY_r(self.get_raw_context(), self.as_raw(), &mut y) != 0 {
                Ok(y)
            } else {
                Err(Error::GenericError("GEOSGeomGetY_r failed".to_owned()))
            }
        }
    }

    /// Returns the Z position. The given `GGeom` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GGeom;
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    pub fn get_z(&self) -> GResult<f64> {
        if self.geometry_type() != GGeomTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut z = 0.;
        unsafe {
            if GEOSGeomGetZ_r(self.get_raw_context(), self.as_raw(), &mut z) != 0 {
                Ok(z)
            } else {
                Err(Error::GenericError("GEOSGeomGetZ_r failed".to_owned()))
            }
        }
    }

    /// The given `GGeom` must be a `LineString`, otherwise it'll fail.
    pub fn get_point_n(&self, n: usize) -> GResult<GGeom<'a>> {
        if self.geometry_type() != GGeomTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetPointN_r(self.get_raw_context(), self.as_raw(), n as _);
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    /// The given `GGeom` must be a `LineString`, otherwise it'll fail.
    pub fn get_start_point(&self) -> GResult<GGeom<'a>> {
        if self.geometry_type() != GGeomTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetStartPoint_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }

    /// The given `GGeom` must be a `LineString`, otherwise it'll fail.
    pub fn get_end_point(&self) -> GResult<GGeom<'a>> {
        if self.geometry_type() != GGeomTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetEndPoint_r(self.get_raw_context(), self.as_raw());
            GGeom::new_from_raw(ptr, self.clone_context())
        }
    }
}

unsafe impl<'a> Send for GGeom<'a> {}
unsafe impl<'a> Sync for GGeom<'a> {}

impl<'a> Drop for GGeom<'a> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { GEOSGeom_destroy_r(self.get_raw_context(), self.as_raw()) }
        }
    }
}

impl<'a> Clone for GGeom<'a> {
    /// Also passes the context to the newly created `GGeom`.
    fn clone(&self) -> GGeom<'a> {
        let context = self.clone_context();
        let ptr = unsafe { GEOSGeom_clone_r(context.as_raw(), self.as_raw()) };
        if ptr.is_null() {
            panic!("Couldn't clone geometry...");
        }
        GGeom {
            ptr: PtrWrap(ptr),
            context,
        }
    }
}

impl<'a> PartialEq for GGeom<'a> {
    fn eq<'b>(&self, other: &GGeom<'b>) -> bool {
        self.equals(other).unwrap_or_else(|_| false)
    }
}

impl<'a> ContextInteractions for GGeom<'a> {
    type Context = GContextHandle<'a>;

    /// Set the context handle to the geometry.
    ///
    /// ```
    /// use geos::{ContextInteractions, GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let mut point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// point_geom.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: Self::Context) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the geometry.
    ///
    /// ```
    /// use geos::{ContextInteractions, GGeom};
    ///
    /// let point_geom = GGeom::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let context = point_geom.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &Self::Context {
        &self.context
    }
}

impl<'a> AsRaw for GGeom<'a> {
    type RawType = *mut GEOSGeometry;

    fn as_raw(&self) -> Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for GGeom<'a> {
    type Context = Arc<GContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<GContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
