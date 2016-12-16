#![crate_name="geos"]
#![crate_type="lib"]

extern crate libc;

use libc::{c_char, c_void, c_int, c_uint, c_double, size_t};
use std::sync::{Once, ONCE_INIT};
use std::ffi::{CString, CStr};
use std::{str, ptr};


#[link(name = "geos_c")]
extern {
    fn initGEOS() -> *mut c_void;
    fn GEOSversion() -> *const c_char;
    fn finishGEOS() -> *mut c_void;

    fn GEOSPrepare(g: *const c_void) -> *mut c_void;
    fn GEOSGeom_destroy(g: *mut c_void);
    fn GEOSGeom_clone(g: *const c_void) -> *mut c_void;

    fn GEOSCoordSeq_create(size: c_uint, dims: c_uint) -> *mut c_void;
    fn GEOSCoordSeq_destroy(s: *mut c_void);
    fn GEOSCoordSeq_clone(s: *const c_void) -> *mut c_void;
    fn GEOSCoordSeq_setX(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setY(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setZ(s: *mut c_void, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_getX(s: *const c_void, idx: c_uint, val: *mut c_double) -> c_int;
    fn GEOSCoordSeq_getY(s: *const c_void, idx: c_uint, val: *mut c_double) -> c_int;
    fn GEOSCoordSeq_getZ(s: *const c_void, idx: c_uint, val: *mut c_double) -> c_int;

    // Geometry must be a LineString, LinearRing or Point :
    fn GEOSGeom_getCoordSeq(g: *const c_void) -> *mut c_void;

    // Geometry constructor :
    fn GEOSGeom_createPoint(s: *const c_void) -> *mut c_void;
    fn GEOSGeom_createLineString(s: *const c_void) -> *mut c_void;
    fn GEOSGeom_createLinearRing(s: *const c_void) -> *mut c_void;

    // Functions acting on GEOSGeometry :
    fn GEOSisEmpty(g: *const c_void) -> c_int;
    fn GEOSisSimple(g: *const c_void) -> c_int;
    fn GEOSisRing(g: *const c_void) -> c_int;
    fn GEOSHasZ(g: *const c_void) -> c_int;
    fn GEOSisClosed(g: *const c_void) -> c_int;

    fn GEOSGeomFromWKT(wkt: *const c_char) -> *mut c_void;
    fn GEOSGeomToWKT(g: *const c_void) -> *const c_char;
    fn GEOSGeomFromWKB_buf(wkb: *const u8, size: size_t) -> *mut c_void;
    fn GEOSGeomToWKB_buf(g: *const c_void, size: *mut size_t) -> *const c_char;
    fn GEOSGeomTypeId(g: *const c_void) -> c_int;
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

    fn GEOSBuffer(g: *const c_void, width: c_double, quadsegs: c_int) -> *mut c_void;
    fn GEOSEnvelope(g: *const c_void) -> *mut c_void;
    fn GEOSConvexHull(g: *const c_void) -> *mut c_void;
    fn GEOSBoundary(g: *const c_void) -> *mut c_void;
    fn GEOSGetCentroid(g: *const c_void) -> *mut c_void;
    fn GEOSSymDifference(g1: *const c_void, g2: *const c_void) -> *mut c_void;
    fn GEOSDifference(g1: *const c_void, g2: *const c_void) -> *mut c_void;
    // fn GEOSClipByRect(g: *const c_void, xmin: c_double, ymin: c_double, xmax: c_double, ymax: c_double) -> *mut c_void;
    // fn GEOSSnap(g1: *const c_void, g2: *const c_void, tolerance: c_double) -> *mut c_void;
    fn GEOSGeom_extractUniquePoints(g: *const c_void) -> *mut c_void;

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
    GEOS_POINT = 0,
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

pub fn _Point(s: &CoordSeq) -> GGeom {
    GGeom::new_from_c_obj(unsafe {GEOSGeom_createPoint(s.0 as *const c_void) })
}

pub fn _LineString(s: &CoordSeq) -> GGeom {
    GGeom::new_from_c_obj(unsafe {GEOSGeom_createLineString(s.0 as *const c_void) })
}

// pub fn Snap(g1: &GGeom, g2: &GGeom, tolerance: f64) -> GGeom {
//     GGeom::new_from_c_obj(unsafe { GEOSSnap(g1.0, g2.0, tolerance as c_double) })
// }
//
// pub fn ClipByRect(g: &GGeom, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GGeom {
//     GGeom::new_from_c_obj(unsafe { GEOSClipByRect(g.0, xmin as c_double, ymin as c_double, xmax as c_double, ymax as c_double)})
// }

pub fn version() -> String {
    unsafe {
        _string(GEOSversion())
    }
}

fn initialize() {
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        initGEOS();
        assert_eq!(libc::atexit(cleanup), 0);
    });

    extern fn cleanup() {
        unsafe { finishGEOS(); }
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
        initialize();
        CoordSeq(unsafe { GEOSCoordSeq_create(size as c_uint, dims as c_uint) })
    }
    pub fn set_x(&self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setX(self.0 as *mut c_void, idx as c_uint, val as c_double) };
        return ret_val;
    }
    pub fn set_y(&self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setY(self.0 as *mut c_void, idx as c_uint, val as c_double) };
        return ret_val;
    }
    pub fn set_z(&self, idx: u32, val: f64) -> i32 {
        let ret_val = unsafe { GEOSCoordSeq_setZ(self.0 as *mut c_void, idx as c_uint, val as c_double) };
        return ret_val;
    }

    pub fn get_x(&self, idx: u32) -> f64 {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe { GEOSCoordSeq_getX(self.0 as *const c_void, idx as c_uint, n_mut_ref as *mut c_double) };
        if ret_val == 0 {panic!("Error when get coordinates from CoordSeq");}
        return *n_mut_ref;
    }

    pub fn get_y(&self, idx: u32) -> f64 {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe { GEOSCoordSeq_getY(self.0 as *const c_void, idx as c_uint, n_mut_ref as *mut c_double) };
        if ret_val == 0 {panic!("Error when get coordinates from CoordSeq");}
        return *n_mut_ref;
    }

    pub fn get_z(&self, idx: u32) -> f64 {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe { GEOSCoordSeq_getZ(self.0 as *const c_void, idx as c_uint, n_mut_ref as *mut c_double) };
        if ret_val == 0 {panic!("Error when get coordinates from CoordSeq");}
        return *n_mut_ref;
    }
}

pub struct GGeom {
    c_obj: *mut c_void,
    pub area: f64,
    pub _type: i32,
}

impl Drop for GGeom {
    fn drop(&mut self){
        unsafe { GEOSGeom_destroy(self.c_obj as *mut c_void)};
        self.c_obj = ptr::null_mut();
    }
}

impl Clone for GGeom {
    fn clone(&self) -> GGeom {
        let n_obj = unsafe { GEOSGeom_clone(self.c_obj as *const c_void)};
        GGeom {c_obj: n_obj, area: self.area, _type: self._type}
    }
}

impl GGeom {
    pub fn new(wkt: &str) -> GGeom {
        initialize();
        let c_str = CString::new(wkt).unwrap();
        let obj = unsafe { GEOSGeomFromWKT(c_str.as_ptr()) };
        GGeom::new_from_c_obj(obj)
    }

    // pub fn new_from_wkb(wkb: &[u8]) -> GGeom {
    //     initialize();
    //     let strr = CString::new(wkb).unwrap();
    //     let t = strr.as_bytes();
    //     let obj = unsafe { GEOSGeomFromWKB_buf(t.as_ptr(), t.len() as size_t) };
    //     GGeom::new_from_c_obj(obj)
    // }

    fn new_from_c_obj(g: *mut c_void) -> GGeom {
        if g.is_null(){ panic!("Invalid geometry"); }
        let area = GGeom::_area(g as *const c_void);
        let type_geom = unsafe { GEOSGeomTypeId(g as *const c_void) as i32};
        GGeom {c_obj: g, area: area, _type: type_geom}
    }

    fn _area(obj: *const c_void) -> f64 {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe { GEOSArea(obj, n_mut_ref as *mut c_double) };
        return *n_mut_ref;
    }

    // pub fn get_type(&self) -> GEOSGeomTypes {
    //     match(self._type){
    //         }
    //     }

    pub fn to_wkt(&self) -> CString {
        unsafe { CStr::from_ptr(GEOSGeomToWKT(self.c_obj as *const c_void)).to_owned() }
    }

    pub fn to_wkb(&self) -> CString {
        let mut dstlen: size_t = 0 as size_t;
        unsafe { CStr::from_ptr(GEOSGeomToWKB_buf(self.c_obj as *const c_void, &mut dstlen)).to_owned() }
    }

    pub fn intersects(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSIntersects(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn crosses(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSCrosses(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn within(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSWithin(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn equals(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSEquals(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn covers(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSCovers(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn covered_by(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSCoveredBy(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn contains(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe {GEOSContains(self.c_obj as *const c_void, g2.c_obj as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GGeom {
        GGeom::new_from_c_obj(unsafe {GEOSBuffer(self.c_obj as *const c_void, width as c_double, quadsegs as c_int)})
    }

    pub fn is_empty(&self) -> bool {
        let ret_val = unsafe { GEOSisEmpty(self.c_obj as *const c_void) };
        return if ret_val == 1 { true } else { false };
    }

    pub fn is_simple(&self) -> bool {
        let ret_val = unsafe { GEOSisSimple(self.c_obj as *const c_void) };
        return if ret_val == 1 { true } else { false };
    }
    pub fn difference(&self, g2: &GGeom) -> GGeom {
        GGeom::new_from_c_obj(unsafe {GEOSDifference(self.c_obj as *const c_void, g2.c_obj as *const c_void)})
    }

    pub fn sym_difference(&self, g2: &GGeom) -> GGeom {
        GGeom::new_from_c_obj(unsafe {GEOSSymDifference(self.c_obj as *const c_void, g2.c_obj as *const c_void)})
    }

    pub fn get_centroid(&self) -> GGeom {
        GGeom::new_from_c_obj(unsafe {GEOSGetCentroid(self.c_obj as *const c_void)})
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
    pub fn new(g: &GGeom) -> GeosPrepGeom {
        GeosPrepGeom(unsafe { GEOSPrepare(g.c_obj)})
    }
    pub fn contains(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedContains(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn contains_properly(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedContainsProperly(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn covered_by(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCoveredBy(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn covers(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCovers(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn crosses(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedCrosses(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn disjoint(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedDisjoint(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn intersects(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedIntersects(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn overlaps(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedOverlaps(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn touches(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedTouches(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
    pub fn within(&self, g2: &GGeom) -> bool {
        let ret_val = unsafe { GEOSPreparedWithin(self.0 as *const c_void, g2.c_obj as *const c_void) };
        if ret_val == 1 { true } else { false }
    }
}


#[cfg(test)]
mod test {
    use super::{GGeom, GeosPrepGeom, GEOSGeomTypes};

    #[test]
    fn test_new_geometry_from_wkt() {
        let geom = GGeom::new("POINT (2.5 2.5)");
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, geom._type);
        assert_eq!(true, geom.is_simple());
        assert_eq!(false, geom.is_empty());
        let line_geom = GGeom::new("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)");
        assert_eq!(GEOSGeomTypes::GEOS_LINESTRING as i32, line_geom._type);
    }

    #[test]
    fn test_relationship(){
        let pt_geom = GGeom::new("POINT (2.5 2.5)");
        let line_geom = GGeom::new("LINESTRING(1 1,10 50,20 25)");
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");

        assert_eq!(true, polygon_geom.covers(&pt_geom));
        assert_eq!(true, polygon_geom.intersects(&pt_geom));
        assert_eq!(false, polygon_geom.covered_by(&pt_geom));
        assert_eq!(false, polygon_geom.equals(&pt_geom));
        assert_eq!(false, polygon_geom.within(&pt_geom));

        assert_eq!(false, pt_geom.covers(&polygon_geom));
        assert_eq!(true, pt_geom.intersects(&polygon_geom));
        assert_eq!(true, pt_geom.covered_by(&polygon_geom));
        assert_eq!(false, pt_geom.equals(&polygon_geom));
        assert_eq!(true, pt_geom.within(&polygon_geom));

        assert_eq!(false, line_geom.covers(&pt_geom));
        assert_eq!(false, line_geom.intersects(&pt_geom));
        assert_eq!(false, line_geom.covered_by(&pt_geom));
        assert_eq!(false, pt_geom.covered_by(&line_geom));
        assert_eq!(true, line_geom.intersects(&polygon_geom));
        assert_eq!(true, line_geom.crosses(&polygon_geom));
        assert_eq!(false, line_geom.equals(&pt_geom));
    }

    #[test]
    fn test_geom_creation_from_geoms(){
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
        let new_geom = polygon_geom.buffer(100.0, 12);
        let g1 = new_geom.difference(&polygon_geom);
        let g2 = polygon_geom.sym_difference(&new_geom);
        let g3 = new_geom.sym_difference(&polygon_geom);
        assert_almost_eq(g1.area, g2.area);
        assert_almost_eq(g2.area, g3.area);
        let g4 = g3.get_centroid();
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, g4._type);
        let g5 = g4.buffer(200.0, 12);
        assert!(g5.area > g4.area);
        assert_eq!(GEOSGeomTypes::GEOS_POLYGON as i32, g5._type);
    }

    #[test]
    fn rest_preapred_geoms() {
        let g1 = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
        let g2 = GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))");
        let pg1 = GeosPrepGeom::new(&g1);
        assert_eq!(true, pg1.intersects(&g2));
        assert_eq!(true, pg1.contains(&g2.get_centroid()));
        let vec_geoms = vec![
            GGeom::new("POINT (1.3 2.4)"),
            GGeom::new("POINT (2.1 0.3)"),
            GGeom::new("POINT (3.1 4.7)"),
            GGeom::new("POINT (0.4 4.1)")
            ];
        for geom in &vec_geoms {
            assert_eq!(true, pg1.intersects(&geom));
        }
    }


    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.0001);
        assert!(f > 0.9999);
    }
}
