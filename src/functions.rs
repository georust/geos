use crate::context_handle::{with_context, PtrWrap};
use crate::enums::*;
use crate::error::{Error, GResult, PredicateType};
use crate::geometry::Geometry;
use crate::{AsRawMut, ContextHandle, Geom};
use geos_sys::*;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::str;

// We need to cleanup only the char* from geos, the const char* are not to be freed.
// this has to be checked method by method in geos
// so we provide 2 method to wrap a char* to a string, one that manage (and thus free) the underlying char*
// and one that does not free it
pub(crate) unsafe fn unmanaged_string(raw_ptr: *const c_char, caller: &str) -> GResult<String> {
    if raw_ptr.is_null() {
        return Err(Error::NoConstructionFromNullPtr(format!(
            "{caller}::unmanaged_string",
        )));
    }
    let c_str = CStr::from_ptr(raw_ptr);
    match str::from_utf8(c_str.to_bytes()) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(Error::GenericError(format!(
            "{caller}::unmanaged_string failed: {e}",
        ))),
    }
}

pub(crate) unsafe fn managed_string(
    raw_ptr: *mut c_char,
    ctx: &ContextHandle,
    caller: &str,
) -> GResult<String> {
    if raw_ptr.is_null() {
        return Err(Error::NoConstructionFromNullPtr(format!(
            "{caller}::managed_string",
        )));
    }
    let s = unmanaged_string(raw_ptr, caller);
    GEOSFree_r(ctx.as_raw(), raw_ptr as *mut _);
    s
}

#[allow(dead_code)]
pub fn clip_by_rect<G: Geom>(
    g: &G,
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
) -> GResult<Geometry> {
    with_context(|ctx| unsafe {
        let ptr = GEOSClipByRect_r(ctx.as_raw(), g.as_raw(), xmin, ymin, xmax, ymax);
        Geometry::new_from_raw(ptr, ctx, "clip_by_rect")
    })
}

pub fn version() -> GResult<String> {
    unsafe { unmanaged_string(GEOSversion(), "version") }
}

pub(crate) fn check_geos_predicate(val: i8, p: PredicateType) -> GResult<bool> {
    match val {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::GeosFunctionError(p, val as _)),
    }
}

pub(crate) fn check_ret(val: i32, p: PredicateType) -> GResult<()> {
    match val {
        1 => Ok(()),
        _ => Err(Error::GeosFunctionError(p, val)),
    }
}

pub(crate) fn check_same_geometry_type(geoms: &[Geometry], geom_type: GeometryTypes) -> bool {
    geoms.iter().all(|g| g.geometry_type() == geom_type)
}

pub(crate) fn create_multi_geom(
    mut geoms: Vec<Geometry>,
    output_type: GeometryTypes,
) -> GResult<Geometry> {
    let nb_geoms = geoms.len();
    let res = {
        let mut geoms: Vec<*mut GEOSGeometry> = geoms.iter_mut().map(|g| g.as_raw_mut()).collect();
        with_context(|ctx| unsafe {
            let ptr = GEOSGeom_createCollection_r(
                ctx.as_raw(),
                output_type.into(),
                geoms.as_mut_ptr() as *mut _,
                nb_geoms as _,
            );
            Geometry::new_from_raw(ptr, ctx, "create_multi_geom")
        })
    };

    // we'll transfert the ownership of the ptr to the new Geometry,
    // so the old one needs to forget their c ptr to avoid double cleanup
    for g in geoms.iter_mut() {
        g.ptr = PtrWrap(::std::ptr::null_mut());
    }

    res
}

pub fn orientation_index(
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    px: f64,
    py: f64,
) -> GResult<Orientation> {
    with_context(|ctx| unsafe {
        Orientation::try_from(GEOSOrientationIndex_r(ctx.as_raw(), ax, ay, bx, by, px, py))
    })
}

/// Returns [`None`] if the segments don't intersect, otherwise returns `Some(x_pos, y_pos)`.
///
/// Available using the `v3_7_0` feature.
#[cfg(any(feature = "v3_7_0", feature = "dox"))]
#[allow(clippy::too_many_arguments)]
pub fn segment_intersection(
    ax0: f64,
    ay0: f64,
    ax1: f64,
    ay1: f64,
    bx0: f64,
    by0: f64,
    bx1: f64,
    by1: f64,
) -> GResult<Option<(f64, f64)>> {
    with_context(|ctx| unsafe {
        let mut cx = 0.;
        let mut cy = 0.;

        let ret = GEOSSegmentIntersection_r(
            ctx.as_raw(),
            ax0,
            ay0,
            ax1,
            ay1,
            bx0,
            by0,
            bx1,
            by1,
            &mut cx,
            &mut cy,
        );
        if ret == -1 {
            Ok(None)
        } else if ret == 0 {
            Ok(Some((cx, cy)))
        } else {
            Err(Error::GenericError(
                "GEOSSegmentIntersection_r failed".to_owned(),
            ))
        }
    })
}

#[cfg(test)]
mod test {
    use super::check_geos_predicate;
    use crate::error::PredicateType;

    #[test]
    fn check_geos_predicate_ok_test() {
        assert!(!check_geos_predicate(0, PredicateType::Intersects).unwrap());
    }

    #[test]
    fn check_geos_predicate_ko_test() {
        assert!(check_geos_predicate(1, PredicateType::Intersects).unwrap());
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
