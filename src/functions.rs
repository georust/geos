use crate::context_handle::with_context;
use crate::enums::*;
use crate::error::{Error, GResult};
use crate::geometry::Geometry;
use crate::{AsRawMut, ContextHandle, Geom};
use geos_sys::*;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;

// We need to cleanup only the char* from geos, the const char* are not to be freed.
// this has to be checked method by method in geos
// so we provide 2 method to wrap a char* to a string, one that manage (and thus free) the underlying char*
// and one that does not free it
pub(crate) unsafe fn unmanaged_string(ptr: NonNull<c_char>) -> GResult<String> {
    let c_str = CStr::from_ptr(ptr.as_ptr());
    String::from_utf8(c_str.to_bytes().to_vec())
        .map_err(|e| Error::GenericError(format!("unmanaged_string failed with {e}")))
}

pub(crate) unsafe fn managed_string(ptr: NonNull<c_char>, ctx: &ContextHandle) -> GResult<String> {
    let s = unmanaged_string(ptr);
    GEOSFree_r(ctx.as_raw(), ptr.as_ptr().cast());
    s
}

pub fn version() -> GResult<String> {
    unsafe {
        let Some(v) = NonNull::new(GEOSversion().cast_mut()) else {
            return Err(Error::GeosError(("GEOSversion", None)));
        };
        unmanaged_string(v)
    }
}

macro_rules! nullcheck {
    ($func:ident($ctx:ident.as_raw() $(, $($args:expr),* $(,)?)?)) => {{
        let result = $func($ctx.as_raw()$(, $($args),*)?);
        std::ptr::NonNull::new(result as *mut _).ok_or_else(|| {
            $crate::Error::GeosError((stringify!($func), $ctx.get_last_error()))
        })
    }};
}

macro_rules! errcheck {
    ($errval:expr, $func:ident($ctx:ident.as_raw() $(, $($args:expr),* $(,)?)?)) => {{
        let result = $func($ctx.as_raw()$(, $($args),*)?);
        if result == $errval {
            Err($crate::Error::GeosError((stringify!($func), $ctx.get_last_error())))
        } else {
            Ok(result)
        }
    }};
    ($($args:tt)*) => {
        errcheck!(0, $($args)*)
    };
}

macro_rules! predicate {
    ($($args:tt)*) => {
        Ok(errcheck!(2, $($args)*)? == 1)
    };
}

pub(crate) use errcheck;
pub(crate) use nullcheck;
pub(crate) use predicate;

pub(crate) fn check_same_geometry_type(geoms: &[Geometry], geom_type: GeometryTypes) -> bool {
    geoms.iter().all(|g| g.geometry_type() == Ok(geom_type))
}

pub(crate) fn create_multi_geom(
    mut geoms: Vec<Geometry>,
    output_type: GeometryTypes,
) -> GResult<Geometry> {
    let nb_geoms = geoms.len();
    let res = {
        let mut geoms: Vec<*mut GEOSGeometry> =
            geoms.iter_mut().map(AsRawMut::as_raw_mut).collect();
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSGeom_createCollection_r(
                ctx.as_raw(),
                output_type.into(),
                geoms.as_mut_ptr().cast(),
                nb_geoms as _,
            ))?;
            Ok(Geometry::new_from_raw(ptr))
        })
    };

    // we'll transfert the ownership of the ptr to the new Geometry,
    // so the old one needs to forget their c ptr to avoid double cleanup
    for g in geoms {
        std::mem::forget(g);
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
        let ret = errcheck!(
            2,
            GEOSOrientationIndex_r(ctx.as_raw(), ax, ay, bx, by, px, py)
        )?;
        Orientation::try_from(ret)
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

        let ret = errcheck!(GEOSSegmentIntersection_r(
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
        ))?;
        Ok((ret != -1).then_some((cx, cy)))
    })
}
