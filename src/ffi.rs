use libc::{atexit, c_char, c_uchar, c_double, c_int, c_uint, c_void, size_t, strlen};
use std::sync::{Once, ONCE_INIT};
use std::ffi::{CStr, CString};
use std::{self, slice, str, mem};
use std::ptr::NonNull;
use error::{Error, Result as GeosResult, PredicateType};
use num_traits::FromPrimitive;

#[repr(C)]
struct GEOSWKTReader { private: [u8; 0] }
#[repr(C)]
struct GEOSWKTWriter { private: [u8; 0] }
#[repr(C)]
struct GEOSPreparedGeometry { private: [u8; 0] }
#[repr(C)]
struct GEOSCoordSequence { private: [u8; 0] }
#[repr(C)]
struct GEOSGeometry { private: [u8; 0] }
#[repr(C)]
struct GEOSContextHandle_HS { private: [u8; 0] }
#[allow(non_camel_case_types)]
type GEOSContextHandle_t = *mut GEOSContextHandle_HS;
#[allow(non_camel_case_types)]
type GEOSMessageHandler_r = Option<unsafe extern "C" fn(message: *const c_char,
                                                        userdata: *mut c_void)>;

#[link(name = "geos_c")]
extern "C" {
    fn initGEOS() -> *mut c_void;
    fn GEOSversion() -> *const c_char;
    fn finishGEOS() -> *mut c_void;

    // API for reading WKT:
    fn GEOSWKTReader_create() -> *mut GEOSWKTReader;
    fn GEOSWKTReader_destroy(reader: *mut GEOSWKTReader);
    fn GEOSWKTReader_read(reader: *mut GEOSWKTReader, wkt: *const c_char) -> *mut GEOSGeometry;

    // API for writing WKT:
    fn GEOSWKTWriter_create() -> *mut GEOSWKTWriter;
    fn GEOSWKTWriter_destroy(writer: *mut GEOSWKTWriter);
    fn GEOSWKTWriter_write(writer: *mut GEOSWKTWriter, g: *const GEOSGeometry) -> *mut c_char;
    fn GEOSWKTWriter_setRoundingPrecision(writer: *mut GEOSWKTWriter, precision: c_int);

    fn GEOSFree(buffer: *mut c_void);

    fn GEOSPrepare(g: *const GEOSGeometry) -> *mut GEOSPreparedGeometry;
    fn GEOSGeom_destroy(g: *mut GEOSGeometry);
    fn GEOSGeom_clone(g: *const GEOSGeometry) -> *mut GEOSGeometry;

    fn GEOSCoordSeq_create(size: c_uint, dims: c_uint) -> *mut GEOSCoordSequence;
    fn GEOSCoordSeq_destroy(s: *mut GEOSCoordSequence);
    fn GEOSCoordSeq_clone(s: *const GEOSCoordSequence) -> *mut GEOSCoordSequence;
    fn GEOSCoordSeq_setX(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setY(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_setZ(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    fn GEOSCoordSeq_getX(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double) -> c_int;
    fn GEOSCoordSeq_getY(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double) -> c_int;
    fn GEOSCoordSeq_getZ(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double) -> c_int;
    fn GEOSCoordSeq_getSize(s: *const GEOSCoordSequence, val: *mut c_uint) -> c_int;

    // Geometry must be a LineString, LinearRing or Point:
    fn GEOSGeom_getCoordSeq(g: *const GEOSGeometry) -> *mut GEOSCoordSequence;

    // Geometry constructor:
    fn GEOSGeom_createPoint(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    fn GEOSGeom_createLineString(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    fn GEOSGeom_createLinearRing(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    fn GEOSGeom_createPolygon(
        shell: *mut GEOSGeometry,
        holes: *mut *mut GEOSGeometry,
        nholes: c_uint,
    ) -> *mut GEOSGeometry;
    fn GEOSGeom_createCollection(
        t: c_int,
        geoms: *mut *mut GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;

    // Functions acting on GEOSGeometry:
    fn GEOSisEmpty(g: *const GEOSGeometry) -> c_int;
    fn GEOSisSimple(g: *const GEOSGeometry) -> c_int;
    fn GEOSisRing(g: *const GEOSGeometry) -> c_int;
    #[allow(dead_code)]
    fn GEOSHasZ(g: *const GEOSGeometry) -> c_int;
    #[allow(dead_code)]
    fn GEOSisClosed(g: *const GEOSGeometry) -> c_int;
    fn GEOSisValid(g: *const GEOSGeometry) -> c_int;

    fn GEOSGeomToWKT(g: *const GEOSGeometry) -> *mut c_char;
    #[allow(dead_code)]
    fn GEOSGeomFromWKB_buf(wkb: *const u8, size: size_t) -> *mut GEOSGeometry;
    #[allow(dead_code)]
    fn GEOSGeomToWKB_buf(g: *const GEOSGeometry, size: *mut size_t) -> *mut u8;
    fn GEOSGeomTypeId(g: *const GEOSGeometry) -> c_int;
    fn GEOSArea(g: *const GEOSGeometry, area: *mut c_double) -> c_int;
    #[allow(dead_code)]
    fn GEOSLength(g: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    fn GEOSDisjoint(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSTouches(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSIntersects(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSCrosses(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSWithin(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSContains(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSOverlaps(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSEquals(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSEqualsExact(g1: *const GEOSGeometry, g2: *const GEOSGeometry, tolerance: c_double) -> c_int;
    fn GEOSCovers(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSCoveredBy(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_int;

    fn GEOSBuffer(g: *const GEOSGeometry, width: c_double, quadsegs: c_int) -> *mut GEOSGeometry;
    fn GEOSEnvelope(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    #[allow(dead_code)]
    fn GEOSConvexHull(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    #[allow(dead_code)]
    fn GEOSBoundary(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSGetCentroid(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSSymDifference(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSDifference(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSUnion(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSUnaryUnion(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSClipByRect(
        g: *const GEOSGeometry,
        xmin: c_double,
        ymin: c_double,
        xmax: c_double,
        ymax: c_double,
    ) -> *mut GEOSGeometry;
    #[allow(dead_code)]
    fn GEOSSnap(g1: *const GEOSGeometry, g2: *const GEOSGeometry, tolerance: c_double) -> *mut GEOSGeometry;
    #[allow(dead_code)]
    fn GEOSGeom_extractUniquePoints(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    fn GEOSVoronoiDiagram(g: *const GEOSGeometry,
                env: *const GEOSGeometry,
                tolerance: c_double,
                onlyEdges: c_int) -> *mut GEOSGeometry;
    fn GEOSNormalize(g: *mut GEOSGeometry) -> c_int;


    // Functions acting on GEOSPreparedGeometry:
    fn GEOSPreparedContains(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedContainsProperly(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedCoveredBy(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedCovers(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedCrosses(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedDisjoint(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedIntersects(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedOverlaps(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedTouches(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedWithin(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    fn GEOSPreparedGeom_destroy(g: *mut GEOSPreparedGeometry);

    fn GEOS_init_r() -> GEOSContextHandle_t;
    fn GEOS_finish_r(handle: GEOSContextHandle_t);
    fn GEOSContext_setNoticeMessageHandler_r(handle: GEOSContextHandle_t,
                                             nf: GEOSMessageHandler_r,
                                             userdata: *mut c_void) -> GEOSMessageHandler_r;
    fn GEOSContext_setErrorMessageHandler_r(handle: GEOSContextHandle_t,
                                            nf: GEOSMessageHandler_r,
                                            userdata: *mut c_void) -> GEOSMessageHandler_r;
    fn GEOS_getWKBOutputDims_r(handle: GEOSContextHandle_t) -> c_int;
    fn GEOS_setWKBOutputDims_r(handle: GEOSContextHandle_t, newDims: c_int) -> c_int;
    fn GEOS_getWKBByteOrder_r(handle: GEOSContextHandle_t) -> c_int;
    fn GEOS_setWKBByteOrder_r(handle: GEOSContextHandle_t, byteOrder: c_int) -> c_int;
    fn GEOSGeomFromWKB_buf_r(handle: GEOSContextHandle_t,
                             wkb: *const c_uchar,
                             size: size_t) -> *mut GEOSGeometry;
    fn GEOSGeomToWKB_buf_r(handle: GEOSContextHandle_t,
                           g: *const GEOSGeometry,
                           size: *mut size_t) -> *mut c_uchar;
    fn GEOSGeomFromHEX_buf_r(handle: GEOSContextHandle_t,
                             hex: *const c_uchar,
                             size: size_t) -> *mut GEOSGeometry;
    fn GEOSGeomToHEX_buf_r(handle: GEOSContextHandle_t,
                           g: *const GEOSGeometry,
                           size: *mut size_t) -> *mut c_uchar;
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Dimensions {
    TwoD,
    ThreeD,
    Other(u32),
}

impl From<c_int> for Dimensions {
    fn from(dimensions: c_int) -> Self {
        match dimensions {
            2 => Dimensions::TwoD,
            3 => Dimensions::ThreeD,
            x if x > 3 => Dimensions::Other(x as _),
            _ => panic!("dimensions must be > 1"),
        }
    }
}

impl Into<c_int> for Dimensions {
    fn into(self) -> c_int {
        match self {
            Dimensions::TwoD => 2,
            Dimensions::ThreeD => 3,
            Dimensions::Other(dim) => dim as _,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl From<c_int> for ByteOrder {
    fn from(dimensions: c_int) -> Self {
        match dimensions {
            0 => ByteOrder::BigEndian,
            _ => ByteOrder::LittleEndian,
        }
    }
}

impl Into<c_int> for ByteOrder {
    fn into(self) -> c_int {
        match self {
            ByteOrder::BigEndian => 0,
            ByteOrder::LittleEndian => 1,
        }
    }
}

#[repr(C)]
pub struct GContextHandle(GEOSContextHandle_t);

impl GContextHandle {
    pub fn init() -> Self {
        initialize();
        let ptr = unsafe { GEOS_init_r() };
        if ptr.is_null() {
            panic!("GEOS_init_r failed")
        } else {
            GContextHandle(ptr)
        }
    }

    pub fn set_notice_message_handler(&self, nf: Option<Box<dyn Fn(&str) + 'static>>) {
        let nf_data: Box<Option<Box<dyn Fn(&str) + 'static>>> = Box::new(nf);

        unsafe extern "C" fn message_handler_func(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'static>> = &*(data as *mut _);

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
        unsafe {
            // We can't get back the previous closure. It'll be lost in the depth of the memory
            // foreveeeeer.
            GEOSContext_setNoticeMessageHandler_r(self.0,
                                                  page_func,
                                                  Box::into_raw(nf_data) as *mut _);
        }
    }

    pub fn set_error_message_handler(&self, ef: Option<Box<dyn Fn(&str) + 'static>>) {
        let nf_data: Box<Option<Box<dyn Fn(&str) + 'static>>> = Box::new(ef);

        unsafe extern "C" fn message_handler_func(message: *const c_char, data: *mut c_void) {
            let callback: &Option<Box<dyn Fn(&str) + 'static>> = &*(data as *mut _);

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
        unsafe {
            // We can't get back the previous closure. It'll be lost in the depth of the memory
            // foreveeeeer.
            GEOSContext_setErrorMessageHandler_r(self.0,
                                                 page_func,
                                                 Box::into_raw(nf_data) as *mut _);
        }
    }

    #[allow(non_snake_case)]
    pub fn get_WKB_output_dimensions(&self) -> Dimensions {
        Dimensions::from(unsafe { GEOS_getWKBOutputDims_r(self.0) })
    }

    #[allow(non_snake_case)]
    pub fn set_WKB_output_dimensions(&self, dimensions: Dimensions) -> Dimensions {
        Dimensions::from(unsafe { GEOS_setWKBOutputDims_r(self.0, dimensions.into()) })
    }

    #[allow(non_snake_case)]
    pub fn get_WKB_byte_order(&self) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_getWKBByteOrder_r(self.0) })
    }

    #[allow(non_snake_case)]
    pub fn set_WKB_byte_order(&self, byte_order: ByteOrder) -> ByteOrder {
        ByteOrder::from(unsafe { GEOS_setWKBByteOrder_r(self.0, byte_order.into()) })
    }

    #[allow(non_snake_case)]
    pub fn geom_from_WKB_buf(&self, wkb: &[u8]) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSGeomFromWKB_buf_r(self.0, wkb.as_ptr(), wkb.len()))
        }
    }

    #[allow(non_snake_case)]
    pub fn geom_to_WKB_buf(&self, g: &GGeom) -> Option<Vec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.0, g.as_raw(), &mut size);
            if ptr.is_null() || size == 0 {
                None
            } else {
                let mut res = Vec::with_capacity(size);
                for i in 0..size {
                    res.push(::std::ptr::read(ptr.add(i)));
                }
                Some(res)
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn geom_from_HEX_buf(&self, hex: &[u8]) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSGeomFromHEX_buf_r(self.0, hex.as_ptr(), hex.len()))
        }
    }

    #[allow(non_snake_case)]
    pub fn geom_to_HEX_buf(&self, g: &GGeom) -> Option<Vec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.0, g.as_raw(), &mut size);
            if ptr.is_null() || size == 0 {
                None
            } else {
                let mut res = Vec::with_capacity(size);
                for i in 0..size {
                    res.push(::std::ptr::read(ptr.add(i)));
                }
                Some(res)
            }
        }
    }
}

impl Drop for GContextHandle {
    fn drop(&mut self) {
        unsafe { GEOS_finish_r(self.0) };
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Primitive)]
#[repr(C)]
pub enum GEOSGeomTypes {
    Point = 0,
    LineString = 1,
    LinearRing = 2,
    Polygon = 3,
    MultiPoint = 4,
    MultiLineString = 5,
    MultiPolygon = 6,
    GeometryCollection = 7,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct GeosError {
    pub desc: &'static str,
}

// We need to cleanup only the char* from geos, the const char* are not to be freed.
// this has to be checked method by method in geos
// so we provide 2 method to wrap a char* to a string, one that manage (and thus free) the underlying char*
// and one that does not free it
unsafe fn unmanaged_string(raw_ptr: *const c_char) -> String {
    let c_str = CStr::from_ptr(raw_ptr);
    str::from_utf8(c_str.to_bytes()).unwrap().to_string()
}

unsafe fn managed_string(raw_ptr: *mut c_char) -> String {
    let s = unmanaged_string(raw_ptr);
    GEOSFree(raw_ptr as *mut c_void);
    s
}

#[allow(dead_code)]
pub fn clip_by_rect(g: &GGeom, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> GeosResult<GGeom> {
    unsafe {
        GGeom::new_from_raw(GEOSClipByRect(
            g.as_raw(),
            xmin as c_double,
            ymin as c_double,
            xmax as c_double,
            ymax as c_double,
        ))
    }
}

pub fn version() -> String {
    unsafe { unmanaged_string(GEOSversion()) }
}

fn initialize() {
    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| unsafe {
        initGEOS();
        assert_eq!(atexit(cleanup), 0);
    });

    extern "C" fn cleanup() {
        unsafe {
            finishGEOS();
        }
    }
}

pub struct CoordSeq(NonNull<GEOSCoordSequence>);

impl Drop for CoordSeq {
    fn drop(&mut self) {
        unsafe { GEOSCoordSeq_destroy(self.0.as_mut()) };
    }
}

impl Clone for CoordSeq {
    fn clone(&self) -> CoordSeq {
        CoordSeq(NonNull::new(unsafe { GEOSCoordSeq_clone(self.0.as_ref()) }).unwrap())
    }
}

impl CoordSeq {
    pub fn new(size: u32, dims: u32) -> CoordSeq {
        initialize();
        CoordSeq(NonNull::new(unsafe { GEOSCoordSeq_create(size as c_uint, dims as c_uint) }).unwrap())
    }

    unsafe fn new_from_raw(c_obj: *mut GEOSCoordSequence) -> GeosResult<CoordSeq> {
        NonNull::new(c_obj)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(CoordSeq)
    }
    pub fn set_x(&mut self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setX(
                self.0.as_mut(),
                idx as c_uint,
                val as c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set x for coord".into()))
        } else {
            Ok(())
        }
    }
    pub fn set_y(&mut self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setY(
                self.0.as_mut(),
                idx as c_uint,
                val as c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set y for coord".into()))
        } else {
            Ok(())
        }
    }
    pub fn set_z(&mut self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setZ(
                self.0.as_mut(),
                idx as c_uint,
                val as c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set z for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn get_x(&self, idx: u32) -> GeosResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe {
            GEOSCoordSeq_getX(
                self.0.as_ref(),
                idx as c_uint,
                &mut n,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_y(&self, idx: u32) -> GeosResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe {
            GEOSCoordSeq_getY(
                self.0.as_ref(),
                idx as c_uint,
                &mut n,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_z(&self, idx: u32) -> GeosResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ(
                self.0.as_ref(),
                idx as c_uint,
                &mut n,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn len(&self) -> GeosResult<usize> {
        let mut n = 0 as c_uint;
        let ret_val = unsafe {
            GEOSCoordSeq_getSize(
                self.0.as_ref(),
                &mut n,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(n as usize)
        }
    }
}

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
    pub fn new(wkt: &str) -> GeosResult<GGeom> {
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

    unsafe fn new_from_raw(g: *mut GEOSGeometry) -> GeosResult<GGeom> {
        NonNull::new(g)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(GGeom)
    }

    fn as_raw(&self) -> &GEOSGeometry {
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
        let type_geom = self.geometry_type()?;
        match type_geom {
            GEOSGeomTypes::Point | GEOSGeomTypes::LineString | GEOSGeomTypes::LinearRing => unsafe {
                let t = GEOSCoordSeq_clone(GEOSGeom_getCoordSeq(self.as_raw()));
                CoordSeq::new_from_raw(t)
            }
            _ => Err(Error::ImpossibleOperation("Geometry must be a Point, LineString or LinearRing to extract it's coordinates".into())),
        }
    }

    pub fn geometry_type(&self) -> GeosResult<GEOSGeomTypes> {
        let type_geom = unsafe { GEOSGeomTypeId(self.as_raw()) as i32 };

        GEOSGeomTypes::from_i32(type_geom).ok_or(Error::GeosError(format!("impossible to get geometry type (val={})", type_geom)))
    }

    pub fn area(&self) -> GeosResult<f64> {
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

    pub fn is_ring(&self) -> GeosResult<bool> {
        let rv = unsafe { GEOSisRing(self.as_raw()) };
        check_geos_predicate(rv, PredicateType::IsRing)
    }

    pub fn intersects(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSIntersects(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Intersects)
    }

    pub fn crosses(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSCrosses(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Crosses)
    }

    pub fn disjoint(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSDisjoint(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Disjoint)
    }

    pub fn touches(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSTouches(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Touches)
    }

    pub fn overlaps(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSOverlaps(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Overlaps)
    }

    pub fn within(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSWithin(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Within)
    }

    pub fn equals(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSEquals(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Equals)
    }

    pub fn equals_exact(&self, g2: &GGeom, precision: f64) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact(
                self.as_raw(),
                g2.as_raw(),
                precision as c_double,
            )
        };
        check_geos_predicate(ret_val, PredicateType::EqualsExact)
    }

    pub fn covers(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSCovers(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Covers)
    }

    pub fn covered_by(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSCoveredBy(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::CoveredBy)
    }

    pub fn contains(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val =
            unsafe { GEOSContains(self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSBuffer(
                self.as_raw(),
                width as c_double,
                quadsegs as c_int,
            ))
        }
    }

    pub fn is_empty(&self) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSisEmpty(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSisSimple(self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::IsSimple)
    }

    pub fn difference(&self, g2: &GGeom) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSDifference(self.as_raw(), g2.as_raw()))
        }
    }

    pub fn envelope(&self) -> GeosResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSEnvelope(self.as_raw())) }
    }

    pub fn sym_difference(&self, g2: &GGeom) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSSymDifference(self.as_raw(), g2.as_raw()))
        }
    }

    pub fn union(&self, g2: &GGeom) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSUnion(self.as_raw(), g2.as_raw()))
        }
    }

    pub fn get_centroid(&self) -> GeosResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGetCentroid(self.as_raw())) }
    }

    pub fn create_polygon(mut exterior: GGeom, mut interiors: Vec<GGeom>) -> GeosResult<GGeom> {
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

    pub fn create_geometrycollection(geoms: Vec<GGeom>) -> GeosResult<GGeom> {
        create_multi_geom(geoms, GEOSGeomTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<GGeom>) -> GeosResult<GGeom> {
        if !check_same_geometry_type(&polygons, GEOSGeomTypes::Polygon) {
            return Err(
                Error::ImpossibleOperation(
                    "all the provided geometry have to be of type Polygon".to_string()));
        }
        create_multi_geom(polygons, GEOSGeomTypes::MultiPolygon)
    }

    pub fn create_multilinestring(linestrings: Vec<GGeom>) -> GeosResult<GGeom> {
        if !check_same_geometry_type(&linestrings, GEOSGeomTypes::LineString) {
            return Err(
                Error::ImpossibleOperation(
                    "all the provided geometry have to be of type LineString".to_string()));
        }
        create_multi_geom(linestrings, GEOSGeomTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<GGeom>) -> GeosResult<GGeom> {
        if !check_same_geometry_type(&points, GEOSGeomTypes::Point) {
            return Err(
                Error::ImpossibleOperation(
                    "all the provided geometry have to be of type Point".to_string()));
        }
        create_multi_geom(points, GEOSGeomTypes::MultiPoint)
    }

    pub fn create_point(s: CoordSeq) -> GeosResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGeom_createPoint(GEOSCoordSeq_clone(s.0.as_ref()))) }
    }

    pub fn create_line_string(s: CoordSeq) -> GeosResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLineString(s.0.as_ref()))
        }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn create_linear_ring(s: CoordSeq) -> GeosResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLinearRing(s.0.as_ref()))
        }?;
        mem::forget(s);
        Ok(obj)
    }

    pub fn unary_union(&self) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSUnaryUnion(self.as_raw()))
        }
    }

    pub fn voronoi(
        &self,
        envelope: Option<&GGeom>,
        tolerance: f64,
        only_edges: bool,
    ) -> GeosResult<GGeom> {
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
    pub fn normalize(&mut self) -> GeosResult<bool> {
        let ret_val = unsafe { GEOSNormalize(self.0.as_ptr()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }
}

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
    pub fn contains(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContains(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedContains)
    }
    pub fn contains_properly(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContainsProperly(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedContainsProperly)
    }
    pub fn covered_by(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCoveredBy(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCoveredBy)
    }
    pub fn covers(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCovers(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCovers)
    }
    pub fn crosses(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCrosses(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCrosses)
    }
    pub fn disjoint(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedDisjoint(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedDisjoint)
    }
    pub fn intersects(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedIntersects(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedIntersects)
    }
    pub fn overlaps(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedOverlaps(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedOverlaps)
    }
    pub fn touches(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedTouches(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedTouches)
    }
    pub fn within(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedWithin(
                self.0.as_ref(),
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedWithin)
    }
}

fn check_geos_predicate(val: i32, p: PredicateType) -> GeosResult<bool> {
    match val {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::GeosFunctionError(p, val))
    }
}

fn check_same_geometry_type(geoms: &[GGeom], geom_type: GEOSGeomTypes) -> bool {
    geoms.iter()
        .all(|g| g.geometry_type().map(|t| t == geom_type).unwrap_or(false))
}

fn create_multi_geom(mut geoms: Vec<GGeom>, output_type: GEOSGeomTypes) -> GeosResult<GGeom> {
    let nb_geoms = geoms.len();
    let res = unsafe {
        GGeom::new_from_raw(GEOSGeom_createCollection(
            output_type as c_int,
            geoms.as_mut_ptr() as *mut *mut GEOSGeometry,
            nb_geoms as c_uint,
        ))
    }?;

    // we'll transfert the ownership of the ptr to the new GGeom,
    // so the old one needs to forget their c ptr to avoid double cleanup
    for g in geoms {
        mem::forget(g);
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::{check_geos_predicate};
    use error::PredicateType;

    #[test]
    fn check_geos_predicate_ok_test() {
        assert_eq!(check_geos_predicate(0, PredicateType::Intersects).unwrap(), false);
    }

    #[test]
    fn check_geos_predicate_ko_test() {
        assert_eq!(check_geos_predicate(1, PredicateType::Intersects).unwrap(), true);
    }

    #[test]
    fn check_geos_predicate_err_test() {
        let r = check_geos_predicate(42, PredicateType::Intersects);
        let e = r.err().unwrap();

        assert_eq!(format!("{}", e), "error while calling libgeos method Intersects (error number = 42)".to_string());
    }
}
