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
use std::sync::Arc;

pub struct GGeom<'a> {
    ptr: NonNull<GEOSGeometry>,
    context: Arc<GContextHandle<'a>>,
}

unsafe impl<'a> Send for GGeom<'a> {}
unsafe impl<'a> Sync for GGeom<'a> {}

impl<'a> Drop for GGeom<'a> {
    fn drop(&mut self) {
        unsafe { GEOSGeom_destroy(self.as_raw_mut()) }
    }
}

impl<'a> Clone for GGeom<'a> {
    /// Also pass the context to the newly created `GGeom`.
    fn clone(&self) -> GGeom<'a> {
        GGeom {
            ptr: NonNull::new(unsafe { GEOSGeom_clone(self.as_raw()) }).unwrap(),
            context: Arc::clone(&self.context),
        }
    }
}

impl<'a> GGeom<'a> {
    pub fn new(wkt: &str) -> GResult<GGeom<'a>> {
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
    pub fn set_context_handle(&mut self, context: GContextHandle<'a>) {
        self.context = Arc::new(content);
    }

    pub fn get_context_handle(&self) -> &GContextHandle<'a> {
        &self.context
    }

    pub(crate) unsafe fn new_from_raw(
        g: *mut GEOSGeometry,
        context: Arc<GContextHandle<'a>>,
    ) -> GResult<GGeom<'a>> {
        NonNull::new(g)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(|ptr| GGeom { ptr, context })
    }

    pub(crate) fn as_raw(&self) -> &GEOSGeometry {
        unsafe { self.ptr.as_ref() }
    }

    pub(crate) fn as_raw_mut(&mut self) -> &mut GEOSGeometry {
        unsafe { self.ptr.as_mut() }
    }

    pub(crate) fn clone_context(&self) -> Arc<GContextHandle<'a>> {
        Arc::clone(&self.context)
    }

    pub fn is_valid(&self) -> bool {
        unsafe { GEOSisValid_r(self.context.as_raw(), self.as_raw()) == 1 }
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
        let type_geom = unsafe { GEOSGeomTypeId_r(self.context.as_raw(), self.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        let mut n = 0.;

        if unsafe { GEOSArea_r(self.context.as_raw(), self.as_raw(), &mut n) } != 0 {
            Err(Error::GeosError("computing the area".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn to_wkt(&self) -> String {
        unsafe { managed_string(GEOSGeomToWKT_r(self.context.as_raw(), self.as_raw())) }
    }

    pub fn to_wkt_precision(&self, precision: Option<u32>) -> String {
        unsafe {
            let writer = GEOSWKTWriter_create_r(self.context.as_raw());
            if let Some(x) = precision {
                GEOSWKTWriter_setRoundingPrecision_r(self.context.as_raw(), writer, x as c_int)
            };
            let c_result = GEOSWKTWriter_write_r(self.context.as_raw(), writer, self.as_raw());
            GEOSWKTWriter_destroy_r(self.context.as_raw(), writer);
            managed_string(c_result)
        }
    }

    pub fn is_ring(&self) -> GResult<bool> {
        let rv = unsafe { GEOSisRing_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn intersects(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn crosses(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn disjoint(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn touches(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn overlaps(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn within(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    pub fn equals(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn equals_exact(&self, g2: &GGeom, precision: f64) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact_r(self.context.as_raw(), self.as_raw(), g2.as_raw(), precision)
        };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn covers(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn covered_by(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn contains(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains_r(self.context.as_raw(), self.as_raw(), g2.as_raw()) };
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
        let ret_val = unsafe { GEOSisEmpty_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.context.as_raw(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn difference(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSDifference_r(self.context.as_raw(), self.as_raw(), g2.as_raw()))
        }
    }

    pub fn envelope(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSEnvelope_r(self.context.as_raw(), self.as_raw())) }
    }

    pub fn sym_difference(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(
                GEOSSymDifference_r(self.context.as_raw(), self.as_raw(), g2.as_raw()))
        }
    }

    pub fn union(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSUnion_r(self.context.as_raw(), self.as_raw(), g2.as_raw()))
        }
    }

    pub fn get_centroid(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGetCentroid_r(self.context.as_raw(), self.as_raw())) }
    }

    pub fn create_polygon(mut exterior: GGeom, mut interiors: Vec<GGeom>) -> GResult<GGeom> {
        if let Ok(context_handle) = GContextHandle::init()
            let nb_interiors = interiors.len();
            let res = unsafe {
                let ptr = GEOSGeom_createPolygon_r(
                    context_handle.as_raw(),
                    exterior.ptr.as_mut(),
                    interiors.as_mut_ptr() as *mut *mut GEOSGeometry,
                    nb_interiors as c_uint,
                );
                GGeom::new_from_raw(ptr, Arc::new(context_handle))
            }?;

            // we'll transfert the ownership of the ptr to the new GGeom,
            // so the old one needs to forget their c ptr to avoid double cleanup
            mem::forget(exterior);
            for i in interiors {
                mem::forget(i);
            }

            Ok(res)
        } else {
            Err(Error::GenericError("GEOS_init_r failed".to_owned()))
        }
    }

    pub fn create_geometrycollection(geoms: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        create_multi_geom(geoms, GGeomTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&polygons, GGeomTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_string(),
            ));
        }
        create_multi_geom(polygons, GGeomTypes::MultiPolygon)
    }

    pub fn create_multilinestring(linestrings: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&linestrings, GGeomTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_string(),
            ));
        }
        create_multi_geom(linestrings, GGeomTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<GGeom<'a>>) -> GResult<GGeom<'a>> {
        if !check_same_geometry_type(&points, GGeomTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_string(),
            ));
        }
        create_multi_geom(points, GGeomTypes::MultiPoint)
    }

    pub fn create_point(s: CoordSeq) -> GResult<GGeom<'a>> {
        if let Ok(context_handle) = GContextHandle::init() {
            unsafe {
                // FIXME: is cloning really necessary?
                let coords = GEOSCoordSeq_clone_r(context_handle.as_raw(), s.as_raw());
                let ptr = GEOSGeom_createPoint_r(context_handle.as_raw(), coords);
                GGeom::new_from_raw(ptr, Arc::new(context_handle))
            }
        } else {
            Err(Error::GenericError("GEOS_init_r failed".to_owned()))
        }
    }

    pub fn create_line_string(s: CoordSeq) -> GResult<GGeom<'a>> {
        if let Ok(context_handle) = GContextHandle::init() {
            unsafe {
                let ptr = GEOSGeom_createLineString_r(context_handle.as_raw(), s.as_raw());
                mem::forget(s);
                GGeom::new_from_raw(ptr, Arc::new(context_handle))
            }
        } else {
            Err(Error::GenericError("GEOS_init_r failed".to_owned()))
        }
    }

    pub fn create_linear_ring(s: CoordSeq) -> GResult<GGeom<'a>> {
        let obj = unsafe { GGeom::new_from_raw(GEOSGeom_createLinearRing(s.as_raw())) }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn unary_union(&self) -> GResult<GGeom<'a>> {
        if let Some(ref context) = self.context {
            return context.geom_unary_union(self);
        }
        unsafe { GGeom::new_from_raw(GEOSUnaryUnion(self.as_raw())) }
    }

    pub fn voronoi(
        &self,
        envelope: Option<&GGeom<'a>>,
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

    pub fn intersection(&self, other: &GGeom<'a>) -> GResult<GGeom<'a>> {
        if let Some(ref context) = self.context {
            return context.geom_intersection(self, other);
        }
        unsafe { GGeom::new_from_raw(GEOSIntersection(self.as_raw(), other.as_raw())) }
    }

    pub fn convex_hull(&self) -> GResult<GGeom<'a>> {
        if let Some(ref context) = self.context {
            return context.geom_convex_hull(self);
        }
        unsafe { GGeom::new_from_raw(GEOSConvexHull(self.as_raw())) }
    }

    pub fn boundary(&self) -> GResult<GGeom<'a>> {
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

    pub fn from_wkb_buf(wkb: &[u8]) -> GResult<GGeom<'a>> {
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

    pub fn distance(&self, other: &GGeom<'a>) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn distance_indexed(&self, other: &GGeom<'a>) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_distance_indexed(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistanceIndexed(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance(&self, other: &GGeom<'a>) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_hausdorff_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance_densify(&self, other: &GGeom<'a>, distance_frac: f64) -> GResult<f64> {
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

    pub fn frechet_distance(&self, other: &GGeom<'a>) -> GResult<f64> {
        if let Some(ref context) = self.context {
            return context.geom_frechet_distance(self, other);
        }
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance(self.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn frechet_distance_densify(&self, other: &GGeom<'a>, distance_frac: f64) -> GResult<f64> {
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

    pub fn snap(&self, other: &GGeom<'a>, tolerance: f64) -> GResult<GGeom<'a>> {
        if let Some(ref context) = self.context {
            return context.geom_snap(self, other, tolerance);
        }
        unsafe { GGeom::new_from_raw(GEOSSnap(self.as_raw(), other.as_raw(), tolerance)) }
    }

    pub fn extract_unique_points(&self) -> GResult<GGeom<'a>> {
        if let Some(ref context) = self.context {
            return context.geom_extract_unique_points(self);
        }
        unsafe { GGeom::new_from_raw(GEOSGeom_extractUniquePoints(self.as_raw())) }
    }

    pub fn nearest_points(&self, other: &GGeom<'a>) -> GResult<CoordSeq> {
        if let Some(ref context) = self.context {
            return context.geom_nearest_points(self, other);
        }
        unsafe {
            CoordSeq::new_from_raw(GEOSNearestPoints(self.as_raw(), other.as_raw()))
        }
    }
}
