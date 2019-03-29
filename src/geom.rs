use coord_seq::CoordSeq;
use enums::*;
use error::{Error, GResult, PredicateType};
use ffi::*;
use functions::*;
use libc::{c_double, c_int, c_uint};
use std::ffi::CString;
use std::ptr::NonNull;
use std::{self, mem, str};

#[repr(C)]
pub struct GGeom(NonNull<GEOSGeometry>);

impl Drop for GGeom {
    fn drop(&mut self) {
        unsafe { GEOSGeom_destroy(self.0.as_mut()) }
    }
}

impl Clone for GGeom {
    fn clone(&self) -> GGeom {
        GGeom(NonNull::new(unsafe { GEOSGeom_clone(self.0.as_ref()) }).unwrap())
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

    pub(crate) unsafe fn new_from_raw(g: *mut GEOSGeometry) -> GResult<GGeom> {
        NonNull::new(g)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(GGeom)
    }

    pub(crate) fn as_raw(&self) -> &GEOSGeometry {
        unsafe { self.0.as_ref() }
    }

    pub fn is_valid(&self) -> bool {
        let rv = unsafe { GEOSisValid(self.as_raw()) };
        return if rv == 1 { true } else { false };
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
        let type_geom = unsafe { GEOSGeomTypeId(self.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe { GEOSArea(self.as_raw(), &mut n) };

        if ret_val == 0 {
            Err(Error::GeosError("computing the area".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn to_wkt(&self) -> String {
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
        let rv = unsafe { GEOSisRing(self.as_raw()) };
        check_geos_predicate(rv, PredicateType::IsRing)
    }

    pub fn intersects(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Intersects)
    }

    pub fn crosses(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Crosses)
    }

    pub fn disjoint(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Disjoint)
    }

    pub fn touches(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Touches)
    }

    pub fn overlaps(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Overlaps)
    }

    pub fn within(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Within)
    }

    pub fn equals(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Equals)
    }

    pub fn equals_exact(&self, g2: &GGeom, precision: f64) -> GResult<bool> {
        let ret_val = unsafe { GEOSEqualsExact(self.as_raw(), g2.as_raw(), precision as c_double) };
        check_geos_predicate(ret_val, PredicateType::EqualsExact)
    }

    pub fn covers(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Covers)
    }

    pub fn covered_by(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::CoveredBy)
    }

    pub fn contains(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSBuffer(
                self.as_raw(),
                width as c_double,
                quadsegs as c_int,
            ))
        }
    }

    pub fn is_empty(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsSimple)
    }

    pub fn difference(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSDifference(self.as_raw(), g2.as_raw())) }
    }

    pub fn envelope(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSEnvelope(self.as_raw())) }
    }

    pub fn sym_difference(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSSymDifference(self.as_raw(), g2.as_raw())) }
    }

    pub fn union(&self, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSUnion(self.as_raw(), g2.as_raw())) }
    }

    pub fn get_centroid(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGetCentroid(self.as_raw())) }
    }

    pub fn create_polygon(mut exterior: GGeom, mut interiors: Vec<GGeom>) -> GResult<GGeom> {
        let nb_interiors = interiors.len();
        let res = unsafe {
            GGeom::new_from_raw(GEOSGeom_createPolygon(
                exterior.0.as_mut(),
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
        unsafe { GGeom::new_from_raw(GEOSUnaryUnion(self.as_raw())) }
    }

    pub fn voronoi(
        &self,
        envelope: Option<&GGeom>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<GGeom> {
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram(
                self.as_raw(),
                envelope
                    .map(|e| e.0.as_ptr() as *const GEOSGeometry)
                    .unwrap_or(std::ptr::null()),
                tolerance as c_double,
                only_edges as c_int,
            );
            Self::new_from_raw(raw_voronoi)
        }
    }

    pub fn normalize(&mut self) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize(self.0.as_ptr()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    pub fn intersection(&self, other: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSIntersection(self.as_raw(), other.as_raw())) }
    }

    pub fn convex_hull(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSConvexHull(self.as_raw())) }
    }

    pub fn boundary(&self) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSBoundary(self.as_raw())) }
    }

    pub fn has_z(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsSimple)
    }

    pub fn is_closed(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisClosed(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsSimple)
    }
}
