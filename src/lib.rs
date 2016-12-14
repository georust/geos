#![crate_name="geos"]
#![crate_type="lib"]

extern crate libc;

use libc::{c_char, c_void, c_int, c_double};
use std::ffi::{CString, CStr};
use std::{str, ptr};

enum GEOSGeomTypes {
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

#[link(name = "geos_c")]
extern {
    fn initGEOS() -> *mut c_void;
    fn GEOSversion() -> *const c_char;
    fn finishGEOS() -> *mut c_void;
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
    fn GEOSEqualsExact(g1: *const c_void, g2: *const c_void, tolerance: *const c_double) -> c_int;
    fn GEOSCovers(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSCoveredBy(g1: *const c_void, g2: *const c_void) -> c_int;
    fn GEOSBuffer(g: *const c_void, width: c_double, quadsegs: c_int) -> *const c_void;
    fn GEOSEnvelope(g: *const c_void) -> *const c_void;
    fn GEOSConvexHull(g: *const c_void) -> *const c_void;
    fn GEOSBounday(g: *const c_void) -> *const c_void;
    fn GEOSGetCentroid(g: *const c_void) -> *const c_void;
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

pub struct gg(*const c_void);

impl Drop for gg {
    fn drop(&mut self) {
        if self.0.is_null() { return; }
        self.0 = ptr::null();
    }
}

impl Clone for gg {
    fn clone(&self) -> gg {
        gg(self.0)
    }
}

impl gg {
    pub fn new(wkt: &str) -> gg {
        let c_str = CString::new(wkt).unwrap();
        unsafe { gg(GEOSGeomFromWKT(c_str.as_ptr())) }
    }

    pub fn toWKT(&self) -> CString {
        unsafe { CStr::from_ptr(GEOSGeomToWKT(self.0)).to_owned() }
    }

    pub fn Area(&self) -> f64 {
        let n_mut_ref = &mut 0.0;
        let n_mut_ptr: *mut c_double = n_mut_ref;
        let ret_val = unsafe { GEOSArea(self.0, n_mut_ptr) };
        return *n_mut_ref;
    }

    pub fn Length(&self) -> f64 {
        let n_mut_ref = &mut 0.0;
        let n_mut_ptr: *mut c_double = n_mut_ref;
        let ret_val = unsafe { GEOSLength(self.0, n_mut_ptr) };
        return *n_mut_ref;
    }

    pub fn Intersects(&self, g2: &gg) -> bool {
        let ret_val = unsafe {GEOSIntersects(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn Within(&self, g2: &gg) -> bool {
        let ret_val = unsafe {GEOSWithin(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn Covers(&self, g2: &gg) -> bool {
        let ret_val = unsafe {GEOSCovers(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn Contains(&self, g2: &gg) -> bool {
        let ret_val = unsafe {GEOSContains(self.0, g2.0 as *const c_void)};
        return if ret_val == 1 { true } else { false };
    }

    pub fn Buffer(&self, width: f64, quadsegs: i32) -> gg {
        let ret_val = unsafe {GEOSBuffer(self.0, width as c_double, quadsegs as c_int)};
        if ret_val.is_null(){ panic!("Invalid geometry"); }
        gg(ret_val)
    }
}
