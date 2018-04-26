use libc::{atexit, c_char, c_double, c_int, c_uint, c_void, size_t};
use std::sync::{Once, ONCE_INIT};
use std::ffi::{CStr, CString};
use std::{ptr, str, mem};
use error::{Error, Result as GeosResult, PredicateType};
use std;
use num_traits::FromPrimitive;

#[repr(C)]
pub struct GEOSWKTReader { private: [u8; 0]}
#[repr(C)]
pub struct GEOSWKTWriter { private: [u8; 0]}
#[repr(C)]
pub struct GEOSPreparedGeometry { private: [u8; 0]}
#[repr(C)]
pub struct GEOSCoordSequence { private: [u8; 0]}
#[repr(C)]
pub struct GEOSGeometry { private: [u8; 0]}

#[link(name = "geos_c")]
extern "C" {
    fn initGEOS() -> *mut c_void;
    fn GEOSversion() -> *const c_char;
    fn finishGEOS() -> *mut c_void;

    // API for reading WKT :
    fn GEOSWKTReader_create() -> *mut GEOSWKTReader;
    fn GEOSWKTReader_destroy(reader: *mut GEOSWKTReader);
    fn GEOSWKTReader_read(reader: *mut GEOSWKTReader, wkt: *const c_char) -> *mut GEOSGeometry;

    // API for writing WKT :
    fn GEOSWKTWriter_create() -> *mut GEOSWKTWriter;
    fn GEOSWKTWriter_destroy(writer: *mut GEOSWKTWriter);
    fn GEOSWKTWriter_write(writer: *mut GEOSWKTWriter, g: *const GEOSGeometry) -> *const c_char;
    fn GEOSWKTWriter_setRoundingPrecision(writer: *mut GEOSWKTWriter, precision: c_int);

    fn GEOSFree(buffer: *mut c_void);

    fn GEOSPrepare(g: *const GEOSGeometry) -> *mut GEOSPreparedGeometry;
    fn GEOSGeom_destroy(g: *mut GEOSGeometry);
    pub fn GEOSGeom_clone(g: *const GEOSGeometry) -> *mut GEOSGeometry;

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

    // Geometry must be a LineString, LinearRing or Point :
    fn GEOSGeom_getCoordSeq(g: *const GEOSGeometry) -> *mut GEOSCoordSequence;

    // Geometry constructor :
    pub fn GEOSGeom_createPoint(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLineString(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLinearRing(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
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

    // Functions acting on GEOSGeometry :
    fn GEOSisEmpty(g: *const GEOSGeometry) -> c_int;
    fn GEOSisSimple(g: *const GEOSGeometry) -> c_int;
    fn GEOSisRing(g: *const GEOSGeometry) -> c_int;
    #[allow(dead_code)]
    fn GEOSHasZ(g: *const GEOSGeometry) -> c_int;
    #[allow(dead_code)]
    fn GEOSisClosed(g: *const GEOSGeometry) -> c_int;
    fn GEOSisValid(g: *const GEOSGeometry) -> c_int;

    fn GEOSGeomToWKT(g: *const GEOSGeometry) -> *const c_char;
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

    // Functions acting on GEOSPreparedGeometry :
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
}

#[derive(Eq, PartialEq, Debug, Primitive)]
#[repr(C)]
#[allow(dead_code)]
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GeosError {
    pub desc: &'static str,
}

// We need to cleanup only the char* from geos, the const char* are not to be freed.
// this has to be checked method by method in geos
// so we provide 2 method to wrap a char* to a string, one that manage (and thus free) the underlying char*
// and one that does not free it
fn unmanaged_string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    let s = str::from_utf8(c_str.to_bytes()).unwrap().to_string();
    s
}

fn managed_string(raw_ptr: *const c_char) -> String {
    let s = unmanaged_string(raw_ptr);
    unsafe { GEOSFree(raw_ptr as *mut c_void) };
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

pub struct CoordSeq(*mut GEOSCoordSequence);

impl Drop for CoordSeq {
    fn drop(&mut self) {
        unsafe { GEOSCoordSeq_destroy(self.0 as *mut GEOSCoordSequence) };
        self.0 = ptr::null_mut();
    }
}

impl Clone for CoordSeq {
    fn clone(&self) -> CoordSeq {
        CoordSeq(unsafe { GEOSCoordSeq_clone(self.0 as *const GEOSCoordSequence) })
    }
}

impl CoordSeq {
    pub fn new(size: u32, dims: u32) -> CoordSeq {
        initialize();
        CoordSeq(unsafe { GEOSCoordSeq_create(size as c_uint, dims as c_uint) })
    }

    unsafe fn new_from_raw(c_obj: *mut GEOSCoordSequence) -> GeosResult<CoordSeq> {
        if c_obj.is_null() {
            Err(Error::NoConstructionFromNullPtr)
        } else {
            Ok(CoordSeq(c_obj))
        }
    }
    pub fn set_x(&self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setX(
                self.0 as *mut GEOSCoordSequence,
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
    pub fn set_y(&self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setY(
                self.0 as *mut GEOSCoordSequence,
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
    pub fn set_z(&self, idx: u32, val: f64) -> GeosResult<()> {
        let ret_val = unsafe {
            GEOSCoordSeq_setZ(
                self.0 as *mut GEOSCoordSequence,
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
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe {
            GEOSCoordSeq_getX(
                self.0 as *const GEOSCoordSequence,
                idx as c_uint,
                n_mut_ref as *mut c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(*n_mut_ref)
        }
    }

    pub fn get_y(&self, idx: u32) -> GeosResult<f64> {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe {
            GEOSCoordSeq_getY(
                self.0 as *const GEOSCoordSequence,
                idx as c_uint,
                n_mut_ref as *mut c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(*n_mut_ref)
        }
    }

    pub fn get_z(&self, idx: u32) -> GeosResult<f64> {
        let n_mut_ref = &mut 0.0;
        let ret_val = unsafe {
            GEOSCoordSeq_getZ(
                self.0 as *const GEOSCoordSequence,
                idx as c_uint,
                n_mut_ref as *mut c_double,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(*n_mut_ref)
        }
    }

    pub fn len(&self) -> GeosResult<usize> {
        let n_mut_ref = &mut 0u32;
        let ret_val = unsafe {
            GEOSCoordSeq_getSize(
                self.0 as *const GEOSCoordSequence,
                n_mut_ref as *mut c_uint,
            )
        };
        if ret_val == 0 {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(*n_mut_ref as usize)
        }
    }

    /// Release the underlying C ptr (so the ptr is not destroyed when the object is destroyed)
    /// The C ptr needs to be cleanup afterward!
    /// This method can be thus called only if the C ptr is given to the C API
    pub fn release(mut self) -> *mut GEOSCoordSequence {
        mem::replace(&mut self.0, std::ptr::null_mut())
    }
}

#[repr(C)]
pub struct GGeom(*mut GEOSGeometry);

impl Drop for GGeom {
    fn drop(&mut self) {
        unsafe { GEOSGeom_destroy(self.0) }
    }
}

impl Clone for GGeom {
    fn clone(&self) -> GGeom {
        GGeom(unsafe { GEOSGeom_clone(self.0) })
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
        if g.is_null() {
            return Err(Error::NoConstructionFromNullPtr);
        }
        Ok(GGeom(g))
    }

    fn as_raw(&self) -> &GEOSGeometry {
        unsafe { &*self.0 }
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
                CoordSeq::new_from_raw(t as *mut GEOSCoordSequence)
            }
            _ => Err(Error::ImpossibleOperation("Geometry must be a Point, LineString or LinearRing to extract it's coordinates".into())),
        }
    }

    pub fn geometry_type(&self) -> GeosResult<GEOSGeomTypes> {
        let type_geom = unsafe { GEOSGeomTypeId(self.as_raw() ) as i32 };

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

    pub fn to_wkt_precison(&self, precision: Option<u32>) -> String {
        let writer = unsafe { GEOSWKTWriter_create() };
        if let Some(x) = precision {
            unsafe { GEOSWKTWriter_setRoundingPrecision(writer, x as c_int) }
        };
        let c_result = unsafe { GEOSWKTWriter_write(writer, self.as_raw()) };
        let result = managed_string(c_result);
        unsafe { GEOSWKTWriter_destroy(writer) };
        result
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

    pub fn get_centroid(&self) -> GeosResult<GGeom> {
        unsafe { GGeom::new_from_raw(GEOSGetCentroid(self.as_raw())) }
    }

    pub fn create_polygon(exterior: GGeom, mut interiors: Vec<GGeom>) -> GeosResult<GGeom> {
        let nb_interiors = interiors.len();
        let res = unsafe {
            GGeom::new_from_raw(GEOSGeom_createPolygon(
                &mut *exterior.0,
                interiors[..].as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_interiors as c_uint,
            ))
        }?;

        // we'll transfert the ownership of the ptr to the new GGeom,
        // so the old one needs to forget their c ptr to avoid double cleanup
        mem::forget(exterior);
        for mut i in interiors {
            mem::forget(i);
        }

        Ok(res)
    }

    pub fn create_multipolygon(mut polygons: Vec<GGeom>) -> GeosResult<GGeom> {
        let nb_polygons = polygons.len();
        let res = unsafe {
            GGeom::new_from_raw(GEOSGeom_createCollection(
                GEOSGeomTypes::MultiPolygon as c_int,
                polygons[..].as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_polygons as c_uint,
            ))
        }?;

        // we'll transfert the ownership of the ptr to the new GGeom,
        // so the old one needs to forget their c ptr to avoid double cleanup
        for mut p in polygons {
            mem::forget(p);
        }

        Ok(res)
    }

    pub fn create_point(s: &CoordSeq) -> GeosResult<GGeom> {
        unsafe {
            GGeom::new_from_raw(GEOSGeom_createPoint(GEOSCoordSeq_clone(s.0 as *const GEOSCoordSequence)))
        }
    }

    pub fn create_line_string(s: CoordSeq) -> GeosResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLineString(s.0 as *const GEOSCoordSequence))
        }?;
        s.release();
        Ok(obj)
    }

    pub fn create_linear_ring(s: CoordSeq) -> GeosResult<GGeom> {
        let obj = unsafe {
            GGeom::new_from_raw(GEOSGeom_createLinearRing(s.0 as *const GEOSCoordSequence))
        }?;
        s.release();
        Ok(obj)
    }
}

pub struct PreparedGGeom(*mut GEOSPreparedGeometry);

impl Clone for PreparedGGeom {
    fn clone(&self) -> PreparedGGeom {
        PreparedGGeom(self.0)
    }
}

impl Drop for PreparedGGeom {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        unsafe { GEOSPreparedGeom_destroy(self.0) };
        self.0 = ptr::null_mut();
    }
}

impl PreparedGGeom {
    pub fn new(g: &GGeom) -> PreparedGGeom {
        PreparedGGeom(unsafe { GEOSPrepare(g.as_raw()) })
    }
    pub fn contains(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContains(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedContains)
    }
    pub fn contains_properly(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedContainsProperly(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedContainsProperly)
    }
    pub fn covered_by(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCoveredBy(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCoveredBy)
    }
    pub fn covers(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCovers(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCovers)
    }
    pub fn crosses(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedCrosses(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedCrosses)
    }
    pub fn disjoint(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedDisjoint(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedDisjoint)
    }
    pub fn intersects(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedIntersects(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedIntersects)
    }
    pub fn overlaps(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedOverlaps(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedOverlaps)
    }
    pub fn touches(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedTouches(
                self.0 as *const GEOSPreparedGeometry,
                g2.as_raw(),
            )
        };
        check_geos_predicate(ret_val, PredicateType::PreparedTouches)
    }
    pub fn within(&self, g2: &GGeom) -> GeosResult<bool> {
        let ret_val = unsafe {
            GEOSPreparedWithin(
                self.0 as *const GEOSPreparedGeometry,
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
