use crate::{CoordSeq, GContextHandle};
use enums::*;
use error::{Error, GResult, PredicateType};
use ffi::*;
use functions::*;
use libc::{c_double, c_int, c_uint};
use std::ffi::CString;
use std::ptr::NonNull;
use std::{self, mem, str};
use c_vec::CVec;

#[repr(C)]
pub struct GGeom {
    ptr: NonNull<GEOSGeometry>,
    context: Option<GContextHandle>,
}

impl Drop for GGeom {
    fn drop(&mut self) {
        unsafe { GEOSGeom_destroy(self.ptr.as_mut()) }
    }
}

impl Clone for GGeom {
    /// Doesn't clone the context handle contained in the original geometry.
    fn clone(&self) -> GGeom {
        GGeom {
            ptr: NonNull::new(unsafe { GEOSGeom_clone(self.ptr.as_ref()) }).unwrap(),
            context: None,
        }
    }
}

impl GGeom {
    pub fn new(wkt: &str) -> GResult<GGeom> {
        initialize();
        let c_str = CString::new(wkt).unwrap();
        let reader = unsafe { GEOSWKTReader_create() };
        let obj = unsafe { GEOSWKTReader_read(reader, c_str.as_ptr()) };
        if obj.is_null() {
            return Err(Error::NoConstructionFromNullPtr);
        }
        unsafe {
            GEOSWKTReader_destroy(reader);
            GGeom::new_from_raw(obj)
        }
    }

    /// Set the context handle to the geometry. It is best doing it this way.
    ///
    /// Therefore, instead of calling `handle.method(geom)`, you'll do:
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// point_geom.to_wkb_buf(); // we don't care about the result
    ///
    /// // Which is the same as doing (but better):
    /// if let Some(handle) = point_geom.set_handle(None) {
    ///     handle.geom_to_wkb_buf(&point_geom);
    /// }
    /// ```
    pub fn set_context_handle(
        &mut self,
        context: Option<GContextHandle>,
    ) -> Option<GContextHandle> {
        match context {
            Some(c) => self.context.replace(c),
            None => self.context.take(),
        }
    }

    pub fn get_context_handle(&self) -> &Option<GContextHandle> {
        &self.context
    }

    pub(crate) unsafe fn new_from_raw(g: *mut GEOSGeometry) -> GResult<GGeom> {
        NonNull::new(g)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(|p| GGeom { ptr: p, context: None })
    }

    pub(crate) fn as_raw(&self) -> &GEOSGeometry {
        unsafe { self.ptr.as_ref() }
    }

    pub(crate) fn as_raw_mut(&mut self) -> &mut GEOSGeometry {
        unsafe { self.ptr.as_mut() }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(ref context) = self.context {
            return context.geom_is_valid(self);
        }
        unsafe { GEOSisValid(self.as_raw()) == 1 }
    }

    /// get the underlying geos CoordSeq object from the geometry
    ///
    /// Note: this clones the underlying CoordSeq to avoid double free
    /// (because CoordSeq handles the object ptr and the CoordSeq is still owned by the geos geometry)
    /// if this method's performance becomes a bottleneck, feel free to open an issue, we could skip this clone with cleaner code
    pub fn get_coord_seq(&self) -> Result<CoordSeq, Error> {
        let type_geom = self.geometry_type();
        match type_geom {
            GGeomTypes::Point | GGeomTypes::LineString | GGeomTypes::LinearRing => unsafe {
                let t = GEOSCoordSeq_clone(GEOSGeom_getCoordSeq(self.as_raw()));
                CoordSeq::new_from_raw(t)
            },
            _ => Err(Error::ImpossibleOperation(
                "Geometry must be a Point, LineString or LinearRing to extract it's coordinates"
                    .into(),
            )),
        }
    }

    pub fn geometry_type(&self) -> GGeomTypes {
        if let Some(ref context) = self.context {
            return context.geometry_type(self);
        }
        let type_geom = unsafe { GEOSGeomTypeId(self.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_area(self);
        }
        let mut n = 0.;

        if unsafe { GEOSArea(self.as_raw(), &mut n) } != 0 {
            Err(Error::GeosError("computing the area".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn to_wkt(&self) -> String {
        if let Some(ref context) = self.context {
            return context.geom_to_wkt(self);
        }
        unsafe { managed_string(GEOSGeomToWKT(self.as_raw())) }
    }

    #[deprecated(note = "renamed to to_wkt_precision")]
    pub fn to_wkt_precison(&self, precision: Option<u32>) -> String {
        self.to_wkt_precision(precision)
    }

    pub fn to_wkt_precision(&self, precision: Option<u32>) -> String {
        unsafe {
            let writer = GEOSWKTWriter_create();
            if let Some(x) = precision {
                GEOSWKTWriter_setRoundingPrecision(writer, x as c_int)
            };
            let c_result = GEOSWKTWriter_write(writer, self.as_raw());
            GEOSWKTWriter_destroy(writer);
            managed_string(c_result)
        }
    }

    pub fn is_ring(&self) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_is_ring(self);
        }
        let rv = unsafe { GEOSisRing(self.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn intersects(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_intersects(self, g2);
        }
        let ret_val = unsafe { GEOSIntersects(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn crosses(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_crosses(self, g2);
        }
        let ret_val = unsafe { GEOSCrosses(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn disjoint(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_disjoint(self, g2);
        }
        let ret_val = unsafe { GEOSDisjoint(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn touches(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_touches(self, g2);
        }
        let ret_val = unsafe { GEOSTouches(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn overlaps(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_overlaps(self, g2);
        }
        let ret_val = unsafe { GEOSOverlaps(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn within(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_within(self, g2);
        }
        let ret_val = unsafe { GEOSWithin(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    pub fn equals(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_equals(self, g2);
        }
        let ret_val = unsafe { GEOSEquals(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn equals_exact(&self, g2: &GGeom, precision: f64) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_equals_exact(self, g2, precision);
        }
        let ret_val = unsafe { GEOSEqualsExact(self.as_raw(), g2.as_raw(), precision) };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn covers(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_covers(self, g2);
        }
        let ret_val = unsafe { GEOSCovers(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn covered_by(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_covered_by(self, g2);
        }
        let ret_val = unsafe { GEOSCoveredBy(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn contains(&self, g2: &GGeom) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_contains(self, g2);
        }
        let ret_val = unsafe { GEOSContains(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<GGeom> {
        assert!(quadsegs > 0);
        unsafe {
            GGeom::new_from_raw(GEOSBuffer(
                self.as_raw(),
                width as c_double,
                quadsegs as c_int,
            ))
        }
    }

    pub fn is_empty(&self) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_is_empty(self);
        }
        let ret_val = unsafe { GEOSisEmpty(self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_is_simple(self);
        }
        let ret_val = unsafe { GEOSisSimple(self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn difference(&self, g2: &GGeom) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_difference(self, g2);
        }
        unsafe { GGeom::new_from_raw(GEOSDifference(self.as_raw(), g2.as_raw())) }
    }

    pub fn envelope(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_envelope(self);
        }
        unsafe { GGeom::new_from_raw(GEOSEnvelope(self.as_raw())) }
    }

    pub fn sym_difference(&self, g2: &GGeom) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_sym_difference(self, g2);
        }
        unsafe { GGeom::new_from_raw(GEOSSymDifference(self.as_raw(), g2.as_raw())) }
    }

    pub fn union(&self, g2: &GGeom) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_union(self, g2);
        }
        unsafe { GGeom::new_from_raw(GEOSUnion(self.as_raw(), g2.as_raw())) }
    }

    pub fn get_centroid(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_get_centroid(self);
        }
        unsafe { GGeom::new_from_raw(GEOSGetCentroid(self.as_raw())) }
    }

    pub fn create_polygon(mut exterior: GGeom, mut interiors: Vec<GGeom>) -> GResult<GGeom> {
        let nb_interiors = interiors.len();
        let res = unsafe {
            GGeom::new_from_raw(GEOSGeom_createPolygon(
                exterior.ptr.as_mut(),
                interiors.as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_interiors as c_uint,
            ))
        }?;

        // we'll transfert the ownership of the ptr to the new GGeom,
        // so the old one needs to forget their c ptr to avoid double cleanup
        mem::forget(exterior);
        for i in interiors {
            mem::forget(i);
        }

        Ok(res)
    }

    pub fn create_geometrycollection(geoms: Vec<GGeom>) -> GResult<GGeom> {
        create_multi_geom(geoms, GGeomTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<GGeom>) -> GResult<GGeom> {
        if !check_same_geometry_type(&polygons, GGeomTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_string(),
            ));
        }
        create_multi_geom(polygons, GGeomTypes::MultiPolygon)
    }

    pub fn create_multilinestring(linestrings: Vec<GGeom>) -> GResult<GGeom> {
        if !check_same_geometry_type(&linestrings, GGeomTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_string(),
            ));
        }
        create_multi_geom(linestrings, GGeomTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<GGeom>) -> GResult<GGeom> {
        if !check_same_geometry_type(&points, GGeomTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_string(),
            ));
        }
        create_multi_geom(points, GGeomTypes::MultiPoint)
    }

    pub fn create_point(s: CoordSeq) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeom_createPoint(GEOSCoordSeq_clone(s.as_raw()))) }
    }

    pub fn create_line_string(s: CoordSeq) -> GResult<GGeom> {
        let obj = unsafe { GGeom::new_from_raw(GEOSGeom_createLineString(s.as_raw())) }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn create_linear_ring(s: CoordSeq) -> GResult<GGeom> {
        let obj = unsafe { GGeom::new_from_raw(GEOSGeom_createLinearRing(s.as_raw())) }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn unary_union(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_unary_union(self);
        }
        unsafe { GGeom::new_from_raw(GEOSUnaryUnion(self.as_raw())) }
    }

    pub fn voronoi(
        &self,
        envelope: Option<&GGeom>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_voronoi(self, envelope, tolerance, only_edges);
        }
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram(
                self.as_raw(),
                envelope
                    .map(|e| e.ptr.as_ptr() as *const GEOSGeometry)
                    .unwrap_or(std::ptr::null()),
                tolerance,
                only_edges as c_int,
            );
            Self::new_from_raw(raw_voronoi)
        }
    }

    pub fn normalize(&mut self) -> GResult<bool> {
        if self.context.is_some() {
            let context = self.context.take().unwrap();
            let ret = context.geom_normalize(self);
            self.context = Some(context);
            ret
        } else {
            let ret_val = unsafe { GEOSNormalize(self.as_raw_mut()) };
            check_geos_predicate(ret_val, PredicateType::Normalize)
        }
    }

    pub fn intersection(&self, other: &GGeom) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_intersection(self, other);
        }
        unsafe { GGeom::new_from_raw(GEOSIntersection(self.as_raw(), other.as_raw())) }
    }

    pub fn convex_hull(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_convex_hull(self);
        }
        unsafe { GGeom::new_from_raw(GEOSConvexHull(self.as_raw())) }
    }

    pub fn boundary(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_boundary(self);
        }
        unsafe { GGeom::new_from_raw(GEOSBoundary(self.as_raw())) }
    }

    pub fn has_z(&self) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_has_z(self);
        }
        let ret_val = unsafe { GEOSHasZ(self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn is_closed(&self) -> GResult<bool> {
        if let Some(ref context) = self.context {
            return context.geom_is_closed(self);
        }
        let ret_val = unsafe { GEOSisClosed(self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn from_wkb_buf(wkb: &[u8]) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeomFromWKB_buf(wkb.as_ptr(), wkb.len())) }
    }

    pub fn to_wkb_buf(&self) -> Option<CVec<u8>> {
        if let Some(ref context) = self.context {
            return context.geom_to_wkb_buf(self);
        }
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf(self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    pub fn length(&self) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_length(self);
        }
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength(self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn distance(&self, other: &GGeom) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn distance_indexed(&self, other: &GGeom) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_distance_indexed(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistanceIndexed(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance(&self, other: &GGeom) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_hausdorff_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance_densify(&self, other: &GGeom, distance_frac: f64) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_hausdorff_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify(self.as_raw(), other.as_raw(), distance_frac,
                                                   &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance(&self, other: &GGeom) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_frechet_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance_densify(&self, other: &GGeom, distance_frac: f64) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_frechet_distance_densify(self, other, distance_frac);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify(self.as_raw(), other.as_raw(), distance_frac,
                                                 &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn get_length(&self) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_get_length(self);
        }
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength(self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn snap(&self, other: &GGeom, tolerance: f64) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_snap(self, other, tolerance);
        }
        unsafe { GGeom::new_from_raw(GEOSSnap(self.as_raw(), other.as_raw(), tolerance)) }
    }

    pub fn extract_unique_points(&self) -> GResult<GGeom> {
        if let Some(ref context) = self.context {
            return context.geom_extract_unique_points(self);
        }
        unsafe { GGeom::new_from_raw(GEOSGeom_extractUniquePoints(self.as_raw())) }
    }

    pub fn nearest_points(&self, other: &GGeom) -> GResult<CoordSeq> {
        if let Some(ref context) = self.context {
            return context.geom_nearest_points(self, other);
        }
        unsafe {
            CoordSeq::new_from_raw(GEOSNearestPoints(self.as_raw(), other.as_raw()))
        }
    }
}
