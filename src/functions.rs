use enums::*;
use error::{Error, GResult, PredicateType};
use ffi::*;
use geom::GGeom;
use libc::{c_char, c_double, c_uint, c_void};
use std::ffi::CStr;
use std::sync::Arc;
use std::str;
use crate::{GContextHandle, AsRaw, ContextHandling};
use context_handle::PtrWrap;

// We need to cleanup only the char* from geos, the const char* are not to be freed.
// this has to be checked method by method in geos
// so we provide 2 method to wrap a char* to a string, one that manage (and thus free) the underlying char*
// and one that does not free it
pub(crate) unsafe fn unmanaged_string(raw_ptr: *const c_char) -> String {
    let c_str = CStr::from_ptr(raw_ptr);
    str::from_utf8(c_str.to_bytes()).unwrap().to_string()
}

pub(crate) unsafe fn managed_string(raw_ptr: *mut c_char, context: &GContextHandle) -> String {
    let s = unmanaged_string(raw_ptr);
    GEOSFree_r(context.as_raw(), raw_ptr as *mut c_void);
    s
}

#[allow(dead_code)]
pub fn clip_by_rect<'a>(g: &GGeom<'a>, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GResult<GGeom<'a>> {
    unsafe {
        let context = g.clone_context();
        let ptr = GEOSClipByRect_r(
            context.as_raw(),
            g.as_raw(),
            xmin as c_double,
            ymin as c_double,
            xmax as c_double,
            ymax as c_double,
        );
        GGeom::new_from_raw(ptr, context)
    }
}

pub fn version() -> String {
    unsafe { unmanaged_string(GEOSversion()) }
}

pub(crate) fn check_geos_predicate(val: i32, p: PredicateType) -> GResult<bool> {
    match val {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::GeosFunctionError(p, val)),
    }
}

pub(crate) fn check_ret(val: i32, p: PredicateType) -> GResult<()> {
    match val {
        1 => Ok(()),
        _ => Err(Error::GeosFunctionError(p, val)),
    }
}

pub(crate) fn check_same_geometry_type(geoms: &[GGeom], geom_type: GGeomTypes) -> bool {
    geoms.iter().all(|g| g.geometry_type() == geom_type)
}

pub(crate) fn create_multi_geom<'a>(mut geoms: Vec<GGeom<'a>>, output_type: GGeomTypes) -> GResult<GGeom<'a>> {
    let nb_geoms = geoms.len();
    let context = if geoms.is_empty() {
        match GContextHandle::init() {
            Ok(ch) => Arc::new(ch),
            _ => return Err(Error::GenericError("GEOS_init_r failed".to_owned())),
        }
    } else {
        geoms[0].clone_context()
    };
    let res = {
        let mut geoms: Vec<*mut GEOSGeometry> = geoms.iter_mut().map(|g| g.as_raw()).collect();
        unsafe {
            let ptr = GEOSGeom_createCollection_r(
                context.as_raw(),
                output_type.into(),
                geoms.as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_geoms as c_uint,
            );
            GGeom::new_from_raw(ptr, context)
        }
    };

    // we'll transfert the ownership of the ptr to the new GGeom,
    // so the old one needs to forget their c ptr to avoid double cleanup
    for g in geoms.iter_mut() {
        g.ptr = PtrWrap(::std::ptr::null_mut());
    }

    res
}

pub fn orientation_index(ax: f64, ay: f64, bx: f64, by: f64, px: f64, py: f64) -> Result<Orientation, &'static str> {
    unsafe {
        Orientation::try_from(GEOSOrientationIndex(ax, ay, bx, by, px, py))
    }
}

#[cfg(test)]
mod test {
    use super::check_geos_predicate;
    use error::PredicateType;

    #[test]
    fn check_geos_predicate_ok_test() {
        assert_eq!(
            check_geos_predicate(0, PredicateType::Intersects).unwrap(),
            false
        );
    }

    #[test]
    fn check_geos_predicate_ko_test() {
        assert_eq!(
            check_geos_predicate(1, PredicateType::Intersects).unwrap(),
            true
        );
    }

    #[test]
    fn check_geos_predicate_err_test() {
        let r = check_geos_predicate(42, PredicateType::Intersects);
        let e = r.err().unwrap();

        assert_eq!(
            format!("{}", e),
            "error while calling libgeos method Intersects (error number = 42)".to_string()
        );
    }
}
