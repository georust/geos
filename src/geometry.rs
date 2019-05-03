use crate::{
    CoordSeq, ContextHandle, AsRaw, ContextHandling, ContextInteractions, PreparedGeometry,
    WKTWriter,
};
#[cfg(feature = "v3_6_0")]
use crate::Precision;
use context_handle::PtrWrap;
use enums::*;
use error::{Error, GResult, PredicateType};
use geos_sys::*;
use functions::*;
use std::ffi::CString;
use std::{self, str};
use c_vec::CVec;
use std::sync::Arc;

pub struct Geometry<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSGeometry>,
    context: Arc<ContextHandle<'a>>,
    owned: bool,
}

impl<'a> Geometry<'a> {
    /// Same as [`new_from_wkt_s`] except it internally uses a reader instead of just using the
    /// given string.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// ```
    pub fn new_from_wkt(wkt: &str) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::new_from_wkt")) {
            Ok(context_handle) => {
                match CString::new(wkt) {
                    Ok(c_str) => {
                        unsafe {
                            let reader = GEOSWKTReader_create_r(context_handle.as_raw());
                            let ptr = GEOSWKTReader_read_r(context_handle.as_raw(), reader,
                                                           c_str.as_ptr());
                            GEOSWKTReader_destroy_r(context_handle.as_raw(), reader);
                            Geometry::new_from_raw(ptr, Arc::new(context_handle), "new_from_wkt")
                        }
                    }
                    Err(e) => {
                        Err(Error::GenericError(format!("Conversion to CString failed: {}", e)))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`Geometry`] from the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to HEX failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = Geometry::new_from_hex(hex_buf.as_ref())
    ///                      .expect("conversion from HEX failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_hex(hex: &[u8]) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::new_from_hex")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromHEX_buf_r(context.as_raw(), hex.as_ptr(), hex.len());
                    Geometry::new_from_raw(ptr, Arc::new(context), "new_from_hex")
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new [`Geometry`] from the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    ///
    /// // The interesting part is here:
    /// let new_geom = Geometry::new_from_wkb(wkb_buf.as_ref())
    ///                      .expect("conversion from WKB failed");
    /// assert!(point_geom.equals(&new_geom) == Ok(true));
    /// ```
    pub fn new_from_wkb(wkb: &[u8]) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::new_from_wkb")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeomFromWKB_buf_r(context.as_raw(), wkb.as_ptr(), wkb.len());
                    Geometry::new_from_raw(ptr, Arc::new(context), "new_from_wkb")
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Converts a [`Geometry`] to the HEX format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to WKB failed");
    /// ```
    pub fn to_hex(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Converts a [`Geometry`] to the WKB format.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    /// ```
    pub fn to_wkb(&self) -> Option<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                None
            } else {
                Some(CVec::new(ptr, size as _))
            }
        }
    }

    /// Creates a new [`PreparedGeometry`] from the current `Geometry`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let prepared_geom = point_geom.to_prepared_geom().expect("failed to create prepared geom");
    /// ```
    pub fn to_prepared_geom(&self) -> GResult<PreparedGeometry<'a>> {
        PreparedGeometry::new(self)
    }

    #[cfg(feature = "v3_8_0")]
    pub fn build_area(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSBuildArea_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "build_area")
        }
    }

    pub fn polygonize<T: AsRef<Geometry<'a>>>(geometries: &[T]) -> GResult<Geometry<'a>> {
        unsafe {
            let context = match geometries.get(0) {
                Some(g) => g.as_ref().clone_context(),
                None => {
                    match ContextHandle::init_e(Some("Geometry::polygonize")) {
                        Ok(context) => Arc::new(context),
                        Err(e) => return Err(e),
                    }
                }
            };
            let geoms = geometries.iter()
                                  .map(|g| g.as_ref().as_raw() as *const _)
                                  .collect::<Vec<_>>();
            let ptr = GEOSPolygonize_r(context.as_raw(), geoms.as_ptr(), geoms.len() as _);
            Geometry::new_from_raw(ptr, context, "polygonize")
        }
    }

    pub fn polygonizer_get_cut_edges<T: AsRef<Geometry<'a>>>(
        &self,
        geometries: &[T],
    ) -> GResult<Geometry<'a>> {
        unsafe {
            let context = match geometries.get(0) {
                Some(g) => g.as_ref().clone_context(),
                None => {
                    match ContextHandle::init_e(Some("Geometry::polygonizer_get_cut_edges")) {
                        Ok(context) => Arc::new(context),
                        Err(e) => return Err(e),
                    }
                }
            };
            let geoms = geometries.iter()
                                  .map(|g| g.as_ref().as_raw() as *const _)
                                  .collect::<Vec<_>>();
            let ptr = GEOSPolygonizer_getCutEdges_r(
                context.as_raw(),
                geoms.as_ptr(),
                geoms.len() as _,
            );
            Geometry::new_from_raw(ptr, context, "polygonizer_get_cut_edges")
        }
    }

    /// Merges `Multi Line String` geometry into a (set of) `Line String`.
    ///
    /// ### Warning
    ///
    /// If you use this function on something else than a `Multi Line String` or a
    /// `Line String`, it'll return an empty `Geometry collection`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let lines = Geometry::new_from_wkt("MULTILINESTRING((-29 -27,-30 -29.7,-36 -31,-45 -33),\
    ///                                                  (-45 -33,-46 -32))")
    ///                   .expect("Invalid geometry");
    /// let lines_merged = lines.line_merge().expect("line merge failed");
    /// assert_eq!(lines_merged.to_wkt().unwrap(),
    ///            "LINESTRING (-29.0000000000000000 -27.0000000000000000, \
    ///                         -30.0000000000000000 -29.6999999999999993, \
    ///                         -36.0000000000000000 -31.0000000000000000, \
    ///                         -45.0000000000000000 -33.0000000000000000, \
    ///                         -46.0000000000000000 -32.0000000000000000)");
    /// ```
    pub fn line_merge(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSLineMerge_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "line_merge")
        }
    }

    #[cfg(feature = "v3_7_0")]
    pub fn reverse(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSReverse_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "reverse")
        }
    }

    pub fn simplify(&self, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSimplify_r(self.get_raw_context(), self.as_raw(), tolerance);
            Geometry::new_from_raw(ptr, self.clone_context(), "simplify")
        }
    }

    pub fn topology_preserve_simplify(&self, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSTopologyPreserveSimplify_r(
                self.get_raw_context(),
                self.as_raw(),
                tolerance);
            Geometry::new_from_raw(ptr, self.clone_context(), "topology_preserve_simplify")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSGeometry,
        context: Arc<ContextHandle<'a>>,
        caller: &str,
    ) -> GResult<Geometry<'a>> {
        if ptr.is_null() {
            return Err(Error::NoConstructionFromNullPtr(format!("Geometry::{}", caller)));
        }
        Ok(Geometry { ptr: PtrWrap(ptr), context, owned: true, })
    }

    /// Checks if the geometry is valid.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                        .expect("Invalid geometry");
    /// assert!(point_geom.is_valid() == false);
    /// ```
    pub fn is_valid(&self) -> bool {
        unsafe { GEOSisValid_r(self.get_raw_context(), self.as_raw()) == 1 }
    }

    /// Returns an explanation on why the geometry is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                        .expect("Invalid geometry");
    /// assert_eq!(point_geom.is_valid_reason(), Ok("Self-intersection[0 0]".to_owned()));
    /// ```
    pub fn is_valid_reason(&self) -> GResult<String> {
        unsafe {
            let ptr = GEOSisValidReason_r(self.get_raw_context(), self.as_raw());
            managed_string(ptr, self.get_context_handle(), "GGeom::is_valid_reason")
        }
    }

    /// Returns the type of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                        .expect("Invalid geometry");
    /// assert_eq!(point_geom.get_type(), Ok("Polygon".to_owned()));
    /// ```
    pub fn get_type(&self) -> GResult<String> {
        unsafe {
            let ptr = GEOSGeomType_r(self.get_raw_context(), self.as_raw());
            managed_string(ptr, self.get_context_handle(), "GGeom::get_type")
        }
    }

    /// Get the underlying geos CoordSeq object from the geometry
    ///
    /// Note: this clones the underlying CoordSeq to avoid double free
    /// (because CoordSeq handles the object ptr and the CoordSeq is still owned by the geos
    /// geometry) if this method's performance becomes a bottleneck, feel free to open an issue,
    /// we could skip this clone with cleaner code
    pub fn get_coord_seq(&self) -> GResult<CoordSeq<'a>> {
        let type_geom = self.geometry_type();
        match type_geom {
            GeometryTypes::Point | GeometryTypes::LineString | GeometryTypes::LinearRing => unsafe {
                let coord = GEOSGeom_getCoordSeq_r(self.get_raw_context(), self.as_raw());
                let t = GEOSCoordSeq_clone_r(self.get_raw_context(), coord);
                let mut size = 0;
                let mut dims = 0;

                if GEOSCoordSeq_getSize_r(self.get_raw_context(), coord, &mut size) == 0 {
                    return Err(Error::GenericError("GEOSCoordSeq_getSize_r failed".to_owned()));
                }
                if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), coord, &mut dims) == 0 {
                    return Err(Error::GenericError("GEOSCoordSeq_getDimensions_r failed".to_owned()));
                }
                CoordSeq::new_from_raw(t, self.clone_context(), size, dims, "get_coord_seq")
            },
            _ => Err(Error::ImpossibleOperation(
                "Geometry must be a Point, LineString or LinearRing to extract its coordinates"
                    .into(),
            )),
        }
    }

    pub fn geometry_type(&self) -> GeometryTypes {
        let type_geom = unsafe { GEOSGeomTypeId_r(self.get_raw_context(), self.as_raw()) as i32 };

        GeometryTypes::from(type_geom)
    }

    pub fn area(&self) -> GResult<f64> {
        let mut n = 0.;

        let res = unsafe { GEOSArea_r(self.get_raw_context(), self.as_raw(), &mut n) };
        if res != 1 {
            Err(Error::GeosError(format!("area failed with code {}", res)))
        } else {
            Ok(n as f64)
        }
    }

    /// Returns a WKT representation of the geometry. It defaults to 2 dimensions output. Use
    /// [`WKTWriter`] type directly if you want more control.
    ///
    /// # Examples
    ///
    /// ```
    /// use geos::{Geometry, OutputDimension, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// assert_eq!(point_geom.to_wkt().unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    ///
    /// // A three dimension point will be output just as a 2 dimension:
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 3)").expect("Invalid geometry");
    /// assert_eq!(point_geom.to_wkt().unwrap(), "POINT (2.5000000000000000 2.5000000000000000)");
    ///
    /// // To "fix" it, use `WKTWriter` instead:
    /// let mut wkt_writer = WKTWriter::new().expect("Failed to create WKTWriter");
    /// wkt_writer.set_output_dimension(OutputDimension::ThreeD);
    /// assert_eq!(wkt_writer.write(&point_geom).unwrap(),
    ///            "POINT Z (2.5000000000000000 2.5000000000000000 3.0000000000000000)");
    /// ```
    pub fn to_wkt(&self) -> GResult<String> {
        match WKTWriter::new_with_context(self.clone_context()) {
            Ok(w) => w.write(self),
            Err(e) => Err(e),
        }
    }


    /// Returns a WKT representation of the geometry with the given `precision`. It is a wrapper
    /// around [`WKTWriter::set_rounding_precision`].
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, WKTWriter};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// assert_eq!(point_geom.to_wkt_precision(2).unwrap(), "POINT (2.50 2.50)");
    ///
    /// // It is a wrapper around:
    /// let mut writer = WKTWriter::new().expect("Failed to create WKTWriter");
    /// writer.set_rounding_precision(2);
    /// assert_eq!(writer.write(&point_geom).unwrap(), "POINT (2.50 2.50)");
    /// ```
    pub fn to_wkt_precision(&self, precision: u32) -> GResult<String> {
        unsafe {
            let writer = GEOSWKTWriter_create_r(self.get_raw_context());
            GEOSWKTWriter_setRoundingPrecision_r(self.get_raw_context(), writer, precision as _);
            let c_result = GEOSWKTWriter_write_r(self.get_raw_context(), writer, self.as_raw());
            GEOSWKTWriter_destroy_r(self.get_raw_context(), writer);
            managed_string(c_result, self.get_context_handle(), "GResult::to_wkt_precision")
        }
    }

    /// Returns `true` if the geometry is a ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let circle = Geometry::new_from_wkt("LINESTRING(0 0, 0 1, 1 1, 0 0)").expect("Invalid geometry");
    /// assert_eq!(circle.is_ring(), Ok(true));
    /// ```
    pub fn is_ring(&self) -> GResult<bool> {
        let rv = unsafe { GEOSisRing_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(rv as _, PredicateType::IsRing)
    }

    pub fn intersects<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSIntersects_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    pub fn crosses<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCrosses_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    pub fn disjoint<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSDisjoint_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    pub fn touches<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSTouches_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    pub fn overlaps<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSOverlaps_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    pub fn within<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSWithin_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Within)
    }

    /// Checks if the two [`Geometry`] objects are equal.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1.equals(&geom2) == Ok(false));
    /// assert!(geom1.equals(&geom3) == Ok(true));
    /// ```
    ///
    /// Note that you can also use method through the `PartialEq` trait:
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.8 3.8)").expect("Invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert!(geom1 != geom2);
    /// assert!(geom1 == geom3);
    /// ```
    pub fn equals<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSEquals_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    pub fn equals_exact<'b>(&self, g2: &Geometry<'b>, precision: f64) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact_r(self.get_raw_context(), self.as_raw(), g2.as_raw(), precision)
        };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    pub fn covers<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCovers_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    pub fn covered_by<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSCoveredBy_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    pub fn contains<'b>(&self, g2: &Geometry<'b>) -> GResult<bool> {
        let ret_val = unsafe { GEOSContains_r(self.get_raw_context(), self.as_raw(), g2.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<Geometry<'a>> {
        assert!(quadsegs > 0);
        unsafe {
            let ptr = GEOSBuffer_r(
                self.get_raw_context(),
                self.as_raw(),
                width,
                quadsegs as _,
            );
            Geometry::new_from_raw(ptr, self.clone_context(), "buffer")
        }
    }

    pub fn is_empty(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn difference<'b>(&self, g2: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSDifference_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "difference")
        }
    }

    pub fn envelope(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSEnvelope_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "envelope")
        }
    }

    pub fn sym_difference<'b>(&self, g2: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSymDifference_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "sym_difference")
        }
    }

    pub fn union(&self, g2: &Geometry<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSUnion_r(self.get_raw_context(), self.as_raw(), g2.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "union")
        }
    }

    pub fn get_centroid(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGetCentroid_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_centroid")
        }
    }

    pub fn create_empty_polygon() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_polygon")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeom_createEmptyPolygon_r(context.as_raw());
                    Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_polygon")
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_empty_point() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_point")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeom_createEmptyPoint_r(context.as_raw());
                    Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_point")
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_empty_line_string() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_line_string")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeom_createEmptyLineString_r(context.as_raw());
                    Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_line_string")
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_empty_collection(type_: GeometryTypes) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_collection")) {
            Ok(context) => {
                unsafe {
                    let ptr = GEOSGeom_createEmptyCollection_r(context.as_raw(), type_.into());
                    Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_collection")
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_polygon<'b>(mut exterior: Geometry<'a>, mut interiors: Vec<Geometry<'b>>) -> GResult<Geometry<'a>> {
        let context_handle = exterior.clone_context();
        let nb_interiors = interiors.len();
        let res = unsafe {
            let mut geoms: Vec<*mut GEOSGeometry> = interiors.iter_mut().map(|g| g.as_raw()).collect();
            let ptr = GEOSGeom_createPolygon_r(
                context_handle.as_raw(),
                exterior.as_raw(),
                geoms.as_mut_ptr() as *mut *mut GEOSGeometry,
                nb_interiors as _,
            );
            Geometry::new_from_raw(ptr, context_handle, "create_polygon")
        };

        // We transfered the ownership of the ptr to the new Geometry,
        // so the old ones need to forget their c ptr to avoid double free.
        exterior.ptr = PtrWrap(::std::ptr::null_mut());
        for i in interiors.iter_mut() {
            i.ptr = PtrWrap(::std::ptr::null_mut());
        }

        res
    }

    pub fn create_geometry_collection(geoms: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        create_multi_geom(geoms, GeometryTypes::GeometryCollection)
    }

    pub fn create_multipolygon(polygons: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&polygons, GeometryTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_owned(),
            ));
        }
        create_multi_geom(polygons, GeometryTypes::MultiPolygon)
    }

    pub fn create_multiline_string(linestrings: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&linestrings, GeometryTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_owned(),
            ));
        }
        create_multi_geom(linestrings, GeometryTypes::MultiLineString)
    }

    pub fn create_multipoint(points: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&points, GeometryTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_owned(),
            ));
        }
        create_multi_geom(points, GeometryTypes::MultiPoint)
    }

    pub fn create_point(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createPoint_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_point");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn create_line_string(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLineString_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_line_string");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn create_linear_ring(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLinearRing_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_linear_ring");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    pub fn unary_union(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSUnaryUnion_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "unary_union")
        }
    }

    pub fn voronoi<'b>(
        &self,
        envelope: Option<&Geometry<'b>>,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<Geometry<'a>> {
        unsafe {
            let raw_voronoi = GEOSVoronoiDiagram_r(
                self.get_raw_context(),
                self.as_raw(),
                envelope
                    .map(|e| e.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                tolerance,
                only_edges as _,
            );
            Self::new_from_raw(raw_voronoi, self.clone_context(), "voronoi")
        }
    }

    pub fn normalize(&mut self) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    pub fn intersection<'b>(&self, other: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSIntersection_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "intersection")
        }
    }

    pub fn convex_hull(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSConvexHull_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "convex_hull")
        }
    }

    pub fn boundary(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSBoundary_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "boundary")
        }
    }

    pub fn has_z(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn is_closed(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisClosed_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    pub fn length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    #[cfg(feature = "v3_7_0")]
    pub fn distance_indexed<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        unsafe {
            let mut distance = 0.;
            if GEOSDistanceIndexed_r(self.get_raw_context(),
                                     self.as_raw(),
                                     other.as_raw(),
                                     &mut distance) != 1 {
                Err(Error::GenericError("GEOSDistanceIndexed_r failed".to_owned()))
            } else {
                Ok(distance)
            }
        }
    }

    pub fn hausdorff_distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn hausdorff_distance_densify<'b>(&self, other: &Geometry<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    #[cfg(feature = "v3_7_0")]
    pub fn frechet_distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    #[cfg(feature = "v3_7_0")]
    pub fn frechet_distance_densify<'b>(&self, other: &Geometry<'b>, distance_frac: f64) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance);
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    pub fn get_length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    pub fn snap<'b>(&self, other: &Geometry<'b>, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSnap_r(self.get_raw_context(), self.as_raw(), other.as_raw(), tolerance);
            Geometry::new_from_raw(ptr, self.clone_context(), "snap")
        }
    }

    pub fn extract_unique_points(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_extractUniquePoints_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "extract_unique_points")
        }
    }

    pub fn nearest_points<'b>(&self, other: &Geometry<'b>) -> GResult<CoordSeq<'a>> {
        unsafe {
            let ptr = GEOSNearestPoints_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
            );
            let mut size = 0;
            let mut dims = 0;

            if GEOSCoordSeq_getSize_r(self.get_raw_context(), ptr, &mut size) == 0 {
                return Err(Error::GenericError("GEOSCoordSeq_getSize_r failed".to_owned()));
            }
            if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), ptr, &mut dims) == 0 {
                return Err(Error::GenericError("GEOSCoordSeq_getDimensions_r failed".to_owned()));
            }
            CoordSeq::new_from_raw(ptr, self.clone_context(), size, dims, "nearest_points")
        }
    }

    /// Returns the X position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_x() == Ok(1.5));
    /// ```
    pub fn get_x(&self) -> GResult<f64> {
        if self.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut x = 0.;
        unsafe {
            if GEOSGeomGetX_r(self.get_raw_context(), self.as_raw(), &mut x) == 1 {
                Ok(x)
            } else {
                Err(Error::GenericError("GEOSGeomGetX_r failed".to_owned()))
            }
        }
    }

    /// Returns the Y position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (1.5 2.5 3.5)").expect("Invalid geometry");
    /// assert!(point_geom.get_y() == Ok(2.5));
    /// ```
    pub fn get_y(&self) -> GResult<f64> {
        if self.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut y = 0.;
        unsafe {
            if GEOSGeomGetY_r(self.get_raw_context(), self.as_raw(), &mut y) == 1 {
                Ok(y)
            } else {
                Err(Error::GenericError("GEOSGeomGetY_r failed".to_owned()))
            }
        }
    }

    /// Returns the Z position. The given `Geometry` must be a `Point`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_z() == Ok(4.0));
    /// ```
    #[cfg(feature = "v3_7_0")]
    pub fn get_z(&self) -> GResult<f64> {
        if self.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError("Geometry must be a point".to_owned()));
        }
        let mut z = 0.;
        unsafe {
            if GEOSGeomGetZ_r(self.get_raw_context(), self.as_raw(), &mut z) == 1 {
                Ok(z)
            } else {
                Err(Error::GenericError("GEOSGeomGetZ_r failed".to_owned()))
            }
        }
    }

    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    pub fn get_point_n(&self, n: usize) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetPointN_r(self.get_raw_context(), self.as_raw(), n as _);
            Geometry::new_from_raw(ptr, self.clone_context(), "get_point_n")
        }
    }

    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    pub fn get_start_point(&self) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetStartPoint_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_start_point")
        }
    }

    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    pub fn get_end_point(&self) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSGeomGetEndPoint_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_end_point")
        }
    }

    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    pub fn get_num_points(&self) -> GResult<usize> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ret = GEOSGeomGetNumPoints_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError("GEOSGeomGetNumPoints_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Returns the number of interior rings.
    pub fn get_num_interior_rings(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetNumInteriorRings_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError("GEOSGetNumInteriorRings_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Returns the nth interior ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                               (1 1, 2 1, 2 5, 1 5, 1 1),\
    ///                                               (8 5, 8 4, 9 4, 9 5, 8 5))")
    ///                        .expect("Invalid geometry");
    /// let interior = point_geom.get_interior_ring_n(0).expect("failed to get interior ring");
    /// assert_eq!(interior.to_wkt().unwrap(),
    ///            "LINEARRING (1.0000000000000000 1.0000000000000000, \
    ///                         2.0000000000000000 1.0000000000000000, \
    ///                         2.0000000000000000 5.0000000000000000, \
    ///                         1.0000000000000000 5.0000000000000000, \
    ///                         1.0000000000000000 1.0000000000000000)");
    /// ```
    pub fn get_interior_ring_n(&self, n: u32) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGetInteriorRingN_r(self.get_raw_context(), self.as_raw(), n as _);
            match Geometry::new_from_raw(ptr, self.clone_context(), "get_interior_ring_n") {
                Ok(mut g) => {
                    g.owned = false;
                    Ok(g)
                }
                e => e,
            }
        }
    }

    /// Returns the exterior ring.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                               (1 1, 2 1, 2 5, 1 5, 1 1))")
    ///                        .expect("Invalid geometry");
    ///
    /// let exterior = point_geom.get_exterior_ring().expect("failed to get exterior ring");
    /// assert_eq!(exterior.to_wkt().unwrap(),
    ///            "LINEARRING (0.0000000000000000 0.0000000000000000, \
    ///                         10.0000000000000000 0.0000000000000000, \
    ///                         10.0000000000000000 6.0000000000000000, \
    ///                         0.0000000000000000 6.0000000000000000, \
    ///                         0.0000000000000000 0.0000000000000000)");
    /// ```
    pub fn get_exterior_ring(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGetExteriorRing_r(self.get_raw_context(), self.as_raw());
            match Geometry::new_from_raw(ptr, self.clone_context(), "get_exterior_ring") {
                Ok(mut g) => {
                    g.owned = false;
                    Ok(g)
                }
                e => e,
            }
        }
    }

    pub fn get_num_coordinates(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetNumCoordinates_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError("GEOSGetNumCoordinates_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    pub fn get_num_dimensions(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGeom_getDimensions_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError("GEOSGeom_getDimensions_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Return in which coordinate dimension the geometry is.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Dimensions, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_coordinate_dimension() == Ok(Dimensions::ThreeD));
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_coordinate_dimension() == Ok(Dimensions::TwoD));
    /// ```
    pub fn get_coordinate_dimension(&self) -> GResult<Dimensions> {
        unsafe {
            let ret = GEOSGeom_getCoordinateDimension_r(self.get_raw_context(), self.as_raw());
            if ret != 2 && ret != 3 {
                Err(Error::GenericError("GEOSGeom_getCoordinateDimension_r failed".to_owned()))
            } else {
                Ok(Dimensions::from(ret))
            }
        }
    }

    #[cfg(feature = "v3_8_0")]
    pub fn make_valid(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMakeValid_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "make_valid")
        }
    }

    /// Returns the number of geometries.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(77.29 29.07,77.42 29.26,77.27 29.31,77.29 29.07)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_num_geometries(), Ok(1));
    ///
    /// let geom = Geometry::new_from_wkt("GEOMETRYCOLLECTION(MULTIPOINT(-2 3 , -2 2),\
    ///                                                       LINESTRING(5 5 ,10 10),\
    ///                                                       POLYGON((-7 4.2,-7.1 5,-7.1 4.3,-7 4.2)))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_num_geometries(), Ok(3));
    /// ```
    pub fn get_num_geometries(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetNumGeometries_r(self.get_raw_context(), self.as_raw());
            if ret < 1 {
                Err(Error::GenericError("GEOSGetNumGeometries_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Returns the 1-based nth geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("MULTIPOINT(1 1, 2 2, 3 3, 4 4)")
    ///                     .expect("Invalid geometry");
    /// let point_nb3 = geom.get_geometry_n(2).expect("failed to get third point");
    /// assert_eq!(point_nb3.to_wkt().unwrap(), "POINT (3.0000000000000000 3.0000000000000000)");
    /// ```
    pub fn get_geometry_n(&self, n: usize) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGetGeometryN_r(self.get_raw_context(), self.as_raw(), n as _);
            Geometry::new_from_raw(ptr, self.clone_context(), "get_geometry_n").map(|mut x| {
                x.owned = false;
                x
            })
        }
    }

    /// Get SRID of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// point_geom.set_srid(4326);
    /// assert_eq!(point_geom.get_srid(), Ok(4326));
    /// ```
    pub fn get_srid(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetSRID_r(self.get_raw_context(), self.as_raw());
            if ret < 1 {
                Err(Error::GenericError("GEOSGetSRID_r failed".to_owned()))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Set SRID of the geometry.
    pub fn set_srid(&self, srid: usize) -> GResult<()> {
        unsafe {
            if GEOSSetSRID_r(self.get_raw_context(), self.as_raw(), srid as _) == 0 {
                Err(Error::GenericError("GEOSSetSRID_r failed".to_owned()))
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn get_precision(&self) -> GResult<f64> {
        unsafe {
            let ret = GEOSGeom_getPrecision_r(self.get_raw_context(), self.as_raw());
            if ret == -1. {
                Err(Error::GenericError("GEOSGeom_getPrecision_r failed".to_owned()))
            } else {
                Ok(ret)
            }
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn set_precision(&self, grid_size: f64, flags: Precision) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_setPrecision_r(self.get_raw_context(),
                                              self.as_raw(),
                                              grid_size,
                                              flags.into());
            Geometry::new_from_raw(ptr, self.clone_context(), "set_precision")
        }
    }

    /// Returns the biggest X of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_max(), Ok(5.));
    /// ```
    #[cfg(feature = "v3_7_0")]
    pub fn get_x_max(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSGeom_getXMax_r(self.get_raw_context(), self.as_raw(), &mut value) == 0 {
                Err(Error::GenericError("GEOSGeom_getXMax_r failed".to_owned()))
            } else {
                Ok(value)
            }
        }
    }

    /// Returns the smallest X of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_min(), Ok(1.));
    /// ```
    #[cfg(feature = "v3_7_0")]
    pub fn get_x_min(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSGeom_getXMin_r(self.get_raw_context(), self.as_raw(), &mut value) == 0 {
                Err(Error::GenericError("GEOSGeom_getXMin_r failed".to_owned()))
            } else {
                Ok(value)
            }
        }
    }

    /// Returns the biggest Y of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_max(), Ok(6.));
    /// ```
    #[cfg(feature = "v3_7_0")]
    pub fn get_y_max(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSGeom_getYMax_r(self.get_raw_context(), self.as_raw(), &mut value) == 0 {
                Err(Error::GenericError("GEOSGeom_getYMax_r failed".to_owned()))
            } else {
                Ok(value)
            }
        }
    }

    /// Returns the smallest Y of the geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_min(), Ok(3.));
    /// ```
    #[cfg(feature = "v3_7_0")]
    pub fn get_y_min(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSGeom_getYMin_r(self.get_raw_context(), self.as_raw(), &mut value) == 0 {
                Err(Error::GenericError("GEOSGeom_getYMin_r failed".to_owned()))
            } else {
                Ok(value)
            }
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn minimum_clearance(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSMinimumClearance_r(self.get_raw_context(), self.as_raw(), &mut value) != 0 {
                Err(Error::GenericError("GEOSMinimumClearance_r failed".to_owned()))
            } else {
                Ok(value)
            }
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn minimum_clearance_line(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumClearanceLine_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_clearance_line")
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn minimum_rotated_rectangle(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumRotatedRectangle_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_rotated_rectangle")
        }
    }

    #[cfg(feature = "v3_6_0")]
    pub fn minimum_width(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumWidth_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_width")
        }
    }

    pub fn delaunay_triangulation(&self, tolerance: f64, only_edges: bool) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSDelaunayTriangulation_r(
                self.get_raw_context(),
                self.as_raw(),
                tolerance,
                only_edges as _,
            );
            Geometry::new_from_raw(ptr, self.clone_context(), "delaunay_triangulation")
        }
    }

    pub fn interpolate(&self, d: f64) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSInterpolate_r(self.get_raw_context(), self.as_raw(), d);
            Geometry::new_from_raw(ptr, self.clone_context(), "interpolate")
        }
    }

    pub fn interpolate_normalized(&self, d: f64) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSInterpolateNormalized_r(self.get_raw_context(), self.as_raw(), d);
            Geometry::new_from_raw(ptr, self.clone_context(), "interpolate_normalized")
        }
    }

    pub fn project(&self, p: &Geometry<'_>) -> GResult<f64> {
        if p.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError("Second geometry must be a Point".to_owned()));
        }
        unsafe {
            let ret = GEOSProject_r(self.get_raw_context(), self.as_raw(), p.as_raw());
            if ret == -1. {
                Err(Error::GenericError("GEOSProject_r failed".to_owned()))
            } else {
                Ok(ret)
            }
        }
    }

    pub fn project_normalized(&self, p: &Geometry<'_>) -> GResult<f64> {
        if p.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError("Second geometry must be a Point".to_owned()));
        }
        unsafe {
            let ret = GEOSProjectNormalized_r(self.get_raw_context(), self.as_raw(), p.as_raw());
            if ret == -1. {
                Err(Error::GenericError("GEOSProjectNormalized_r failed".to_owned()))
            } else {
                Ok(ret)
            }
        }
    }

    pub fn node(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSNode_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "node")
        }
    }

    ///  Return an offset line at a given distance and side from an input line. All points of the
    /// returned geometries are not further than the given distance from the input geometry.
    ///
    /// ### Parameters description:
    ///
    /// #### width
    ///
    /// * If `width` is positive, the offset will be at the left side of the input line and retain
    ///   the same direction.
    /// * If `width` is negative, it'll be at the right side and in the opposite direction.
    ///
    /// #### quadrant_segments
    ///
    /// * If `quadrant_segments` is >= 1, joins are round, and `quadrant_segments` indicates the
    ///   number of segments to use to approximate a quarter-circle.
    /// * If `quadrant_segments` == 0, joins are bevelled (flat).
    /// * If `quadrant_segments` < 0, joins are mitred, and the value of `quadrant_segments`
    ///   indicates the mitre ration limit as `mitre_limit = |quadrant_segments|`
    ///
    /// #### mitre_limit
    ///
    /// The mitre ratio is the ratio of the distance from the corner to the end of the mitred offset
    /// corner. When two line segments meet at a sharp angle, a miter join will extend far beyond
    /// the original geometry (and in the extreme case will be infinitely far). To prevent
    /// unreasonable geometry, the mitre limit allows controlling the maximum length of the join
    /// corner. Corners with a ratio which exceed the limit will be beveled.
    pub fn offset_curve(
        &self,
        width: f64,
        quadrant_segments: i32,
        join_style: JoinStyle,
        mitre_limit: f64,
    ) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError("Geometry must be a LineString".to_owned()));
        }
        unsafe {
            let ptr = GEOSOffsetCurve_r(self.get_raw_context(), self.as_raw(), width,
                                        quadrant_segments, join_style.into(), mitre_limit);
            Geometry::new_from_raw(ptr, self.clone_context(), "offset_curve")
        }
    }

    pub fn point_on_surface(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSPointOnSurface_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "point_on_surface")
        }
    }

    /// Returns, in the tuple elements order:
    ///
    /// 1. The polygonized geometry.
    /// 2. The cuts geometries collection.
    /// 3. The dangles geometries collection.
    /// 4. The invalid geometries collection.
    pub fn polygonize_full(
        &self,
    ) -> GResult<(Geometry<'a>, Option<Geometry<'a>>, Option<Geometry<'a>>, Option<Geometry<'a>>)> {
        let mut cuts: *mut GEOSGeometry = ::std::ptr::null_mut();
        let mut dangles: *mut GEOSGeometry = ::std::ptr::null_mut();
        let mut invalids: *mut GEOSGeometry = ::std::ptr::null_mut();

        unsafe {
            let ptr = GEOSPolygonize_full_r(
                self.get_raw_context(),
                self.as_raw(),
                &mut cuts,
                &mut dangles,
                &mut invalids,
            );
            let cuts = if !cuts.is_null() {
                Geometry::new_from_raw(cuts, self.clone_context(), "polygonize_full").ok()
            } else {
                None
            };
            let dangles = if !dangles.is_null() {
                Geometry::new_from_raw(dangles, self.clone_context(), "polygonize_full").ok()
            } else {
                None
            };
            let invalids = if !invalids.is_null() {
                Geometry::new_from_raw(invalids, self.clone_context(), "polygonize_full").ok()
            } else {
                None
            };
            Geometry::new_from_raw(ptr, self.clone_context(), "polygonize_full")
                  .map(|x| (x, cuts, dangles, invalids))
        }
    }

    pub fn shared_paths(&self, other: Geometry<'_>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSharedPaths_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "shared_paths")
        }
    }
}

unsafe impl<'a> Send for Geometry<'a> {}
unsafe impl<'a> Sync for Geometry<'a> {}

impl<'a> Drop for Geometry<'a> {
    fn drop(&mut self) {
        if !self.ptr.is_null() && self.owned {
            unsafe { GEOSGeom_destroy_r(self.get_raw_context(), self.as_raw()) }
        }
    }
}

impl<'a> Clone for Geometry<'a> {
    /// Also passes the context to the newly created `Geometry`.
    fn clone(&self) -> Geometry<'a> {
        let context = self.clone_context();
        let ptr = unsafe { GEOSGeom_clone_r(context.as_raw(), self.as_raw()) };
        if ptr.is_null() {
            panic!("Couldn't clone geometry...");
        }
        Geometry {
            ptr: PtrWrap(ptr),
            context,
            owned: true,
        }
    }
}

impl<'a> PartialEq for Geometry<'a> {
    fn eq<'b>(&self, other: &Geometry<'b>) -> bool {
        self.equals(other).unwrap_or_else(|_| false)
    }
}

impl<'a> ContextInteractions<'a> for Geometry<'a> {
    /// Set the context handle to the geometry.
    ///
    /// ```
    /// use geos::{ContextInteractions, ContextHandle, Geometry};
    ///
    /// let context_handle = ContextHandle::init().expect("invalid init");
    /// context_handle.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// let mut point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// point_geom.set_context_handle(context_handle);
    /// ```
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    /// Get the context handle of the geometry.
    ///
    /// ```
    /// use geos::{ContextInteractions, Geometry};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let context = point_geom.get_context_handle();
    /// context.set_notice_message_handler(Some(Box::new(|s| println!("new message: {}", s))));
    /// ```
    fn get_context_handle(&self) -> &ContextHandle<'a> {
        &self.context
    }
}

impl<'a> AsRaw for Geometry<'a> {
    type RawType = *mut GEOSGeometry;

    fn as_raw(&self) -> Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextHandling for Geometry<'a> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}
