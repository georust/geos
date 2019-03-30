use crate::{CoordSeq, GGeom, GGeomTypes};
use c_vec::CVec;
use enums::{ByteOrder, Dimensions};
use error::{Error, GResult, PredicateType};
use ffi::*;
use functions::*;
use libc::{c_char, c_int, c_void, strlen};
use std::cell::RefCell;
use std::ffi::CStr;
use std::slice;
use std::mem;

pub struct GContextHandle {
    ptr: GEOSContextHandle_t,
    // TODO: maybe store the closure directly?
    notice_message: RefCell<*mut c_void>,
    // TODO: maybe store the closure directly?
    error_message: RefCell<*mut c_void>,
}

impl GContextHandle {
    /// Creates a new `GContextHandle`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// ```
    pub fn init() -> GResult<Self> {
        initialize();
        let ptr = unsafe { GEOS_init_r() };
        if ptr.is_null() {
            Err(Error::GenericError("GEOS_init_r failed".to_owned()))
        } else {
            Ok(GContextHandle {
                ptr,
                notice_message: RefCell::new(::std::ptr::null_mut()),
                error_message: RefCell::new(::std::ptr::null_mut()),
            })
        }
    }

    pub(crate) fn as_raw(&self) -> GEOSContextHandle_t {
        self.ptr
    }

    /// Allows to set a notice message handler.
    ///
    /// Passing [`None`] as parameter will unset this callback.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_notice_message_handler<'a>(&'a self, nf: Option<Box<dyn Fn(&str) + 'a>>) {
        let nf_data: Box<Option<Box<dyn Fn(&str) + 'a>>> = Box::new(nf);

        unsafe extern "C" fn message_handler_func<'a>(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'a>> = &*(data as *mut _);

            let bytes = slice::from_raw_parts(message as *const u8, strlen(message));
            if let Some(ref callback) = *callback {
                let s = CStr::from_bytes_with_nul_unchecked(bytes);
                callback(s.to_str().expect("invalid CStr -> &str conversion"))
            } else {
                panic!("cannot get closure...")
            }
        }

        let page_func = if nf_data.is_some() {
            Some(message_handler_func as _)
        } else {
            None
        };
        let nf_data = Box::into_raw(nf_data) as *mut _;
        let previous_ptr = self.notice_message.replace(nf_data);
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
        unsafe {
            GEOSContext_setNoticeMessageHandler_r(self.as_raw(), page_func, nf_data);
        }
    }

    /// Allows to set an error message handler.
    ///
    /// Passing [`None`] as parameter will unset this callback.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::GContextHandle;
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_error_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    pub fn set_error_message_handler<'a>(&'a self, ef: Option<Box<dyn Fn(&str) + 'a>>) {
        let ef_data: Box<Option<Box<dyn Fn(&str) + 'a>>> = Box::new(ef);

        unsafe extern "C" fn message_handler_func<'a>(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'a>> = &*(data as *mut _);

            let bytes = slice::from_raw_parts(message as *const u8, strlen(message));
            if let Some(ref callback) = *callback {
                let s = CStr::from_bytes_with_nul_unchecked(bytes);
                callback(s.to_str().expect("invalid CStr -> &str conversion"))
            } else {
                panic!("cannot get closure...")
            }
        }

        let page_func = if ef_data.is_some() {
            Some(message_handler_func as _)
        } else {
            None
        };
        let ef_data = Box::into_raw(ef_data) as *mut _;
        let previous_ptr = self.notice_message.replace(ef_data);
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
        unsafe {
            GEOSContext_setErrorMessageHandler_r(self.as_raw(), page_func, ef_data);
        }
    }

    /// Gets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, Dimensions};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(Dimensions::TwoD);
    /// assert!(context_handle.get_wkb_output_dimensions() == Dimensions::TwoD);
    /// ```
    pub fn get_wkb_output_dimensions(&self) -> Dimensions {
        Dimensions::from(unsafe { GEOS_getWKBOutputDims_r(self.as_raw()) })
    }

    /// Sets WKB output dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, Dimensions};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_output_dimensions(Dimensions::TwoD);
    /// assert!(context_handle.get_wkb_output_dimensions() == Dimensions::TwoD);
    /// ```
    pub fn set_wkb_output_dimensions(&self, dimensions: Dimensions) -> Dimensions {
        Dimensions::from(unsafe { GEOS_setWKBOutputDims_r(self.as_raw(), dimensions.into()) })
    }

    /// Gets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, ByteOrder};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn get_wkb_byte_order(&self) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_getWKBByteOrder_r(self.as_raw()) })
    }

    /// Sets WKB byte order.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, ByteOrder};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    ///
    /// context_handle.set_wkb_byte_order(ByteOrder::LittleEndian);
    /// assert!(context_handle.get_wkb_byte_order() == ByteOrder::LittleEndian);
    /// ```
    pub fn set_wkb_byte_order(&self, byte_order: ByteOrder) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_setWKBByteOrder_r(self.as_raw(), byte_order.into()) })
    }

    /// Convert [`GGeom`] from WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_wkb_buf(&point_geom)
    ///                             .expect("conversion to WKB failed");
    /// let new_geom = context_handle.geom_from_wkb_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_from_wkb_buf(&self, wkb: &[u8]) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeomFromWKB_buf_r(self.as_raw(), wkb.as_ptr(), wkb.len())) }
    }

    /// Convert [`GGeom`] to WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_wkb_buf(&point_geom)
    ///                             .expect("conversion to WKB failed");
    /// let new_geom = context_handle.geom_from_wkb_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_to_wkb_buf(&self, g: &GGeom) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.as_raw(), g.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Convert [`GGeom`] from HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_hex_buf(&point_geom)
    ///                             .expect("conversion to HEX failed");
    /// let new_geom = context_handle.geom_from_hex_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_from_hex_buf(&self, hex: &[u8]) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeomFromHEX_buf_r(self.as_raw(), hex.as_ptr(), hex.len())) }
    }

    /// Convert [`GGeom`] to HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{GContextHandle, GGeom};
    ///
    /// let context_handle = GContextHandle::init().expect("invalid init");
    /// let point_geom = GGeom::new("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = context_handle.geom_to_hex_buf(&point_geom)
    ///                             .expect("conversion to HEX failed");
    /// let new_geom = context_handle.geom_from_hex_buf(wkb_buf.as_ref())
    ///                              .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn geom_to_hex_buf(&self, g: &GGeom) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.as_raw(), g.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    pub fn geom_is_valid(&self, g: &GGeom) -> bool {
        unsafe { GEOSisValid_r(self.as_raw(), g.as_raw()) == 1 }
    }

    pub fn geometry_type(&self, g: &GGeom) -> GGeomTypes {
        let type_geom = unsafe { GEOSGeomTypeId_r(self.as_raw(), g.as_raw()) as i32 };

        GGeomTypes::from(type_geom)
    }

    pub fn geom_area(&self, g: &GGeom) -> GResult<f64> {
        let mut n = 0.;

        if unsafe { GEOSArea_r(self.as_raw(), g.as_raw(), &mut n) } != 0 {
            Err(Error::GeosError("computing the area".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn geom_to_wkt(&self, g: &GGeom) -> String {
        unsafe { managed_string(GEOSGeomToWKT_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_is_ring(&self, g: &GGeom) -> GResult<bool> {
        let rv = unsafe { GEOSisRing_r(self.as_raw(), g.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn geom_intersects(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn geom_crosses(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn geom_disjoint(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn geom_touches(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn geom_overlaps(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn geom_within(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    pub fn geom_equals(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn geom_equals_exact(&self, g: &GGeom, g2: &GGeom, precision: f64) -> GResult<bool> {
        let ret_val = unsafe { GEOSEqualsExact_r(self.as_raw(), g.as_raw(), g2.as_raw(), precision) };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn geom_covers(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn geom_covered_by(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn geom_contains(&self, g: &GGeom, g2: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains_r(self.as_raw(), g.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    pub fn geom_is_empty(&self, g: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty_r(self.as_raw(), g.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn geom_is_simple(&self, g: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.as_raw(), g.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn geom_difference(&self, g: &GGeom, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSDifference_r(self.as_raw(), g.as_raw(), g2.as_raw())) }
    }

    pub fn geom_envelope(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSEnvelope_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_sym_difference(&self, g: &GGeom, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSSymDifference_r(self.as_raw(), g.as_raw(), g2.as_raw())) }
    }

    pub fn geom_union(&self, g: &GGeom, g2: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSUnion_r(self.as_raw(), g.as_raw(), g2.as_raw())) }
    }

    pub fn geom_get_centroid(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGetCentroid_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_create_point(&self, s: CoordSeq) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(
                GEOSGeom_createPoint_r(
                    self.as_raw(), GEOSCoordSeq_clone(s.as_raw())))
        }
    }

    pub fn geom_create_line_string(&self, s: CoordSeq) -> GResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLineString_r(self.as_raw(), s.as_raw()))
        }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn geom_create_linear_ring(&self, s: CoordSeq) -> GResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLinearRing_r(self.as_raw(), s.as_raw()))
        }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn geom_unary_union(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSUnaryUnion_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_voronoi(
        &self,
        g: &GGeom,
        envelope: Option<&GGeom>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<GGeom> {
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram_r(
                self.as_raw(),
                g.as_raw(),
                envelope
                    .map(|e| e.as_raw() as *const GEOSGeometry)
                    .unwrap_or(std::ptr::null()),
                tolerance,
                only_edges as c_int,
            );
            GGeom::new_from_raw(raw_voronoi)
        }
    }

    pub fn geom_normalize(&self, g: &mut GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize_r(self.as_raw(), g.as_raw_mut()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    pub fn geom_intersection(&self, g: &GGeom, other: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSIntersection_r(self.as_raw(), g.as_raw(), other.as_raw())) }
    }

    pub fn geom_convex_hull(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSConvexHull_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_boundary(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSBoundary_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_has_z(&self, g: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ_r(self.as_raw(), g.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn geom_is_closed(&self, g: &GGeom) -> GResult<bool> {
        let ret_val = unsafe { GEOSisClosed_r(self.as_raw(), g.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn geom_length(&self, g: &GGeom) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength_r(self.as_raw(), g.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn geom_distance(&self, g: &GGeom, other: &GGeom) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance_r(self.as_raw(), g.as_raw(), other.as_raw(), &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_distance_indexed(&self, g: &GGeom, other: &GGeom) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistanceIndexed_r(self.as_raw(), g.as_raw(), other.as_raw(),
                                            &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_hausdorff_distance(&self, g: &GGeom, other: &GGeom) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance_r(self.as_raw(), g.as_raw(), other.as_raw(),
                                              &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_hausdorff_distance_densify(
        &self,
        g: &GGeom,
        other: &GGeom,
        distance_frac: f64,
    ) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify_r(self.as_raw(), g.as_raw(), other.as_raw(),
                                                     distance_frac, &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_frechet_distance(&self, g: &GGeom, other: &GGeom) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance_r(self.as_raw(), g.as_raw(), other.as_raw(),
                                            &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_frechet_distance_densify(
        &self,
        g: &GGeom,
        other: &GGeom,
        distance_frac: f64,
    ) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify_r(self.as_raw(), g.as_raw(), other.as_raw(),
                                                   distance_frac, &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn geom_get_length(&self, g: &GGeom) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength_r(self.as_raw(), g.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }



    pub fn geom_snap(&self, g: &GGeom, other: &GGeom, tolerance: f64) -> GResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSSnap_r(self.as_raw(), g.as_raw(), other.as_raw(), tolerance))
        }
    }

    pub fn geom_extract_unique_points(&self, g: &GGeom) -> GResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeom_extractUniquePoints_r(self.as_raw(), g.as_raw())) }
    }

    pub fn geom_nearest_points(&self, g: &GGeom, other: &GGeom) -> GResult<CoordSeq> {
        unsafe {
            CoordSeq::new_from_raw(GEOSNearestPoints_r(self.as_raw(), g.as_raw(), other.as_raw()))
        }
    }
}

impl Drop for GContextHandle {
    fn drop<'a>(&'a mut self) {
        unsafe { GEOS_finish_r(self.as_raw()) };

        let previous_ptr = self.notice_message.replace(::std::ptr::null_mut());
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }

        let previous_ptr = self.error_message.replace(::std::ptr::null_mut());
        if !previous_ptr.is_null() {
            // We free the previous closure.
            let _callback: Box<Option<Box<dyn Fn(&str) + 'a>>> =
                unsafe { Box::from_raw(previous_ptr as *mut _) };
        }
    }
}
