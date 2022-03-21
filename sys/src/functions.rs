use libc::{c_char, c_double, c_int, c_uchar, c_uint, c_void, size_t};
use crate::types::*;

extern "C" {
    pub fn initGEOS(nf: GEOSMessageHandler, ef: GEOSMessageHandler);
    pub fn GEOSversion() -> *const c_char;
    pub fn finishGEOS();

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
    pub fn GEOSWKTWriter_getOutputDimension(
        writer: *mut GEOSWKTWriter,
    ) -> c_int;
    pub fn GEOSWKTWriter_setOutputDimension(
        writer: *mut GEOSWKTWriter,
        dim: c_int,
    );
    pub fn GEOSWKTWriter_setTrim(
        writer: *mut GEOSWKTWriter,
        trim: c_char,
    );
    pub fn GEOSWKTWriter_setOld3D(
        writer: *mut GEOSWKTWriter,
        useOld3D: c_int,
    );

    pub fn GEOSFree(buffer: *mut c_void);
    pub fn GEOSFree_r(context: GEOSContextHandle_t, buffer: *mut c_void);

    pub fn GEOSPrepare(g: *const GEOSGeometry) -> *const GEOSPreparedGeometry;
    pub fn GEOSGeom_destroy(g: *mut GEOSGeometry);
    pub fn GEOSGeom_clone(g: *const GEOSGeometry) -> *mut GEOSGeometry;

    pub fn GEOSCoordSeq_create(size: c_uint, dims: c_uint) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_destroy(s: *mut GEOSCoordSequence);
    pub fn GEOSCoordSeq_clone(s: *const GEOSCoordSequence) -> *mut GEOSCoordSequence;
    pub fn GEOSCoordSeq_setX(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_setY(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_setZ(s: *mut GEOSCoordSequence, idx: c_uint, val: c_double) -> c_int;
    pub fn GEOSCoordSeq_setOrdinate(
        s: *mut GEOSCoordSequence,
        idx: c_uint,
        dim: c_uint,
        val: c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getX(
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getY(
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getZ(
        s: *const GEOSCoordSequence,
        idx: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getOrdinate(
        s: *const GEOSCoordSequence,
        idx: c_uint,
        dim: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getSize(s: *const GEOSCoordSequence, val: *mut c_uint) -> c_int;
    pub fn GEOSCoordSeq_getDimensions(s: *const GEOSCoordSequence, val: *mut c_uint) -> c_int;

    // Geometry must be a LineString, LinearRing or Point:
    pub fn GEOSGeom_getCoordSeq(g: *const GEOSGeometry) -> *const GEOSCoordSequence;

    // Geometry constructor:
    pub fn GEOSGeom_createPoint(s: *mut GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLineString(s: *mut GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLinearRing(s: *mut GEOSCoordSequence) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createPolygon(
        shell: *mut GEOSGeometry,
        holes: *mut *mut GEOSGeometry,
        nholes: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyPolygon() -> *mut GEOSGeometry;
    pub fn GEOSGeom_createCollection(
        t: c_int,
        geoms: *mut *mut GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyCollection(type_: c_int) -> *mut GEOSGeometry;

    // Functions acting on GEOSGeometry:
    pub fn GEOSisEmpty(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisSimple(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisRing(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSHasZ(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisClosed(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisValid(g: *const GEOSGeometry) -> c_char;
    pub fn GEOSisValidReason(g: *const GEOSGeometry) -> *mut c_char;

    pub fn GEOSGeomFromHEX_buf(
        hex: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeomToHEX_buf(
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSGeomFromWKB_buf(wkb: *const c_uchar, size: size_t) -> *mut GEOSGeometry;
    pub fn GEOSGeomToWKB_buf(g: *const GEOSGeometry, size: *mut size_t) -> *mut c_uchar;
    pub fn GEOSGeomTypeId(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSArea(g: *const GEOSGeometry, area: *mut c_double) -> c_int;
    pub fn GEOSLength(g: *const GEOSGeometry, distance: *mut c_double) -> c_int;
    pub fn GEOSDistance(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSDistanceIndexed(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSHausdorffDistance(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSHausdorffDistanceDensify(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        density_frac: c_double,
        distance: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSFrechetDistance(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSFrechetDistanceDensify(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        density_frace: c_double,
        distance: *mut c_double,
    ) -> c_int;
    pub fn GEOSGeomGetLength(g: *const GEOSGeometry, length: *mut c_double) -> c_int;
    pub fn GEOSGeomGetX(g: *const GEOSGeometry, x: *mut c_double) -> c_int;
    pub fn GEOSGeomGetY(g: *const GEOSGeometry, y: *mut c_double) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeomGetZ(g: *const GEOSGeometry, z: *mut c_double) -> c_int;
    pub fn GEOSGeomGetPointN(g: *const GEOSGeometry, n: c_int) -> *mut GEOSGeometry;
    pub fn GEOSGeomGetStartPoint(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSGeomGetEndPoint(g: *const GEOSGeometry) -> *mut GEOSGeometry;
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
    pub fn GEOSBufferParams_create() -> *mut GEOSBufferParams;
    pub fn GEOSBufferParams_destroy(params: *mut GEOSBufferParams);
    pub fn GEOSBufferParams_setEndCapStyle(p: *mut GEOSBufferParams, style: c_int) -> c_int;
    pub fn GEOSBufferParams_setJoinStyle(p: *mut GEOSBufferParams, joinStyle: c_int) -> c_int;
    pub fn GEOSBufferParams_setMitreLimit(p: *mut GEOSBufferParams, mitreLimit: c_double) -> c_int;
    pub fn GEOSBufferParams_setQuadrantSegments(p: *mut GEOSBufferParams, quadSegs: c_int) -> c_int;
    pub fn GEOSBufferParams_setSingleSided(p: *mut GEOSBufferParams, singleSided: c_int) -> c_int;
    pub fn GEOSBufferWithParams(
        g: *const GEOSGeometry,
        p: *const GEOSBufferParams,
        width: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSBufferWithStyle(
        g: *const GEOSGeometry,
        width: c_double,
        quadSegs: c_int,
        endCapStyle: c_int,
        joinStyle: c_int,
        mitreLimit: c_double,
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
    #[cfg(feature = "v3_8_0")]
    pub fn GEOSBuildArea(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSLineMerge(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSReverse(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSSimplify(g: *const GEOSGeometry, tolerance: c_double) -> *mut GEOSGeometry;
    pub fn GEOSTopologyPreserveSimplify(
        g: *const GEOSGeometry,
        tolerance: c_double,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_8_0")]
    pub fn GEOSMakeValid(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSGetNumGeometries(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeomType(g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSGetSRID(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSSetSRID(g: *mut GEOSGeometry, srid: c_int);
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSSegmentIntersection(
        ax0: c_double,
        ay0: c_double,
        ax1: c_double,
        ay1: c_double,
        bx0: c_double,
        by0: c_double,
        bx1: c_double,
        by1: c_double,
        cx: *mut c_double,
        cy: *mut c_double,
    ) -> c_int;
    pub fn GEOSDelaunayTriangulation(
        g: *const GEOSGeometry,
        tolerance: c_double,
        onlyEdges: c_int,
    ) -> *mut GEOSGeometry;

    // Functions acting on GEOSPreparedGeometry:
    pub fn GEOSPreparedContains(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedContainsProperly(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCoveredBy(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCovers(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCrosses(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedDisjoint(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedIntersects(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedOverlaps(
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedTouches(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSPreparedWithin(pg1: *const GEOSPreparedGeometry, g2: *const GEOSGeometry) -> c_char;
    pub fn GEOSPreparedGeom_destroy(g: *const GEOSPreparedGeometry);
    pub fn GEOSGetInteriorRingN(g: *const GEOSGeometry, n: c_int) -> *const GEOSGeometry;
    pub fn GEOSGetExteriorRing(g: *const GEOSGeometry) -> *const GEOSGeometry;
    pub fn GEOSGetNumInteriorRings(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeomGetNumPoints(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGetNumCoordinates(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeom_getDimensions(g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeom_getCoordinateDimension(g: *const GEOSGeometry) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSCoordSeq_isCCW(s: *const GEOSCoordSequence, is_ccw: *mut c_char) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSGeom_getPrecision(g: *const GEOSGeometry) -> c_double;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSGeom_setPrecision(
        g: *const GEOSGeometry,
        grid_size: c_double,
        flags: c_int,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getXMax(g: *const GEOSGeometry, value: *mut c_double) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getXMin(g: *const GEOSGeometry, value: *mut c_double) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getYMax(g: *const GEOSGeometry, value: *mut c_double) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getYMin(g: *const GEOSGeometry, value: *mut c_double) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumClearance(g: *const GEOSGeometry, d: *mut c_double) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumClearanceLine(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumRotatedRectangle(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumWidth(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSOrientationIndex(
        ax: c_double,
        ay: c_double,
        bx: c_double,
        by: c_double,
        px: c_double,
        py: c_double,
    ) -> c_int;
    pub fn GEOSGeom_createEmptyLineString() -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyPoint() -> *mut GEOSGeometry;
    pub fn GEOSGeom_getUserData(g: *const GEOSGeometry) -> *mut c_void;
    pub fn GEOSGeom_setUserData(g: *mut GEOSGeometry, userData: *mut c_void);
    pub fn GEOSSTRtree_create(nodeCapacity: size_t) -> *mut GEOSSTRtree;
    pub fn GEOSSTRtree_insert(
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        item: *mut c_void,
    );
    pub fn GEOSSTRtree_query(
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        callback: GEOSQueryCallback,
        userdata: *mut c_void,
    );
    pub fn GEOSSTRtree_nearest(
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
    ) -> *const GEOSGeometry;
    pub fn GEOSSTRtree_nearest_generic(
        tree: *mut GEOSSTRtree,
        item: *const c_void,
        itemEnvelope: *const GEOSGeometry,
        distancefn: GEOSDistanceCallback,
        userdata: *mut c_void,
    ) -> *const c_void;
    pub fn GEOSSTRtree_iterate(
        tree: *mut GEOSSTRtree,
        callback: GEOSQueryCallback,
        userdata: *mut c_void,
    );
    pub fn GEOSSTRtree_remove(
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        item: *mut c_void,
    ) -> c_char;
    pub fn GEOSSTRtree_destroy(tree: *mut GEOSSTRtree);
    pub fn GEOS_getWKBOutputDims() -> c_int;
    pub fn GEOS_setWKBOutputDims(newDims: c_int) -> c_int;
    pub fn GEOS_getWKBByteOrder() -> c_int;
    pub fn GEOS_setWKBByteOrder(byteOrder: c_int) -> c_int;
    pub fn GEOSGetGeometryN(g: *const GEOSGeometry, n: c_int) -> *const GEOSGeometry;
    pub fn GEOSInterpolate(g: *const GEOSGeometry, d: c_double) -> *mut GEOSGeometry;
    pub fn GEOSInterpolateNormalized(g: *const GEOSGeometry, d: c_double) -> *mut GEOSGeometry;
    pub fn GEOSProjectNormalized(g: *const GEOSGeometry, p: *const GEOSGeometry) -> c_double;
    pub fn GEOSNode(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSOffsetCurve(
        g: *const GEOSGeometry,
        width: c_double,
        quadsegs: c_int,
        joinStyle: c_int,
        mitreLimit: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSPointOnSurface(g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSPolygonize(geoms: *const *const GEOSGeometry, ngeoms: c_uint) -> *mut GEOSGeometry;
    pub fn GEOSPolygonize_full(
        input: *const GEOSGeometry,
        cuts: *mut *mut GEOSGeometry,
        dangles: *mut *mut GEOSGeometry,
        invalidRings: *mut *mut GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSPolygonizer_getCutEdges(
        geoms: *const *const GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSProject(g: *const GEOSGeometry, p: *const GEOSGeometry) -> c_double;
    pub fn GEOSRelatePattern(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        pat: *const c_char,
    ) -> c_char;
    pub fn GEOSRelate(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSRelatePatternMatch(mat: *const c_char, pat: *const c_char) -> c_char;
    pub fn GEOSRelateBoundaryNodeRule(
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        bnr: c_int,
    ) -> *mut c_char;
    pub fn GEOSSharedPaths(g1: *const GEOSGeometry, g2: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSWKBReader_create() -> *mut GEOSWKBReader;
    pub fn GEOSWKBReader_destroy(reader: *mut GEOSWKBReader);
    pub fn GEOSWKBReader_read(
        reader: *mut GEOSWKBReader,
        wkb: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKBReader_readHEX(
        reader: *mut GEOSWKBReader,
        hex: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKBWriter_create() -> *mut GEOSWKBWriter;
    pub fn GEOSWKBWriter_destroy(reader: *mut GEOSWKBWriter);
    pub fn GEOSWKBWriter_write(
        writer: *mut GEOSWKBWriter,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSWKBWriter_writeHEX(
        writer: *mut GEOSWKBWriter,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSWKBWriter_getOutputDimension(writer: *const GEOSWKBWriter) -> c_int;
    pub fn GEOSWKBWriter_setOutputDimension(
        writer: *mut GEOSWKBWriter,
        newDimension: c_int,
    );
    pub fn GEOSWKBWriter_getByteOrder(writer: *const GEOSWKBWriter) -> c_int;
    pub fn GEOSWKBWriter_setByteOrder(
        writer: *mut GEOSWKBWriter,
        byteOrder: c_int,
    );
    pub fn GEOSWKBWriter_getIncludeSRID(writer: *const GEOSWKBWriter) -> c_char;
    pub fn GEOSWKBWriter_setIncludeSRID(
        writer: *mut GEOSWKBWriter,
        writeSRID: c_char,
    );
    pub fn GEOSisValidDetail(
        g: *const GEOSGeometry,
        flags: c_int,
        reason: *mut *mut c_char,
        location: *mut *mut GEOSGeometry,
    ) -> c_char;
    pub fn GEOS_interruptCancel();
    pub fn GEOS_interruptRequest();
    pub fn GEOS_interruptRegisterCallback(
        cb: *mut GEOSInterruptCallback,
    ) -> *mut GEOSInterruptCallback;

    /* Thread safe calls */
    pub fn GEOS_init_r() -> GEOSContextHandle_t;
    pub fn GEOS_finish_r(handle: GEOSContextHandle_t);
    pub fn GEOSContext_setNoticeHandler_r(
        handle: GEOSContextHandle_t,
        nf: GEOSMessageHandler,
    ) -> GEOSMessageHandler;
    pub fn GEOSContext_setNoticeMessageHandler_r(
        handle: GEOSContextHandle_t,
        nf: GEOSMessageHandler_r,
        userdata: *mut c_void,
    ) -> GEOSMessageHandler_r;
    pub fn GEOSContext_setErrorHandler_r(
        handle: GEOSContextHandle_t,
        nf: GEOSMessageHandler,
    ) -> GEOSMessageHandler;
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
    pub fn GEOSisValidReason_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSGeomTypeId_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSArea_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        area: *mut c_double,
    ) -> c_int;
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
        s: *mut GEOSCoordSequence,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLineString_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createLinearRing_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
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
    #[cfg(feature = "v3_7_0")]
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
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSFrechetDistance_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        distance: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
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
    pub fn GEOSGeomGetX_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        x: *mut c_double,
    ) -> c_int;
    pub fn GEOSGeomGetY_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        y: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeomGetZ_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        z: *mut c_double,
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
    ) -> *const GEOSCoordSequence;
    pub fn GEOSCoordSeq_getDimensions_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        dims: *mut c_uint,
    ) -> c_int;
    pub fn GEOSPrepare_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *const GEOSPreparedGeometry;
    pub fn GEOSPreparedContains_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedContainsProperly_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCoveredBy_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCovers_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedCrosses_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedDisjoint_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedIntersects_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedOverlaps_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedTouches_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedWithin_r(
        handle: GEOSContextHandle_t,
        pg1: *const GEOSPreparedGeometry,
        g2: *const GEOSGeometry,
    ) -> c_char;
    pub fn GEOSPreparedGeom_destroy_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSPreparedGeometry,
    );
    pub fn GEOSCoordSeq_setOrdinate_r(
        handle: GEOSContextHandle_t,
        s: *mut GEOSCoordSequence,
        idx: c_uint,
        dim: c_uint,
        val: c_double,
    ) -> c_int;
    pub fn GEOSCoordSeq_getOrdinate_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        idx: c_uint,
        dim: c_uint,
        val: *mut c_double,
    ) -> c_int;
    pub fn GEOSGeomGetPointN_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        n: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeomGetStartPoint_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeomGetEndPoint_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_8_0")]
    pub fn GEOSBuildArea_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSLineMerge_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSReverse_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSSimplify_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        tolerance: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSTopologyPreserveSimplify_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        tolerance: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGetInteriorRingN_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        n: c_int,
    ) -> *const GEOSGeometry;
    pub fn GEOSGetExteriorRing_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *const GEOSGeometry;
    pub fn GEOSGetNumInteriorRings_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeomGetNumPoints_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGetNumCoordinates_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeom_getDimensions_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeom_getCoordinateDimension_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> c_int;
    pub fn GEOSMakeValid_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGetNumGeometries_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSGeomType_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> *mut c_char;
    pub fn GEOSGetSRID_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_int;
    pub fn GEOSSetSRID_r(handle: GEOSContextHandle_t, g: *mut GEOSGeometry, srid: c_int);
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSCoordSeq_isCCW_r(
        handle: GEOSContextHandle_t,
        s: *const GEOSCoordSequence,
        is_ccw: *mut c_char,
    ) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSGeom_getPrecision_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> c_double;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSGeom_setPrecision_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        grid_size: c_double,
        flags: c_int,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getXMax_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        value: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getXMin_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        value: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getYMax_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        value: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSGeom_getYMin_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        value: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumClearance_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        d: *mut c_double,
    ) -> c_int;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumClearanceLine_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumRotatedRectangle_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_6_0")]
    pub fn GEOSMinimumWidth_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    #[cfg(feature = "v3_7_0")]
    pub fn GEOSSegmentIntersection_r(
        handle: GEOSContextHandle_t,
        ax0: c_double,
        ay0: c_double,
        ax1: c_double,
        ay1: c_double,
        bx0: c_double,
        by0: c_double,
        bx1: c_double,
        by1: c_double,
        cx: *mut c_double,
        cy: *mut c_double,
    ) -> c_int;
    pub fn GEOSDelaunayTriangulation_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        tolerance: c_double,
        onlyEdges: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyPolygon_r(handle: GEOSContextHandle_t) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyCollection_r(
        handle: GEOSContextHandle_t,
        type_: c_int,
    ) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyLineString_r(handle: GEOSContextHandle_t) -> *mut GEOSGeometry;
    pub fn GEOSGeom_createEmptyPoint_r(handle: GEOSContextHandle_t) -> *mut GEOSGeometry;
    pub fn GEOSGeom_getUserData_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut c_void;
    pub fn GEOSGeom_setUserData_r(
        handle: GEOSContextHandle_t,
        g: *mut GEOSGeometry,
        userData: *mut c_void,
    );
    pub fn GEOSSTRtree_create_r(
        handle: GEOSContextHandle_t,
        nodeCapacity: size_t,
    ) -> *mut GEOSSTRtree;
    pub fn GEOSSTRtree_insert_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        item: *mut c_void,
    );
    pub fn GEOSSTRtree_query_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        callback: GEOSQueryCallback,
        userdata: *mut c_void,
    );
    pub fn GEOSSTRtree_nearest_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
    ) -> *const GEOSGeometry;
    pub fn GEOSSTRtree_nearest_generic_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        item: *const c_void,
        itemEnvelope: *const GEOSGeometry,
        distancefn: GEOSDistanceCallback,
        userdata: *mut c_void,
    ) -> *const c_void;
    pub fn GEOSSTRtree_iterate_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        callback: GEOSQueryCallback,
        userdata: *mut c_void,
    );
    pub fn GEOSSTRtree_remove_r(
        handle: GEOSContextHandle_t,
        tree: *mut GEOSSTRtree,
        g: *const GEOSGeometry,
        item: *mut c_void,
    ) -> c_char;
    pub fn GEOSSTRtree_destroy_r(handle: GEOSContextHandle_t, tree: *mut GEOSSTRtree);
    pub fn GEOSGetGeometryN_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        n: c_int,
    ) -> *const GEOSGeometry;
    pub fn GEOSInterpolate_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        d: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSInterpolateNormalized_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        d: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSProjectNormalized_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        p: *const GEOSGeometry,
    ) -> c_double;
    pub fn GEOSNode_r(handle: GEOSContextHandle_t, g: *const GEOSGeometry) -> *mut GEOSGeometry;
    pub fn GEOSOffsetCurve_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        width: c_double,
        quadsegs: c_int,
        joinStyle: c_int,
        mitreLimit: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSOrientationIndex_r(
        handle: GEOSContextHandle_t,
        ax: c_double,
        ay: c_double,
        bx: c_double,
        by: c_double,
        px: c_double,
        py: c_double,
    ) -> c_int;
    pub fn GEOSPointOnSurface_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSPolygonize_r(
        handle: GEOSContextHandle_t,
        geoms: *const *const GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSPolygonize_full_r(
        handle: GEOSContextHandle_t,
        input: *const GEOSGeometry,
        cuts: *mut *mut GEOSGeometry,
        dangles: *mut *mut GEOSGeometry,
        invalidRings: *mut *mut GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSPolygonizer_getCutEdges_r(
        handle: GEOSContextHandle_t,
        geoms: *const *const GEOSGeometry,
        ngeoms: c_uint,
    ) -> *mut GEOSGeometry;
    pub fn GEOSBufferParams_create_r(
        handle: GEOSContextHandle_t,
    ) -> *mut GEOSBufferParams;
    pub fn GEOSBufferParams_destroy_r(
        handle: GEOSContextHandle_t,
        params: *mut GEOSBufferParams,
    );
    pub fn GEOSBufferParams_setEndCapStyle_r(
        handle: GEOSContextHandle_t,
        p: *mut GEOSBufferParams,
        style: c_int,
    ) -> c_int;
    pub fn GEOSBufferParams_setJoinStyle_r(
        handle: GEOSContextHandle_t,
        p: *mut GEOSBufferParams,
        joinStyle: c_int,
    ) -> c_int;
    pub fn GEOSBufferParams_setMitreLimit_r(
        handle: GEOSContextHandle_t,
        p: *mut GEOSBufferParams,
        mitreLimit: c_double,
    ) -> c_int;
    pub fn GEOSBufferParams_setQuadrantSegments_r(
        handle: GEOSContextHandle_t,
        p: *mut GEOSBufferParams,
        quadSegs: c_int,
    ) -> c_int;
    pub fn GEOSBufferParams_setSingleSided_r(
        handle: GEOSContextHandle_t,
        p: *mut GEOSBufferParams,
        singleSided: c_int,
    ) -> c_int;
    pub fn GEOSBufferWithParams_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        p: *const GEOSBufferParams,
        width: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSBufferWithStyle_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        width: c_double,
        quadSegs: c_int,
        endCapStyle: c_int,
        joinStyle: c_int,
        mitreLimit: c_double,
    ) -> *mut GEOSGeometry;
    pub fn GEOSProject_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        p: *const GEOSGeometry,
    ) -> c_double;
    pub fn GEOSRelatePattern_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        pat: *const c_char,
    ) -> c_char;
    pub fn GEOSRelate_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut c_char;
    pub fn GEOSRelatePatternMatch_r(
        handle: GEOSContextHandle_t,
        mat: *const c_char,
        pat: *const c_char,
    ) -> c_char;
    pub fn GEOSRelateBoundaryNodeRule_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
        bnr: c_int,
    ) -> *mut c_char;
    pub fn GEOSSharedPaths_r(
        handle: GEOSContextHandle_t,
        g1: *const GEOSGeometry,
        g2: *const GEOSGeometry,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKTWriter_getOutputDimension_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
    ) -> c_int;
    pub fn GEOSWKTWriter_setOutputDimension_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
        dim: c_int,
    );
    pub fn GEOSWKTWriter_setTrim_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
        trim: c_char,
    );
    pub fn GEOSWKTWriter_setOld3D_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKTWriter,
        useOld3D: c_int,
    );

    pub fn GEOSWKBReader_create_r(
        handle: GEOSContextHandle_t,
    ) -> *mut GEOSWKBReader;
    pub fn GEOSWKBReader_destroy_r(
        handle: GEOSContextHandle_t,
        reader: *mut GEOSWKBReader,
    );
    pub fn GEOSWKBReader_read_r(
        handle: GEOSContextHandle_t,
        reader: *mut GEOSWKBReader,
        wkb: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKBReader_readHEX_r(
        handle: GEOSContextHandle_t,
        reader: *mut GEOSWKBReader,
        hex: *const c_uchar,
        size: size_t,
    ) -> *mut GEOSGeometry;
    pub fn GEOSWKBWriter_create_r(
        handle: GEOSContextHandle_t,
    ) -> *mut GEOSWKBWriter;
    pub fn GEOSWKBWriter_destroy_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
    );
    pub fn GEOSWKBWriter_write_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSWKBWriter_writeHEX_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
        g: *const GEOSGeometry,
        size: *mut size_t,
    ) -> *mut c_uchar;
    pub fn GEOSWKBWriter_getOutputDimension_r(
        handle: GEOSContextHandle_t,
        writer: *const GEOSWKBWriter,
    ) -> c_int;
    pub fn GEOSWKBWriter_setOutputDimension_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
        newDimension: c_int,
    );
    pub fn GEOSWKBWriter_getByteOrder_r(
        handle: GEOSContextHandle_t,
        writer: *const GEOSWKBWriter,
    ) -> c_int;
    pub fn GEOSWKBWriter_setByteOrder_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
        byteOrder: c_int,
    );
    pub fn GEOSWKBWriter_getIncludeSRID_r(
        handle: GEOSContextHandle_t,
        writer: *const GEOSWKBWriter,
    ) -> c_char;
    pub fn GEOSWKBWriter_setIncludeSRID_r(
        handle: GEOSContextHandle_t,
        writer: *mut GEOSWKBWriter,
        writeSRID: c_char,
    );
    pub fn GEOSisValidDetail_r(
        handle: GEOSContextHandle_t,
        g: *const GEOSGeometry,
        flags: c_int,
        reason: *mut *mut c_char,
        location: *mut *mut GEOSGeometry,
    ) -> c_char;
}
