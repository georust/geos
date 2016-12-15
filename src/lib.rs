#![crate_name="geos"]
#![crate_type="lib"]

extern crate libc;

use libc::{c_char, c_void, c_int, c_uint, c_double};
use std::ffi::{CString, CStr};
use std::{str, ptr};

#[link(name = "geos_c")]
extern {
    fn initGEOS() -> *mut c_void;
    fn GEOSversion() -> *const c_char;
    fn finishGEOS() -> *mut c_void;
    fn GEOSPrepare(g: *const c_void) -> *mut c_void;

    fn GEOSCoordSeq_create(size: c_uint, dims: c_uint) -> *mut c_void;
    fn GEOSCoordSeq_destroy(s: *mut c_void);
    fn GEOSCoordSeq_clone(s: *const c_void) -> *mut c_void;
    fn GEOSCoordSeq_setX(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setY(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setZ(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;

    fn GEOSGeom_getCoordSeq(g: *const c_void) -> *const c_void;
    // Geometry constructor :
    fn GEOSGeom_createPoint(s: *const c_void) -> *const c_void;
    fn GEOSGeom_createLineString(s: *const c_void) -> *const c_void;

    // Functions acting on GEOSGeometry :
    fn GEOSisEmpty(g: *const c_void) -> c_int;
    fn GEOSisSimple(g: *const c_void) -> c_int;
    fn GEOSisRing(g: *const c_void) -> c_int;
    fn GEOSHasZ(g: *const c_void) -> c_int;
    fn GEOSisClosed(g: *const c_void) -> c_int;

    fn GEOSGeomFromWKT(wkt: *const c_char) -> *const c_void;
    fn GEOSGeomToWKT(g: *const c_void) -> *const c_char;

    fn GEOSArea(g: *const c_void, area: *mut c_double) -> c_int;
    fn GEOSLength(g: *const c_void, distance: *mut c_double) -> c_int;
    fn GEOSDisjoint(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSTouches(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSIntersects(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSCrosses(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSWithin(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSContains(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSOverlaps(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSEquals(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSEqualsExact(g1: *const c_void, g2: *const c_void, tolerance: c_double) -> c_int;
    fn GEOSCovers(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSCoveredBy(g1: *const c_void, g2: *const c_void) -> c_int;

    fn GEOSBuffer(g: *const c_void, width: c_double, quadsegs: c_int) -> *const c_void;
    fn GEOSEnvelope(g: *const c_void) -> *const c_void;
    fn GEOSConvexHull(g: *const c_void) -> *const c_void;
    fn GEOSBoundary(g: *const c_void) -> *const c_void;
    fn GEOSGetCentroid(g: *const c_void) -> *const c_void;
    fn GEOSSymDifference(g1: *const c_void, g2: *const c_void) -> *const c_void;
    fn GEOSDifference(g1: *const c_void, g2: *const c_void) -> *const c_void;
    fn GEOSClipByRect(g: *const c_void, xmin: c_double, ymin: c_double, xmax: c_double, ymax: c_double) -> *const c_void;
    fn GEOSSnap(g1: *const c_void, g2: *const c_void, tolerance: c_double) -> *const c_void;
    fn GEOSGeom_extractUniquePoints(g: *const c_void) -> *const c_void;
    // Functions acting on GEOSPreparedGeometry :
    fn GEOSPreparedContains(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedContainsProperly(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedCoveredBy(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedCovers(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedCrosses(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedDisjoint(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedIntersects(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedOverlaps(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedTouches(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedWithin(pg1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSPreparedGeom_destroy(g: *mut c_void);
}

#[derive(Debug)]
pub enum GEOSGeomTypes {
    GEOS_POINT,
    GEOS_LINESTRING,
    GEOS_LINEARRING,
    GEOS_POLYGON,
    GEOS_MULTIPOINT,
    GEOS_MULTILINESTRING,
    GEOS_MULTIPOLYGON,
    GEOS_GEOMETRYCOLLECTION
}

pub fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    return str::from_utf8(c_str.to_bytes()).unwrap().to_string();
}

pub fn _Point(s: &CoordSeq) -> GeosGeom {
    GeosGeom(unsafe { GEOSGeom_createPoint(s.0 as *const c_void) })
}

pub fn LineString(s: &CoordSeq) -> GeosGeom {
    GeosGeom(unsafe { GEOSGeom_createLineString(s.0 as *const c_void)})
}

pub fn Snap(g1: &GeosGeom, g2: &GeosGeom, tolerance: f64) -> GeosGeom {
    GeosGeom(unsafe { GEOSSnap(g1.0, g2.0, tolerance as c_double) })
}

pub fn ClipByRect(g: &GeosGeom, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GeosGeom {
    GeosGeom(unsafe { GEOSClipByRect(g.0, xmin as c_double, ymin as c_double, xmax as c_double, ymax as c_double)})
}

pub fn version() -> String {
    unsafe {
        _string(GEOSversion())
    }
}

pub fn init() -> *mut c_void {
    unsafe {
        initGEOS()
    }
}

pub fn finish() -> *mut c_void {
    unsafe {
        finishGEOS()
    }
}

pub struct CoordSeq(*mut c_void);

impl Drop for CoordSeq {
    fn drop(&mut self) {
        if self.0.is_null() { return; }
        unsafe { GEOSCoordSeq_destroy(self.0)};
        self.0 = ptr::null_mut();
    }
}

impl Clone for CoordSeq {
    fn clone(&self) -> CoordSeq {
        CoordSeq(unsafe { GEOSCoordSeq_clone(self.0 as *const c_void) })
    }
}

impl CoordSeq {
    pub fn new(size: u32, dims: u32) -> CoordSeq {
        CoordSeq(unsafe { GEOSCoordSeq_create(size as c_uint, dims as c_uint) })
    }
    pub fn set_x(&mut self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setX(self.0, idx as c_uint, val as c_double) };
        return ret_val;
    }
    pub fn set_y(&mut self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setY(self.0, idx as c_uint, val as c_double) };
        return ret_val;
    }
    pub fn set_z(&mut self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setZ(self.0, idx as c_uint, val as c_double) };
        return ret_val;
    }
}

pub struct GeosGeom(*const c_void);

impl Drop for GeosGeom {
    fn drop(&mut self) {
        if self.0.is_null() { return; }
        self.0 = ptr::null();
    }
}

impl Clone for GeosGeom {
    fn clone(&self) -> GeosGeom {
        GeosGeom(self.0)
    }
}

impl GeosGeom {
    pub fn new(wkt: &str) -> GeosGeom {
        let c_str = CString::new(wkt).unwrap();
        unsafe { GeosGeom(GEOSGeomFromWKT(c_str.as_ptr())) }
    }

    pub fn to_wkt(&self) -> CString {
        unsafe { CStr::from_ptr(GEOSGeomToWKT(self.0 as *const c_void)).to_owned() }
    }

    pub fn area(&self) -> f64 {
        let n_mut_ref = &mut 0.0;
        let n_mut_ptr: *mut c_double = n_mut_ref;
        let ret_val = unsafe { GEOSArea(self.0, n_mut_ptr) };
        return *n_mut_ref;
    }

    pub fn length(&self) -> f64 {
        let n_mut_ref = &mut 0.0;
        let n_mut_ptr: *mut c_double = n_mut_ref;
        let ret_val = unsafe { GEOSLength(self.0, n_mut_ptr) };
        return *n_mut_ref;
    }

    pub fn intersects(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSIntersects(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn disjoint(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSDisjoint(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn touches(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSTouches(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn crosses(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSCrosses(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn within(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSWithin(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn covers(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSCovers(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn covered_by(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSCoveredBy(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn contains(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSContains(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn overlaps(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSOverlaps(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn equals(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe {GEOSEquals(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn equals_exact(&self, g2: &GeosGeom, tolerance: f64) -> bool {
        let ret_val = unsafe { GEOSEqualsExact(self.0, g2.0 as *const c_void, tolerance as c_double) };
        return if ret_val == 1 { true } else { false };
    }

    pub fn is_empty(&self) -> bool {
        let ret_val = unsafe { GEOSisEmpty(self.0) };
        println!("{}", ret_val);
        return if ret_val == 1 { true } else { false };
    }

    pub fn is_simple(&self) -> bool {
        let ret_val = unsafe { GEOSisSimple(self.0) };
        println!("{}", ret_val);
        return if ret_val == 1 { true } else { false };
    }

    pub fn convex_hull(&self) -> *const c_void {
        unsafe { GEOSConvexHull(self.0) }
    }

    pub fn envelope(&self) -> *const c_void {
        unsafe { GEOSEnvelope(self.0) }
    }

    pub fn boundary(&self) -> GeosGeom {
        let ret_val = unsafe { GEOSBoundary(self.0) };
        if ret_val.is_null(){ panic!("Invalid geometry"); }
        GeosGeom(ret_val)
    }

    pub fn get_centroid(&self) -> GeosGeom {
        let ret_val = unsafe { GEOSGetCentroid(self.0) };
        if ret_val.is_null(){ panic!("Invalid geometry"); }
        GeosGeom(ret_val)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GeosGeom {
        let ret_val = unsafe {GEOSBuffer(self.0, width as c_double, quadsegs as c_int)};
        if ret_val.is_null(){ panic!("Invalid geometry"); }
        GeosGeom(ret_val)
    }

    pub fn difference(&self, g2: &GeosGeom) -> GeosGeom {
        GeosGeom(unsafe { GEOSDifference(self.0, g2.0) })
    }

    pub fn sym_difference(&self, g2: &GeosGeom) -> GeosGeom {
        GeosGeom(unsafe { GEOSSymDifference(self.0, g2.0) })
    }
}

pub struct GeosPrepGeom(*mut c_void);

impl Clone for GeosPrepGeom {
    fn clone(&self) -> GeosPrepGeom {
        GeosPrepGeom(self.0)
    }
}

impl Drop for GeosPrepGeom {
    fn drop(&mut self) {
        if self.0.is_null() { return; }
        unsafe { GEOSPreparedGeom_destroy(self.0) };
        self.0 = ptr::null_mut();
    }
}

impl GeosPrepGeom {
    pub fn new(g: &GeosGeom) -> GeosPrepGeom {
        GeosPrepGeom(unsafe { GEOSPrepare(g.0)})
    }
    pub fn contains(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedContains(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn contains_properly(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedContainsProperly(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn covered_by(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCoveredBy(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn covers(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCovers(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn crosses(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCrosses(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn disjoint(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedDisjoint(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn intersects(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedIntersects(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn overlaps(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedOverlaps(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn touches(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedTouches(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
    pub fn within(&self, g2: &GeosGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedWithin(self.0 as *const c_void, g2.0) };
        if ret_val == 1 { true } else { false }
    }
}


#[cfg(test)]
mod test {
    use super::{init, GeosGeom, finish};

    #[test]
    fn test_new_geometry_from_wkt() {
        init();
        let pt_wkt = "POINT (2.5 2.5)";
        let geom = GeosGeom::new(pt_wkt);
        assert_eq!(true, geom.is_simple());
        assert_eq!(false, geom.is_empty());
        finish();
    }

    #[test]
    fn test_relationship(){
        init();
        let pt_wkt = "POINT (2.5 2.5)";
        let pt_geom = GeosGeom::new(pt_wkt);
        let polygon_geom = GeosGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");

        assert_eq!(true, polygon_geom.covers(&pt_geom));
        assert_eq!(true, polygon_geom.intersects(&pt_geom));
        assert_eq!(false, polygon_geom.covered_by(&pt_geom));
        assert_eq!(false, polygon_geom.equals(&pt_geom));

        assert_eq!(false, pt_geom.covers(&polygon_geom));
        assert_eq!(true, pt_geom.intersects(&polygon_geom));
        assert_eq!(true, pt_geom.covered_by(&polygon_geom));
        assert_eq!(false, pt_geom.equals(&polygon_geom));
        finish();
    }

    #[test]
    fn test_geom_creation_from_geoms(){
        init();
        let polygon_geom = GeosGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
        let new_geom = polygon_geom.buffer(100.0, 12);
        assert!(new_geom.area() > polygon_geom.area());
        assert_eq!(true, polygon_geom.covered_by(&new_geom));

        let g1 = new_geom.difference(&polygon_geom);
        let g2 = polygon_geom.sym_difference(&new_geom);
        let g3 = new_geom.sym_difference(&polygon_geom);
        assert_almost_eq(g1.area(), g2.area());
        assert_almost_eq(g2.area(), g3.area());
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.0001);
        assert!(f > 0.9999);
    }
}
