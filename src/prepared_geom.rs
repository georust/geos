use crate::{GGeom, GResult};
use error::PredicateType;
use ffi::*;
use functions::*;
use std::ptr::NonNull;

pub struct PreparedGGeom(NonNull<GEOSPreparedGeometry>);

impl Drop for PreparedGGeom {
    fn drop(&mut self) {
        unsafe { GEOSPreparedGeom_destroy(self.0.as_mut()) };
    }
}

impl PreparedGGeom {
    pub fn new(g: &GGeom) -> PreparedGGeom {
        PreparedGGeom(NonNull::new(unsafe { GEOSPrepare(g.as_raw()) }).unwrap())
    }
    pub fn contains(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedContains(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedContains)
    }
    pub fn contains_properly(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedContainsProperly(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedContainsProperly)
    }
    pub fn covered_by(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedCoveredBy(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedCoveredBy)
    }
    pub fn covers(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedCovers(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedCovers)
    }
    pub fn crosses(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedCrosses(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedCrosses)
    }
    pub fn disjoint(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedDisjoint(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedDisjoint)
    }
    pub fn intersects(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedIntersects(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedIntersects)
    }
    pub fn overlaps(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedOverlaps(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedOverlaps)
    }
    pub fn touches(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedTouches(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedTouches)
    }
    pub fn within(&self, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSPreparedWithin(self.0.as_ref(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::PreparedWithin)
    }
}
