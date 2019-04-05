use libc::{c_char, c_double, c_int, c_uchar, c_uint, c_void, size_t};

#[repr(C)]
pub struct GEOSWKTReader {
    private: [u8; 0],
}
#[repr(C)]
pub struct GEOSWKTWriter {
    private: [u8; 0],
}
#[repr(C)]
pub struct GEOSPreparedGeometry {
    private: [u8; 0],
}
#[repr(C)]
pub struct GEOSCoordSequence {
    private: [u8; 0],
}
#[repr(C)]
pub struct GEOSGeometry {
    private: [u8; 0],
}
#[repr(C)]
pub struct GEOSContextHandle_HS {
    private: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type GEOSContextHandle_t = *mut GEOSContextHandle_HS;
#[allow(non_camel_case_types)]
pub type GEOSMessageHandler_r =
    Option<unsafe extern "C" fn(message: *const c_char, userdata: *mut c_void)>;

#[link(name = "geos_c")]
extern "C" {
    pub fn initGEOS() -> *mut c_void;
    pub fn GEOSversion() -> *const c_char;
    pub fn finishGEOS() -> *mut c_void;

    // API for reading WKT:
    pub fn GEOSWKTReader_create() -> *mut GEOSWKTReader;
    pub fn GEOSWKTReader_destroy(reader: *mut GEOSWKTReader);
    pub fn GEOSWKTReader_read(reader: *mut GEOSWKTReader, wkt: *const c_char) -> *mut GEOSGeometry;

    // API for writing WKT:
    pub fn GEOSWKTWriter_create() -> *mut GEOSWKTWriter;
    pub fn GEOSWKTWriter_destroy(writer: *mut GEOSWKTWriter);
    pub fn GEOSWKTWriter_write(writer: *mut GEOSWKTWriter, g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSWKTWriter_setRoundingPrecision(writer: *mut GEOSWKTWriter, precision: c_int);
    pub fn GEOSWKTWriter_create_r(handle: GEOSContextHandle_t) -> *mut GEOSWKTWriter;
    pub fn GEOSWKTWriter_destroy_r(handle: GEOSContextHandle_t, writer: *mut GEOSWKTWriter);
    pub fn GEOSWKTWriter_write_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
        g: *const GEOSGeometry,
    ) -> *mut c_char;
    pub fn GEOSWKTWriter_setRoundingPrecision_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
        precision: c_int,
    );

    pub fn GEOSFree(buffer: *mut c_void);

    pub fn GEOSPrepare(g: *const GEOSGeometry) -> *mut GEOSPreparedGeometry;
    pub fn GEOSGeom_destroy(g: *mut GEOSGeometry);
    pub fn GEOSGeom_clone(g: *const GEOSGeometry) -> *mut GEOSGeometry;

    pub fn GEOSCoordSeq_create(size: c_uint, dims: c_uint) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_destroy(s: *mut GEOSCoordSequence);
    pub fn GEOSCoordSeq_clone(s: *const GEOSCoordSequence) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_setX(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_setY(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_setZ(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_getX(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double)
        -> c_int;
    pub fn GEOSCoordSeq_getY(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double)
        -> c_int;
    pub fn GEOSCoordSeq_getZ(s: *const GEOSCoordSequence, idx: c_uint, val: *mut c_double)
        -> c_int;
    pub fn GEOSCoordSeq_getSize(s: *const GEOSCoordSequence, val: *mut c_uint) -> c_int;
    pub fn GEOSCoordSeq_getDimensions(s: *const GEOSCoordSequence, val: *mut c_uint) -> c_int;

    // Geometry must be a LineString, LinearRing or Point:
    pub fn GEOSGeom_getCoordSeq(g: *const GEOSGeometry) -> *mut GEOSCoordSequence;

    // Geometry constructor:
    pub fn GEOSGeom_createPoint(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLineString(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLinearRing(s: *const GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createPolygon(
        shell: *mut GEOSGeometry,
        holes: *mut *mut GEOSGeometry,
        nholes: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createCollection(
        t: c_int,
        geoms: *mut *mut GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;

    // Functions acting on GEOSGeometry:
    pub fn GEOSisEmpty(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisSimple(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisRing(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSHasZ(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisClosed(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisValid(g: *const GEOSGeometry) -> c_char;

    pub fn GEOSGeomToWKT(g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSGeomFromWKB_buf(wkb: *const u8, size: size_t) -> *mut GEOSGeometry;
    pub fn GEOSGeomToWKB_buf(g: *const GEOSGeometry, size: *mut size_t) -> *mut u8;
    pub fn GEOSGeomTypeId(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSArea(g: *const GEOSGeometry, area: *mut c_double) -> c_int;
    pub fn GEOSLength(g: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSDistance(g1: *const GEOSGeometry, g2: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSDistanceIndexed(g1: *const GEOSGeometry, g2: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSHausdorffDistance(g1: *const GEOSGeometry, g2: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSHausdorffDistanceDensify(g1: *const GEOSGeometry, g2: *const GEOSGeometry, density_frac: c_double, distance: *mut c_double) -> c_int;
    pub fn GEOSFrechetDistance(g1: *const GEOSGeometry, g2: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSFrechetDistanceDensify(g1: *const GEOSGeometry, g2: *const GEOSGeometry, density_frace: c_double, distance: *mut c_double) -> c_int;
    pub fn GEOSGeomGetLength(g: *const GEOSGeometry, length: *mut c_double) -> c_int;
    pub fn GEOSNearestPoints(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSCoordSequence;
    pub fn GEOSDisjoint(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSTouches(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSIntersects(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSCrosses(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSWithin(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSContains(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSOverlaps(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSEquals(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSEqualsExact(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        tolerance: c_double,
    ) -> c_char;
    pub fn GEOSCovers(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSCoveredBy(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> c_char;

    pub fn GEOSBuffer(
        g: *const GEOSGeometry,
        width: c_double,
        quadsegs: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSEnvelope(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSIntersection(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSConvexHull(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSBoundary(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSGetCentroid(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSSymDifference(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSDifference(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSUnion(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSUnaryUnion(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSClipByRect(
        g: *const GEOSGeometry,
        xmin: c_double,
        ymin: c_double,
        xmax: c_double,
        ymax: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSSnap(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        tolerance: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_extractUniquePoints(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSVoronoiDiagram(
        g: *const GEOSGeometry,
        env: *const GEOSGeometry,
        tolerance: c_double,
        onlyEdges: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSNormalize(g: *mut GEOSGeometry) -> c_int;

    // Functions acting on GEOSPreparedGeometry:
    pub fn GEOSPreparedContains(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry)
        -> c_int;
    pub fn GEOSPreparedContainsProperly(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedCoveredBy(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedCovers(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    pub fn GEOSPreparedCrosses(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    pub fn GEOSPreparedDisjoint(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry)
        -> c_int;
    pub fn GEOSPreparedIntersects(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedOverlaps(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry)
        -> c_int;
    pub fn GEOSPreparedTouches(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    pub fn GEOSPreparedWithin(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_int;
    pub fn GEOSPreparedGeom_destroy(g: *mut GEOSPreparedGeometry);

    pub fn GEOS_init_r() -> GEOSContextHandle_t;
    pub fn GEOS_finish_r(handle: GEOSContextHandle_t);
    pub fn GEOSContext_setNoticeMessageHandler_r(
        handle: GEOSContextHandle_t,
        nf: GEOSMessageHandler_r,
        userdata: *mut c_void,
    ) -> GEOSMessageHandler_r;
    pub fn GEOSContext_setErrorMessageHandler_r(
        handle: GEOSContextHandle_t,
        nf: GEOSMessageHandler_r,
        userdata: *mut c_void,
    ) -> GEOSMessageHandler_r;
    pub fn GEOS_getWKBOutputDims_r(handle: GEOSContextHandle_t) -> c_int;
    pub fn GEOS_setWKBOutputDims_r(handle: GEOSContextHandle_t, newDims: c_int) -> c_int;
    pub fn GEOS_getWKBByteOrder_r(handle: GEOSContextHandle_t) -> c_int;
    pub fn GEOS_setWKBByteOrder_r(handle: GEOSContextHandle_t, byteOrder: c_int) -> c_int;
    pub fn GEOSGeomFromWKB_buf_r(
        handle: GEOSContextHandle_t,
        wkb: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeomToWKB_buf_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSGeomFromHEX_buf_r(
        handle: GEOSContextHandle_t,
        hex: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeomToHEX_buf_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSisValid_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSGeomTypeId_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSArea_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry, area: *mut c_double) -> c_int;
    pub fn GEOSGeomToWKT_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSisEmpty_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisSimple_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisRing_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSHasZ_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisClosed_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_char;
    pub fn GEOSIntersects_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSDisjoint_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSTouches_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSCrosses_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSWithin_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSContains_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSOverlaps_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSEquals_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSEqualsExact_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        tolerance: c_double,
    ) -> c_char;
    pub fn GEOSCovers_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSCoveredBy_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSDifference_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSEnvelope_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGetCentroid_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSUnion_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSSymDifference_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createPoint_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLineString_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLinearRing_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
    ) -> *mut GEOSGeometry;
    pub fn GEOSUnaryUnion_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSVoronoiDiagram_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        env: *const GEOSGeometry,
        tolerance: c_double,
        onlyEdges: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSNormalize_r(
        handle: GEOSContextHandle_t,
        g: *mut GEOSGeometry,
    ) -> c_int;
    pub fn GEOSIntersection_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSConvexHull_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSBoundary_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;

    pub fn GEOSLength_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSDistance_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSDistanceIndexed_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSHausdorffDistance_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSHausdorffDistanceDensify_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        density_frac: c_double,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSFrechetDistance_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSFrechetDistanceDensify_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        density_frace: c_double,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSGeomGetLength_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        length: *mut c_double,
    ) -> c_int;
    pub fn GEOSSnap_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        tolerance: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_extractUniquePoints_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSNearestPoints_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSCoordSequence;
    pub fn GEOSBuffer_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        width: c_double,
        quadsegs: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createPolygon_r(
        handle: GEOSContextHandle_t,
        shell: *mut GEOSGeometry,
        holes: *mut *mut GEOSGeometry,
        nholes: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createCollection_r(
        handle: GEOSContextHandle_t,
        t: c_int,
        geoms: *mut *mut GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKTReader_create_r(handle: GEOSContextHandle_t) -> *mut GEOSWKTReader;
    pub fn GEOSWKTReader_destroy_r(handle: GEOSContextHandle_t, reader: *mut GEOSWKTReader);
    pub fn GEOSWKTReader_read_r(
        handle: GEOSContextHandle_t,
        reader: *mut GEOSWKTReader,
        wkt: *const c_char,
    ) -> *mut GEOSGeometry;
    pub fn GEOSClipByRect_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        xmin: c_double,
        ymin: c_double,
        xmax: c_double,
        ymax: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_clone_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_destroy_r(handle: GEOSContextHandle_t, g: *mut GEOSGeometry);
    pub fn GEOSCoordSeq_create_r(
        handle: GEOSContextHandle_t,
        size: c_uint,
        dims: c_uint,
    ) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_destroy_r(handle: GEOSContextHandle_t, s: *mut GEOSCoordSequence);
    pub fn GEOSCoordSeq_clone_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
    ) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_setX_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
        idx: c_uint,
        val: c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_setY_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
        idx: c_uint,
        val: c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_setZ_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
        idx: c_uint,
        val: c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getX_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getY_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getZ_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getSize_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        val: *mut c_uint,
    ) -> c_int;
    pub fn GEOSGeom_getCoordSeq_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_getDimensions_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        dims: *mut c_uint,
    ) -> c_int;
    pub fn GEOSPrepare_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSPreparedGeometry;
    pub fn GEOSPreparedContains_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedContainsProperly_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedCoveredBy_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedCovers_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedCrosses_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedDisjoint_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedIntersects_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedOverlaps_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedTouches_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedWithin_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSPreparedGeom_destroy_r(
        handle: GEOSContextHandle_t,
        g: *mut GEOSPreparedGeometry,
    );

    pub fn GEOSOrientationIndex(ax: c_double, ay: c_double, bx: c_double, by: c_double, px: c_double, py: c_double) -> c_int;
}
