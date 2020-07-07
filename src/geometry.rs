#[cfg(any(feature = "v3_6_0", feature = "dox"))]
use crate::Precision;
use crate::{
    AsRaw, ContextHandle, ContextHandling, ContextInteractions, CoordSeq, PreparedGeometry,
    WKTWriter,
};
use c_vec::CVec;
use context_handle::PtrWrap;
use enums::*;
use error::{Error, GResult, PredicateType};
use functions::*;
use geos_sys::*;
use std::borrow::Borrow;
use std::ffi::CString;
use std::sync::Arc;
use std::{self, str};

/// Representation of a GEOS geometry.
///
/// # Example
///
/// ```
/// use geos::Geometry;
///
/// let point_geom = Geometry::new_from_wkt("POINT (2.5 3.5)").expect("Invalid geometry");
/// assert_eq!(point_geom.get_x(), Ok(2.5));
/// assert_eq!(point_geom.get_y(), Ok(3.5));
/// ```
pub struct Geometry<'a> {
    pub(crate) ptr: PtrWrap<*mut GEOSGeometry>,
    context: Arc<ContextHandle<'a>>,
    owned: bool,
}

impl<'a> Geometry<'a> {
    /// Creates a `Geometry` from the WKT format.
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
            Ok(context_handle) => match CString::new(wkt) {
                Ok(c_str) => unsafe {
                    let reader = GEOSWKTReader_create_r(context_handle.as_raw());
                    let ptr = GEOSWKTReader_read_r(context_handle.as_raw(), reader, c_str.as_ptr());
                    GEOSWKTReader_destroy_r(context_handle.as_raw(), reader);
                    Geometry::new_from_raw(ptr, Arc::new(context_handle), "new_from_wkt")
                },
                Err(e) => Err(Error::GenericError(format!(
                    "Conversion to CString failed: {}",
                    e
                ))),
            },
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
    /// assert_eq!(point_geom.equals(&new_geom), Ok(true));
    /// ```
    pub fn new_from_hex(hex: &[u8]) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::new_from_hex")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeomFromHEX_buf_r(context.as_raw(), hex.as_ptr(), hex.len());
                Geometry::new_from_raw(ptr, Arc::new(context), "new_from_hex")
            },
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
    /// assert_eq!(point_geom.equals(&new_geom), Ok(true));
    /// ```
    pub fn new_from_wkb(wkb: &[u8]) -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::new_from_wkb")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeomFromWKB_buf_r(context.as_raw(), wkb.as_ptr(), wkb.len());
                Geometry::new_from_raw(ptr, Arc::new(context), "new_from_wkb")
            },
            Err(e) => Err(e),
        }
    }

    /// Converts a [`Geometry`] to the HEX format. For more control over the generated output,
    /// use the [`WKBWriter`] type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let hex_buf = point_geom.to_hex().expect("conversion to WKB failed");
    /// ```
    pub fn to_hex(&self) -> GResult<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToHEX_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                Err(Error::NoConstructionFromNullPtr(
                    "Geometry::to_hex failed: GEOSGeomToHEX_buf_r returned null pointer".to_owned(),
                ))
            } else {
                Ok(CVec::new(ptr, size as _))
            }
        }
    }

    /// Converts a [`Geometry`] to the WKB format. For more control over the generated output,
    /// use the [`WKBWriter`] type.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    /// let wkb_buf = point_geom.to_wkb().expect("conversion to WKB failed");
    /// ```
    pub fn to_wkb(&self) -> GResult<CVec<u8>> {
        let mut size = 0;
        unsafe {
            let ptr = GEOSGeomToWKB_buf_r(self.get_raw_context(), self.as_raw(), &mut size);
            if ptr.is_null() {
                Err(Error::NoConstructionFromNullPtr(
                    "Geometry::to_wkb failed: GEOSGeomToWKB_buf_r returned null pointer".to_owned(),
                ))
            } else {
                Ok(CVec::new(ptr, size as _))
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

    /// Creates an areal geometry formed by the constituent linework of given geometry.
    ///
    /// You can find new illustrations on [postgis](https://postgis.net/docs/ST_BuildArea.html)
    /// documentation.
    ///
    /// Available using the `v3_8_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT(100 90)").expect("Invalid geometry");
    /// let small_geom = geom.buffer(25., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(50., 8).expect("buffer failed");
    ///
    /// let union_geom = small_geom.union(&big_geom).expect("union failed");
    /// let build_area_geom = union_geom.build_area().expect("build_area failed");
    ///
    /// // Looks like a donut.
    /// assert_eq!(union_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((150.0 90.0, 149.0 80.2, 146.2 70.9, 141.6 62.2, 135.4 54.6, \
    ///                       127.8 48.4, 119.1 43.8, 109.8 41.0, 100.0 40.0, 90.2 41.0, \
    ///                       80.9 43.8, 72.2 48.4, 64.6 54.6, 58.4 62.2, 53.8 70.9, 51.0 80.2, \
    ///                       50.0 90.0, 51.0 99.8, 53.8 109.1, 58.4 117.8, 64.6 125.4, \
    ///                       72.2 131.6, 80.9 136.2, 90.2 139.0, 100.0 140.0, 109.8 139.0, \
    ///                       119.1 136.2, 127.8 131.6, 135.4 125.4, 141.6 117.8, 146.2 109.1, \
    ///                       149.0 99.8, 150.0 90.0))");
    /// ```
    #[cfg(any(feature = "v3_8_0", feature = "dox"))]
    pub fn build_area(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSBuildArea_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "build_area")
        }
    }

    /// Description from [postgis](https://postgis.net/docs/ST_Polygonize.html):
    ///
    /// > Creates a GeometryCollection containing possible polygons formed from the constituent
    /// > linework of a set of geometries.
    ///
    /// # Example:
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((-71.040878 42.285678,\
    ///                                              -71.040943 42.2856,\
    ///                                              -71.04096 42.285752,\
    ///                                              -71.040878 42.285678))")
    ///                      .expect("Failed to create geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((-71.17166 42.353675,\
    ///                                              -71.172026 42.354044,\
    ///                                              -71.17239 42.354358,\
    ///                                              -71.171794 42.354971,\
    ///                                              -71.170511 42.354855,\
    ///                                              -71.17112 42.354238,\
    ///                                              -71.17166 42.353675))")
    ///                      .expect("Failed to create geometry");
    ///
    /// let polygonized = Geometry::polygonize(&[geom1, geom2]).expect("polygonize failed");
    /// assert_eq!(polygonized.to_wkt().unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((-71.0408780000000064 42.2856779999999972, \
    ///                                           -71.0409429999999986 42.2856000000000023, \
    ///                                           -71.0409599999999983 42.2857520000000022, \
    ///                                           -71.0408780000000064 42.2856779999999972)), \
    ///                                 POLYGON ((-71.1716600000000028 42.3536750000000026, \
    ///                                           -71.1720260000000025 42.3540440000000018, \
    ///                                           -71.1723899999999929 42.3543579999999977, \
    ///                                           -71.1717940000000056 42.3549709999999990, \
    ///                                           -71.1705110000000047 42.3548550000000006, \
    ///                                           -71.1711200000000019 42.3542380000000023, \
    ///                                           -71.1716600000000028 42.3536750000000026)))");
    /// ```
    pub fn polygonize<T: Borrow<Geometry<'a>>>(geometries: &[T]) -> GResult<Geometry<'a>> {
        unsafe {
            let context = match geometries.get(0) {
                Some(g) => g.borrow().clone_context(),
                None => match ContextHandle::init_e(Some("Geometry::polygonize")) {
                    Ok(context) => Arc::new(context),
                    Err(e) => return Err(e),
                },
            };
            let geoms = geometries
                .iter()
                .map(|g| g.borrow().as_raw() as *const _)
                .collect::<Vec<_>>();
            let ptr = GEOSPolygonize_r(context.as_raw(), geoms.as_ptr(), geoms.len() as _);
            Geometry::new_from_raw(ptr, context, "polygonize")
        }
    }

    pub fn polygonizer_get_cut_edges<T: Borrow<Geometry<'a>>>(
        &self,
        geometries: &[T],
    ) -> GResult<Geometry<'a>> {
        unsafe {
            let context = match geometries.get(0) {
                Some(g) => g.borrow().clone_context(),
                None => match ContextHandle::init_e(Some("Geometry::polygonizer_get_cut_edges")) {
                    Ok(context) => Arc::new(context),
                    Err(e) => return Err(e),
                },
            };
            let geoms = geometries
                .iter()
                .map(|g| g.borrow().as_raw() as *const _)
                .collect::<Vec<_>>();
            let ptr =
                GEOSPolygonizer_getCutEdges_r(context.as_raw(), geoms.as_ptr(), geoms.len() as _);
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
    /// assert_eq!(lines_merged.to_wkt_precision(1).unwrap(),
    ///            "LINESTRING (-29.0 -27.0, -30.0 -29.7, -36.0 -31.0, -45.0 -33.0, -46.0 -32.0)");
    /// ```
    pub fn line_merge(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSLineMerge_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "line_merge")
        }
    }

    /// Reverses the order of the vertexes.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 10,1 2)").expect("invalid geometry");
    /// let reversed_line = line.reverse().expect("reverse failed");
    ///
    /// assert_eq!(reversed_line.to_wkt_precision(1).unwrap(), "LINESTRING (1.0 2.0, 1.0 10.0)");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn reverse(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSReverse_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "reverse")
        }
    }

    /// Returns a simplified version of the given geometry.
    pub fn simplify(&self, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSimplify_r(self.get_raw_context(), self.as_raw(), tolerance);
            Geometry::new_from_raw(ptr, self.clone_context(), "simplify")
        }
    }

    /// Returns a simplified version of the given geometry. It will avoid creating invalid derived
    /// geometries.
    pub fn topology_preserve_simplify(&self, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr =
                GEOSTopologyPreserveSimplify_r(self.get_raw_context(), self.as_raw(), tolerance);
            Geometry::new_from_raw(ptr, self.clone_context(), "topology_preserve_simplify")
        }
    }

    pub(crate) unsafe fn new_from_raw(
        ptr: *mut GEOSGeometry,
        context: Arc<ContextHandle<'a>>,
        caller: &str,
    ) -> GResult<Geometry<'a>> {
        if ptr.is_null() {
            let extra = if let Some(x) = context.get_last_error() {
                format!("\nLast error: {}", x)
            } else {
                String::new()
            };
            return Err(Error::NoConstructionFromNullPtr(format!(
                "Geometry::{}{}",
                caller, extra
            )));
        }
        Ok(Geometry {
            ptr: PtrWrap(ptr),
            context,
            owned: true,
        })
    }

    /// Checks if the geometry is valid.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert!(geom.is_valid() == false);
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
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_valid_reason(), Ok("Self-intersection[0 0]".to_owned()));
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
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 1 2, 1 1, 0 0))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.get_type(), Ok("Polygon".to_owned()));
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
    /// we could skip this clone with cleaner code.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT (2 3)").expect("Invalid geometry");
    /// let coord_seq = geom.get_coord_seq().expect("get_coord_seq failed");
    ///
    /// assert_eq!(coord_seq.get_x(0), Ok(2.));
    /// assert_eq!(coord_seq.get_y(0), Ok(3.));
    /// ```
    pub fn get_coord_seq(&self) -> GResult<CoordSeq<'a>> {
        let type_geom = self.geometry_type();
        match type_geom {
            GeometryTypes::Point | GeometryTypes::LineString | GeometryTypes::LinearRing => unsafe {
                let coord = GEOSGeom_getCoordSeq_r(self.get_raw_context(), self.as_raw());
                let t = GEOSCoordSeq_clone_r(self.get_raw_context(), coord);
                let mut size = 0;
                let mut dims = 0;

                if GEOSCoordSeq_getSize_r(self.get_raw_context(), coord, &mut size) == 0 {
                    return Err(Error::GenericError(
                        "GEOSCoordSeq_getSize_r failed".to_owned(),
                    ));
                }
                if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), coord, &mut dims) == 0 {
                    return Err(Error::GenericError(
                        "GEOSCoordSeq_getDimensions_r failed".to_owned(),
                    ));
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

    /// Returns the area of the geometry. Units are specified by the SRID of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))").expect("Invalid geometry");
    /// assert_eq!(geom1.area(), Ok(60.));
    /// ```
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
            managed_string(
                c_result,
                self.get_context_handle(),
                "GResult::to_wkt_precision",
            )
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

    /// Returns `true` if `self` shares any portion of space with `other`. So if any of this is
    /// `true`:
    ///
    /// * `self` overlaps `other`
    /// * `self` touches `other`
    /// * `self` is within `other`
    ///
    /// Then `intersects` will return `true` as well.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.intersects(&geom2), Ok(false));
    /// assert_eq!(geom1.intersects(&geom3), Ok(true));
    /// ```
    pub fn intersects<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSIntersects_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Intersects)
    }

    /// Returns `true` if `self` and `other` have at least one interior into each other.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(1 1,2 2)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 1,1 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.crosses(&geom2), Ok(true));
    /// ```
    pub fn crosses<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSCrosses_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Crosses)
    }

    /// Returns `true` if `self` doesn't:
    ///
    /// * Overlap `other`
    /// * Touch `other`
    /// * Is within `other`
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("invalid geometry");
    /// let geom3 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.disjoint(&geom2), Ok(true));
    /// assert_eq!(geom1.disjoint(&geom3), Ok(false));
    /// ```
    pub fn disjoint<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSDisjoint_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Disjoint)
    }

    /// Returns `true` if the only points in common between `self` and `other` lie in the union of
    /// the boundaries of `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(0 0, 1 1, 0 2)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT(1 1)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.touches(&geom2), Ok(false));
    ///
    /// let geom2 = Geometry::new_from_wkt("POINT(0 2)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.touches(&geom2), Ok(true));
    /// ```
    pub fn touches<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSTouches_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Touches)
    }

    /// Returns `true` if `self` spatially overlaps `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 0.5)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(1 0, 1 1, 3 5)").expect("invalid geometry");
    ///
    /// assert_eq!(geom1.overlaps(&geom2), Ok(false));
    ///
    /// let geom1 = geom1.buffer(3., 8).expect("buffer failed");
    /// let geom2 = geom2.buffer(0.5, 8).expect("buffer failed");
    ///
    /// assert_eq!(geom1.overlaps(&geom2), Ok(true));
    /// ```
    pub fn overlaps<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSOverlaps_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Overlaps)
    }

    /// Returns `true` if `self` is completely inside `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT(50 50)").expect("invalid geometry");
    /// let small_geom = geom.buffer(20., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(40., 8).expect("buffer failed");
    ///
    /// assert_eq!(small_geom.within(&small_geom), Ok(true));
    /// assert_eq!(small_geom.within(&big_geom), Ok(true));
    /// assert_eq!(big_geom.within(&small_geom), Ok(false));
    /// ```
    pub fn within<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSWithin_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
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
    pub fn equals<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSEquals_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Equals)
    }

    /// Checks if the two [`Geometry`] objects are exactly equal.
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
    /// assert_eq!(geom1.equals_exact(&geom2, 0.1), Ok(false));
    /// assert_eq!(geom1.equals_exact(&geom3, 0.1), Ok(true));
    /// ```
    pub fn equals_exact<'b>(&self, other: &Geometry<'b>, precision: f64) -> GResult<bool> {
        let ret_val = unsafe {
            GEOSEqualsExact_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                precision,
            )
        };
        check_geos_predicate(ret_val as _, PredicateType::EqualsExact)
    }

    /// Returns `true` if no point of `other` is outside of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// assert_eq!(little_geom.covers(&big_geom), Ok(false));
    /// assert_eq!(big_geom.covers(&little_geom), Ok(true));
    /// ```
    pub fn covers<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSCovers_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Covers)
    }

    /// Returns `true` if no point of `self` is outside of `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let little_geom = geom.buffer(10., 8).expect("buffer failed");
    /// let big_geom = geom.buffer(20., 8).expect("buffer failed");
    ///
    /// assert_eq!(little_geom.covered_by(&big_geom), Ok(true));
    /// assert_eq!(big_geom.covered_by(&little_geom), Ok(false));
    /// ```
    pub fn covered_by<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSCoveredBy_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::CoveredBy)
    }

    /// Returns `true` if no points of the `other` geometry is outside the exterior of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.contains(&geom2), Ok(true));
    /// ```
    pub fn contains<'b>(&self, other: &Geometry<'b>) -> GResult<bool> {
        let ret_val =
            unsafe { GEOSContains_r(self.get_raw_context(), self.as_raw(), other.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::Contains)
    }

    /// Returns a geometry which represents all points whose distance from `self` is less than or
    /// equal to distance.
    ///
    /// You can find nice examples about this in [postgis](https://postgis.net/docs/ST_Buffer.html)
    /// documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let buffer_geom = geom.buffer(50., 2).expect("buffer failed");
    ///
    /// assert_eq!(buffer_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((51.0 3.0, 36.4 -32.4, 1.0 -47.0, -34.4 -32.4, -49.0 3.0, -34.4 38.4, \
    ///                       1.0 53.0, 36.4 38.4, 51.0 3.0))");
    /// ```
    pub fn buffer(&self, width: f64, quadsegs: i32) -> GResult<Geometry<'a>> {
        assert!(quadsegs > 0);
        unsafe {
            let ptr = GEOSBuffer_r(self.get_raw_context(), self.as_raw(), width, quadsegs as _);
            Geometry::new_from_raw(ptr, self.clone_context(), "buffer")
        }
    }

    /// Returns `true` if the given geometry is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::create_empty_polygon().expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON EMPTY").expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// assert_eq!(geom.is_empty(), Ok(false));
    /// ```
    pub fn is_empty(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisEmpty_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsEmpty)
    }

    /// Returns true if the given geometry has no anomalous geometric points, such as self
    /// intersection or self tangency.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT (2.5 2.5)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_simple(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 1,2 2,2 3.5,1 3,1 2,2 1)")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_simple(), Ok(false));
    /// ```
    pub fn is_simple(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSisSimple_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    /// Returns a geometry which represents part of `self` that doesn't intersect with `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(50 100, 50 200)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(50 50, 50 150)").expect("Invalid geometry");
    ///
    /// let difference_geom = geom1.difference(&geom2).expect("envelope failed");
    ///
    /// assert_eq!(difference_geom.to_wkt_precision(1).unwrap(),
    ///            "LINESTRING (50.0 150.0, 50.0 200.0)");
    /// ```
    pub fn difference<'b>(&self, other: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSDifference_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "difference")
        }
    }

    /// Returns the minimum bouding box of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    /// let envelope_geom = geom.envelope().expect("envelope failed");
    ///
    /// assert_eq!(envelope_geom.to_wkt_precision(1).unwrap(), "POINT (1.0 3.0)");
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 1 3)").expect("Invalid geometry");
    /// let envelope_geom = geom.envelope().expect("envelope failed");
    ///
    /// assert_eq!(envelope_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((0.0 0.0, 1.0 0.0, 1.0 3.0, 0.0 3.0, 0.0 0.0))");
    /// ```
    pub fn envelope(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSEnvelope_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "envelope")
        }
    }

    /// Returns a geometry which represents the parts of `self` and `other` that don't intersect.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING(50 100, 50 200)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(50 50, 50 150)").expect("Invalid geometry");
    ///
    /// let sym_diff_geom = geom1.sym_difference(&geom2).expect("sym_difference failed");
    ///
    /// assert_eq!(sym_diff_geom.to_wkt_precision(1).unwrap(),
    ///            "MULTILINESTRING ((50.0 150.0, 50.0 200.0), (50.0 50.0, 50.0 100.0))");
    /// ```
    pub fn sym_difference<'b>(&self, other: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSymDifference_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "sym_difference")
        }
    }

    /// Aggregates the given geometry with another one.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT(1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT(3 4)").expect("Invalid geometry");
    ///
    /// let union_geom = geom1.union(&geom2).expect("union failed");
    ///
    /// assert_eq!(union_geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 2.0, 3.0 4.0)");
    /// ```
    pub fn union(&self, other: &Geometry<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSUnion_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "union")
        }
    }

    /// Returns the geometric center or (equivalently) the center of mass of the given geometry as
    /// a point.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("MULTIPOINT(-1 0, -1 2, -1 3, -1 4, -1 7, 0 1, 0 3, 1 1)")
    ///                     .expect("Invalid geometry");
    /// let centroid = geom.get_centroid().expect("failed to get centroid");
    ///
    /// assert_eq!(centroid.to_wkt_precision(1).unwrap(), "POINT (-0.5 2.6)");
    /// ```
    pub fn get_centroid(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGetCentroid_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_centroid")
        }
    }

    /// Creates an empty polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::create_empty_polygon().expect("Failed to build empty polygon");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "POLYGON EMPTY");
    /// ```
    pub fn create_empty_polygon() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_polygon")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeom_createEmptyPolygon_r(context.as_raw());
                Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_polygon")
            },
            Err(e) => Err(e),
        }
    }

    /// Creates an empty point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::create_empty_point().expect("Failed to build empty point");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "POINT EMPTY");
    /// ```
    pub fn create_empty_point() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_point")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeom_createEmptyPoint_r(context.as_raw());
                Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_point")
            },
            Err(e) => Err(e),
        }
    }

    /// Creates an empty line string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::create_empty_line_string().expect("Failed to build empty line string");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "LINESTRING EMPTY");
    /// ```
    pub fn create_empty_line_string() -> GResult<Geometry<'a>> {
        match ContextHandle::init_e(Some("Geometry::create_empty_line_string")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeom_createEmptyLineString_r(context.as_raw());
                Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_line_string")
            },
            Err(e) => Err(e),
        }
    }

    /// Creates an empty collection.
    ///
    /// The `type_` must be one of:
    ///
    /// * [`GeometryTypes::GeometryCollection`]
    /// * [`GeometryTypes::MultiPoint`]
    /// * [`GeometryTypes::MultiLineString`]
    /// * [`GeometryTypes::MultiPolygon`]
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, GeometryTypes};
    ///
    /// let geom = Geometry::create_empty_collection(GeometryTypes::MultiPolygon)
    ///                     .expect("Failed to build empty collection");
    ///
    /// assert_eq!(geom.to_wkt().unwrap(), "MULTIPOLYGON EMPTY");
    /// ```
    pub fn create_empty_collection(type_: GeometryTypes) -> GResult<Geometry<'a>> {
        match type_ {
            GeometryTypes::GeometryCollection
            | GeometryTypes::MultiPoint
            | GeometryTypes::MultiLineString
            | GeometryTypes::MultiPolygon => {}
            _ => return Err(Error::GenericError("Invalid geometry type".to_owned())),
        }
        match ContextHandle::init_e(Some("Geometry::create_empty_collection")) {
            Ok(context) => unsafe {
                let ptr = GEOSGeom_createEmptyCollection_r(context.as_raw(), type_.into());
                Geometry::new_from_raw(ptr, Arc::new(context), "create_empty_collection")
            },
            Err(e) => Err(e),
        }
    }

    /// Creates a polygon formed by the given shell and array of holes.
    ///
    /// ### Note
    ///
    /// `exterior` must be a `LinearRing`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINEARRING(75.15 29.53,77 29,77.6 29.5, 75.15 29.53)")
    ///                     .expect("Invalid geometry");
    /// let polygon_geom = Geometry::create_polygon(geom, vec![]).expect("create_polygon failed");
    ///
    /// assert_eq!(polygon_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((75.2 29.5, 77.0 29.0, 77.6 29.5, 75.2 29.5))");
    /// ```
    pub fn create_polygon<'b>(
        mut exterior: Geometry<'a>,
        mut interiors: Vec<Geometry<'b>>,
    ) -> GResult<Geometry<'a>> {
        if exterior.geometry_type() != GeometryTypes::LinearRing {
            return Err(Error::GenericError(
                "exterior must be a LinearRing".to_owned(),
            ));
        }
        let context_handle = exterior.clone_context();
        let nb_interiors = interiors.len();
        let res = unsafe {
            let mut geoms: Vec<*mut GEOSGeometry> =
                interiors.iter_mut().map(|g| g.as_raw()).collect();
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

    /// Create a geometry collection.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.0 4.0)").expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_geometry_collection(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "GEOMETRYCOLLECTION (POLYGON ((0.0 0.0, 10.0 0.0, 10.0 6.0, 0.0 6.0, 0.0 0.0)), \
    ///                                 POINT (3.0 4.0))");
    /// ```
    pub fn create_geometry_collection(geoms: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        create_multi_geom(geoms, GeometryTypes::GeometryCollection)
    }

    /// Create a multi polygon geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((3 3, 10 3, 10 6, 3 6, 3 3))")
    ///                      .expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipolygon(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "MULTIPOLYGON (((0.0 0.0, 10.0 0.0, 10.0 6.0, 0.0 6.0, 0.0 0.0)), \
    ///                           ((3.0 3.0, 10.0 3.0, 10.0 6.0, 3.0 6.0, 3.0 3.0)))");
    /// ```
    pub fn create_multipolygon(polygons: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&polygons, GeometryTypes::Polygon) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Polygon".to_owned(),
            ));
        }
        create_multi_geom(polygons, GeometryTypes::MultiPolygon)
    }

    /// Create a multiline string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (1.0 2.0, 3.0 4.0)").expect("invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (5.0 6.0, 7.0 8.0)").expect("invalid geometry");
    ///
    /// let geom = Geometry::create_multiline_string(vec![geom1, geom2])
    ///                     .expect("Failed to build multiline string");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "MULTILINESTRING ((1.0 2.0, 3.0 4.0), (5.0 6.0, 7.0 8.0))");
    /// ```
    pub fn create_multiline_string(linestrings: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&linestrings, GeometryTypes::LineString) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type LineString".to_owned(),
            ));
        }
        create_multi_geom(linestrings, GeometryTypes::MultiLineString)
    }

    /// Creates a multi point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1.0 2.0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (3.0 4.0)").expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipoint(vec![geom1, geom2])
    ///                     .expect("Failed to build multipoint");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 2.0, 3.0 4.0)");
    /// ```
    pub fn create_multipoint(points: Vec<Geometry<'a>>) -> GResult<Geometry<'a>> {
        if !check_same_geometry_type(&points, GeometryTypes::Point) {
            return Err(Error::ImpossibleOperation(
                "all the provided geometry have to be of type Point".to_owned(),
            ));
        }
        create_multi_geom(points, GeometryTypes::MultiPoint)
    }

    /// Creates a point geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_point(coords).expect("Failed to create a point");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "POINT (1.0 2.0)");
    /// ```
    pub fn create_point(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createPoint_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_point");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    /// Creates a line string geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[1., 2.], &[3., 4.]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_line_string(coords).expect("Failed to create a line string");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(), "LINESTRING (1.0 2.0, 3.0 4.0)");
    /// ```
    pub fn create_line_string(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLineString_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_line_string");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    /// Creates a linear ring geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{CoordDimensions, CoordSeq, Geometry};
    ///
    /// let coords = CoordSeq::new_from_vec(&[&[75.15, 29.53],
    ///                                       &[77., 29.],
    ///                                       &[77.6, 29.5],
    ///                                       &[75.15, 29.53]])
    ///                       .expect("failed to create CoordSeq");
    ///
    /// let geom = Geometry::create_linear_ring(coords).expect("Failed to create a linea ring");
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "LINEARRING (75.2 29.5, 77.0 29.0, 77.6 29.5, 75.2 29.5)");
    /// ```
    pub fn create_linear_ring(mut s: CoordSeq<'a>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_createLinearRing_r(s.get_raw_context(), s.as_raw());
            let res = Geometry::new_from_raw(ptr, s.clone_context(), "create_linear_ring");
            s.ptr = PtrWrap(::std::ptr::null_mut());
            res
        }
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_UnaryUnion.html):
    ///
    /// > Unlike ST_Union, ST_UnaryUnion does dissolve boundaries between components of a
    /// > multipolygon (invalid) and does perform union between the components of a
    /// > geometrycollection. Each components of the input geometry is assumed to be valid, so you
    /// > won't get a valid multipolygon out of a bow-tie polygon (invalid).
    /// >
    /// > You may use this function to node a set of linestrings. You may mix ST_UnaryUnion with
    /// > ST_Collect to fine-tune how many geometries at once you want to dissolve to be nice on
    /// > both memory size and CPU time, finding the balance between ST_Union and ST_MemUnion.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POLYGON((1 1, 2 1, 2 5, 1 5, 1 1))")
    ///                      .expect("Invalid geometry");
    ///
    /// let geom = Geometry::create_multipolygon(vec![geom1, geom2])
    ///                     .expect("Failed to build multipolygon");
    ///
    /// let union_geom = geom.unary_union().expect("unary_union failed");
    ///
    /// assert_eq!(union_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((0.0 0.0, 0.0 6.0, 10.0 6.0, 10.0 0.0, 0.0 0.0))");
    /// ```
    pub fn unary_union(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSUnaryUnion_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "unary_union")
        }
    }

    /// Create a voronoi diagram.
    ///
    /// # Example
    ///
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let input = Geometry::new_from_wkt("MULTIPOINT((100 200), (105 202), (110 200), (140 230),
    ///                                                (210 240), (220 190), (170 170), (170 260),
    ///                                                (213 245), (220 190))")
    ///                   .expect("Invalid geometry");
    /// let mut expected = Geometry::new_from_wkt(
    ///     "GEOMETRYCOLLECTION(POLYGON ((-20 50, -20 380, -3.75 380, 105 235, 105 115, 77.14285714285714 50, -20 50)),\
    ///                          POLYGON ((247 50, 77.14285714285714 50, 105 115, 145 195, 178.33333333333334 211.66666666666666, 183.51851851851853 208.7037037037037, 247 50)),\
    ///                          POLYGON ((-3.75 380, 20.000000000000007 380, 176.66666666666666 223.33333333333334, 178.33333333333334 211.66666666666666, 145 195, 105 235, -3.75 380)),\
    ///                          POLYGON ((105 115, 105 235, 145 195, 105 115)),\
    ///                          POLYGON ((20.000000000000007 380, 255 380, 176.66666666666666 223.33333333333334, 20.000000000000007 380)),\
    ///                          POLYGON ((255 380, 340 380, 340 240, 183.51851851851853 208.7037037037037, 178.33333333333334 211.66666666666666, 176.66666666666666 223.33333333333334, 255 380)),\
    ///                          POLYGON ((340 240, 340 50, 247 50, 183.51851851851853 208.7037037037037, 340 240)))")
    ///                             .expect("Invalid geometry");
    ///
    /// let mut voronoi = input.voronoi(None, 6., false).expect("voronoi failed");
    ///
    /// expected.normalize().expect("normalize failed");
    /// voronoi.normalize().expect("normalize failed");
    ///
    /// assert_eq!(expected.equals(&voronoi), Ok(true));
    /// ```
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
                envelope.map(|e| e.as_raw()).unwrap_or(std::ptr::null_mut()),
                tolerance,
                only_edges as _,
            );
            Self::new_from_raw(raw_voronoi, self.clone_context(), "voronoi")
        }
    }

    /// Normalizes `self` in its normalized/canonical form. May reorder vertices in polygon rings,
    /// rings in a polygon, elements in a multi-geometry complex.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let mut geom = Geometry::new_from_wkt("GEOMETRYCOLLECTION(POINT(2 3),
    ///                                                           MULTILINESTRING((0 0, 1 1),(2 2, 3 3)))")
    ///                         .expect("Invalid geometry");
    ///
    /// geom.normalize();
    ///
    /// assert_eq!(geom.to_wkt_precision(1).unwrap(),
    ///            "GEOMETRYCOLLECTION (MULTILINESTRING ((2.0 2.0, 3.0 3.0), (0.0 0.0, 1.0 1.0)), \
    ///                                 POINT (2.0 3.0))");
    /// ```
    pub fn normalize(&mut self) -> GResult<bool> {
        let ret_val = unsafe { GEOSNormalize_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val, PredicateType::Normalize)
    }

    /// Returns a geometry representing the intersection between `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let mut geom1 = Geometry::new_from_wkt("POINT(0 0)").expect("Invalid geometry");
    /// let mut geom2 = Geometry::new_from_wkt("LINESTRING(2 0, 0 2)").expect("Invalid geometry");
    ///
    /// let intersection_geom = geom1.intersection(&geom2).expect("intersection failed");
    ///
    /// // No intersection.
    /// assert_eq!(intersection_geom.is_empty(), Ok(true));
    ///
    /// // We slighty change the linestring so we have an intersection:
    /// let mut geom2 = Geometry::new_from_wkt("LINESTRING(0 0, 0 2)").expect("Invalid geometry");
    ///
    /// let intersection_geom = geom1.intersection(&geom2).expect("intersection failed");
    ///
    /// // Intersection!
    /// assert_eq!(intersection_geom.to_wkt_precision(1).unwrap(), "POINT (0.0 0.0)");
    /// ```
    pub fn intersection<'b>(&self, other: &Geometry<'b>) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSIntersection_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "intersection")
        }
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_ConvexHull.html):
    ///
    /// > The convex hull of a geometry represents the minimum convex geometry that encloses all
    /// > geometries within the set.
    /// >
    /// > One can think of the convex hull as the geometry you get by wrapping an elastic band
    /// > around a set of geometries. This is different from a concave hull which is analogous to
    /// > shrink-wrapping your geometries.
    /// >
    /// > It is usually used with MULTI and Geometry Collections. Although it is not an aggregate -
    /// > you can use it in conjunction with ST_Collect to get the convex hull of a set of points.
    /// > ST_ConvexHull(ST_Collect(somepointfield)).
    /// >
    /// > It is often used to determine an affected area based on a set of point observations.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let mut geom1 = Geometry::new_from_wkt("MULTILINESTRING((100 190,10 8),
    ///                                                         (150 10, 20 30))")
    ///                          .expect("Invalid geometry");
    /// let mut geom2 = Geometry::new_from_wkt("MULTIPOINT(50 5, 150 30, 50 10, 10 10)")
    ///                          .expect("Invalid geometry");
    ///
    /// let geom = geom1.union(&geom2).expect("union failed");
    /// let convex_hull_geom = geom.convex_hull().expect("convex_hull failed");
    ///
    /// assert_eq!(convex_hull_geom.to_wkt_precision(1).unwrap(),
    ///            "POLYGON ((50.0 5.0, 10.0 8.0, 10.0 10.0, 100.0 190.0, 150.0 30.0, 150.0 10.0, 50.0 5.0))");
    /// ```
    pub fn convex_hull(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSConvexHull_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "convex_hull")
        }
    }

    /// Returns the closure of the combinatorial boundary of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 1,0 0, -1 1)").expect("Invalid geometry");
    /// let boundary_geom = geom.boundary().expect("boundary failed");
    ///
    /// assert_eq!(boundary_geom.to_wkt_precision(1).unwrap(), "MULTIPOINT (1.0 1.0, -1.0 1.0)");
    /// ```
    pub fn boundary(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSBoundary_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "boundary")
        }
    }

    /// Returns `true` if `self` has a Z coordinate.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2 3)").expect("Invalid geometry");
    /// assert_eq!(geom.has_z(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("POINT(1 2)").expect("Invalid geometry");
    /// assert_eq!(geom.has_z(), Ok(false));
    /// ```
    pub fn has_z(&self) -> GResult<bool> {
        let ret_val = unsafe { GEOSHasZ_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    /// Returns `true` if start and end point are coincident.
    ///
    /// Only works on `LineString` and `MultiLineString`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 1 1)").expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(false));
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(0 0, 0 1, 1 1, 0 0)").expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(true));
    ///
    /// let geom = Geometry::new_from_wkt("MULTILINESTRING((0 0, 0 1, 1 1, 0 0),(0 0, 1 1))")
    ///                     .expect("Invalid geometry");
    /// assert_eq!(geom.is_closed(), Ok(false));
    /// ```
    pub fn is_closed(&self) -> GResult<bool> {
        if self.geometry_type() != GeometryTypes::LineString
            && self.geometry_type() != GeometryTypes::MultiLineString
        {
            return Err(Error::GenericError(
                "Geometry must be a LineString or a MultiLineString".to_owned(),
            ));
        }
        let ret_val = unsafe { GEOSisClosed_r(self.get_raw_context(), self.as_raw()) };
        check_geos_predicate(ret_val as _, PredicateType::IsSimple)
    }

    /// Returns the length of `self`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(743238 2967416,743238 2967450)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.length().map(|x| format!("{:.2}", x)).unwrap(), "34.00");
    /// ```
    pub fn length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    /// Returns the distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    pub fn distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance,
            );
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    /// Returns the indexed distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.distance_indexed(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn distance_indexed<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        unsafe {
            let mut distance = 0.;
            if GEOSDistanceIndexed_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance,
            ) != 1
            {
                Err(Error::GenericError(
                    "GEOSDistanceIndexed_r failed".to_owned(),
                ))
            } else {
                Ok(distance)
            }
        }
    }

    /// Returns the hausdorff distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.hausdorff_distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "1.00");
    /// ```
    pub fn hausdorff_distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance,
            );
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    /// Returns the hausdorff distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POINT (1 2)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("POINT (2 2)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.hausdorff_distance_densify(&geom2, 1.).map(|x| format!("{:.2}", x))
    ///                                                        .unwrap(), "1.00");
    /// ```
    pub fn hausdorff_distance_densify<'b>(
        &self,
        other: &Geometry<'b>,
        distance_frac: f64,
    ) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSHausdorffDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance,
            );
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    /// Returns the frechet distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (0 0, 100 0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (0 0, 50 50, 100 0)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.frechet_distance(&geom2).map(|x| format!("{:.2}", x)).unwrap(), "70.71");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn frechet_distance<'b>(&self, other: &Geometry<'b>) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistance_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                &mut distance,
            );
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    /// Returns the frechet distance between `self` and `other`. The unit depends of the SRID.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("LINESTRING (0 0, 100 0)").expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING (0 0, 50 50, 100 0)").expect("Invalid geometry");
    ///
    /// assert_eq!(geom1.frechet_distance_densify(&geom2, 1.).map(|x| format!("{:.2}", x))
    ///                                                      .unwrap(), "70.71");
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
    pub fn frechet_distance_densify<'b>(
        &self,
        other: &Geometry<'b>,
        distance_frac: f64,
    ) -> GResult<f64> {
        let mut distance = 0.;
        unsafe {
            let ret = GEOSFrechetDistanceDensify_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                distance_frac,
                &mut distance,
            );
            check_ret(ret, PredicateType::IsSimple).map(|_| distance)
        }
    }

    /// Returns the length of the given geometry.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4, 5 6)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_length().map(|x| format!("{:.2}", x)).unwrap(), "5.66");
    /// ```
    pub fn get_length(&self) -> GResult<f64> {
        let mut length = 0.;
        unsafe {
            let ret = GEOSGeomGetLength_r(self.get_raw_context(), self.as_raw(), &mut length);
            check_ret(ret, PredicateType::IsSimple).map(|_| length)
        }
    }

    /// Documentation from [postgis](https://postgis.net/docs/ST_Snap.html):
    ///
    /// >  Snaps the vertices and segments of a geometry another Geometry's vertices. A snap
    /// > distance tolerance is used to control where snapping is performed. The result geometry is
    /// > the input geometry with the vertices snapped. If no snapping occurs then the input
    /// > geometry is returned unchanged.
    /// >
    /// > Snapping one geometry to another can improve robustness for overlay operations by
    /// > eliminating nearly-coincident edges (which cause problems during noding and intersection
    /// > calculation).
    /// >
    /// > Too much snapping can result in invalid topology being created, so the number and location
    /// > of snapped vertices is decided using heuristics to determine when it is safe to snap. This
    /// > can result in some potential snaps being omitted, however.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("MULTIPOLYGON(((26 125, 26 200, 126 200, 126 125, 26 125),
    ///                                                   (51 150, 101 150, 76 175, 51 150)),
    ///                                                  ((151 100, 151 200, 176 175, 151 100)))")
    ///                      .expect("Invalid geometry");
    /// let geom2 = Geometry::new_from_wkt("LINESTRING(5 107, 54 84, 101 100)")
    ///                      .expect("Invalid geometry");
    ///
    /// let distance = geom1.distance(&geom2).expect("distance failed");
    /// let snap_geom = geom1.snap(&geom2, distance * 1.25).expect("snap failed");
    ///
    /// assert_eq!(snap_geom.to_wkt_precision(1).unwrap(),
    ///            "MULTIPOLYGON (((5.0 107.0, 26.0 200.0, 126.0 200.0, 126.0 125.0, 101.0 100.0, 54.0 84.0, 5.0 107.0), \
    ///                            (51.0 150.0, 101.0 150.0, 76.0 175.0, 51.0 150.0)), \
    ///                           ((151.0 100.0, 151.0 200.0, 176.0 175.0, 151.0 100.0)))");
    /// ```
    pub fn snap<'b>(&self, other: &Geometry<'b>, tolerance: f64) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSSnap_r(
                self.get_raw_context(),
                self.as_raw(),
                other.as_raw(),
                tolerance,
            );
            Geometry::new_from_raw(ptr, self.clone_context(), "snap")
        }
    }

    /// Returns unique points of `self`.
    pub fn extract_unique_points(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_extractUniquePoints_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "extract_unique_points")
        }
    }

    pub fn nearest_points<'b>(&self, other: &Geometry<'b>) -> GResult<CoordSeq<'a>> {
        unsafe {
            let ptr = GEOSNearestPoints_r(self.get_raw_context(), self.as_raw(), other.as_raw());
            let mut size = 0;
            let mut dims = 0;

            if GEOSCoordSeq_getSize_r(self.get_raw_context(), ptr, &mut size) == 0 {
                return Err(Error::GenericError(
                    "GEOSCoordSeq_getSize_r failed".to_owned(),
                ));
            }
            if GEOSCoordSeq_getDimensions_r(self.get_raw_context(), ptr, &mut dims) == 0 {
                return Err(Error::GenericError(
                    "GEOSCoordSeq_getDimensions_r failed".to_owned(),
                ));
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
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert!(point_geom.get_z() == Ok(4.0));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
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

    /// Returns the nth point of the given geometry.
    ///
    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4, 5 6)")
    ///                     .expect("Invalid geometry");
    /// let nth_point = geom.get_point_n(1).expect("get_point_n failed");
    ///
    /// assert_eq!(nth_point.to_wkt_precision(1).unwrap(), "POINT (3.0 4.0)");
    /// ```
    pub fn get_point_n(&self, n: usize) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSGeomGetPointN_r(self.get_raw_context(), self.as_raw(), n as _);
            Geometry::new_from_raw(ptr, self.clone_context(), "get_point_n")
        }
    }

    /// Returns the start point of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    /// let start_point = geom.get_start_point().expect("get_start_point failed");
    ///
    /// assert_eq!(start_point.to_wkt_precision(1).unwrap(), "POINT (1.0 2.0)");
    /// ```
    pub fn get_start_point(&self) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSGeomGetStartPoint_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_start_point")
        }
    }

    /// Returns the end point of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    /// let end_point = geom.get_end_point().expect("get_end_point failed");
    ///
    /// assert_eq!(end_point.to_wkt_precision(1).unwrap(), "POINT (3.0 4.0)");
    /// ```
    pub fn get_end_point(&self) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSGeomGetEndPoint_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "get_end_point")
        }
    }

    /// Returns the number of points of `self`.
    ///
    /// The given `Geometry` must be a `LineString`, otherwise it'll fail.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING (1 2, 3 4)")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_points(), Ok(2));
    /// ```
    pub fn get_num_points(&self) -> GResult<usize> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ret = GEOSGeomGetNumPoints_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError(
                    "GEOSGeomGetNumPoints_r failed".to_owned(),
                ))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Returns the number of interior rings.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                            (1 1, 2 1, 2 5, 1 5, 1 1),\
    ///                                            (8 5, 8 4, 9 4, 9 5, 8 5))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_interior_rings(), Ok(2));
    /// ```
    pub fn get_num_interior_rings(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetNumInteriorRings_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError(
                    "GEOSGetNumInteriorRings_r failed".to_owned(),
                ))
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
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0),\
    ///                                            (1 1, 2 1, 2 5, 1 5, 1 1),\
    ///                                            (8 5, 8 4, 9 4, 9 5, 8 5))")
    ///                     .expect("Invalid geometry");
    /// let interior = geom.get_interior_ring_n(0).expect("failed to get interior ring");
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

    /// Returns the number of coordinates inside `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_coordinates(), Ok(5));
    /// ```
    pub fn get_num_coordinates(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGetNumCoordinates_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError(
                    "GEOSGetNumCoordinates_r failed".to_owned(),
                ))
            } else {
                Ok(ret as _)
            }
        }
    }

    /// Returns the number of dimensions used in `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
    ///                     .expect("Invalid geometry");
    ///
    /// assert_eq!(geom.get_num_dimensions(), Ok(2));
    /// ```
    pub fn get_num_dimensions(&self) -> GResult<usize> {
        unsafe {
            let ret = GEOSGeom_getDimensions_r(self.get_raw_context(), self.as_raw());
            if ret == -1 {
                Err(Error::GenericError(
                    "GEOSGeom_getDimensions_r failed".to_owned(),
                ))
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
                Err(Error::GenericError(
                    "GEOSGeom_getCoordinateDimension_r failed".to_owned(),
                ))
            } else {
                Ok(Dimensions::from(ret))
            }
        }
    }

    /// This functions attempts to return a valid representation of `self`.
    ///
    /// Available using the `v3_8_0` feature.
    #[cfg(any(feature = "v3_8_0", feature = "dox"))]
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
                Err(Error::GenericError(
                    "GEOSGetNumGeometries_r failed".to_owned(),
                ))
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

    /// Get SRID of `self`.
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

    /// Set SRID of `self`.
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
    pub fn set_srid(&self, srid: usize) -> GResult<()> {
        unsafe {
            if GEOSSetSRID_r(self.get_raw_context(), self.as_raw(), srid as _) == 0 {
                Err(Error::GenericError("GEOSSetSRID_r failed".to_owned()))
            } else {
                Ok(())
            }
        }
    }

    /// Returns the precision of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    /// assert_eq!(point_geom.get_precision().map(|x| format!("{:.2}", x)).unwrap(), "0.00");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn get_precision(&self) -> GResult<f64> {
        unsafe {
            let ret = GEOSGeom_getPrecision_r(self.get_raw_context(), self.as_raw());
            if ret == -1. {
                Err(Error::GenericError(
                    "GEOSGeom_getPrecision_r failed".to_owned(),
                ))
            } else {
                Ok(ret)
            }
        }
    }

    /// Returns the precision of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::{Geometry, Precision};
    ///
    /// let point_geom = Geometry::new_from_wkt("POINT (2.5 2.5 4.0)").expect("Invalid geometry");
    ///
    /// point_geom.set_precision(1., Precision::KeepCollapsed);
    /// assert_eq!(point_geom.get_precision().map(|x| format!("{:.2}", x)).unwrap(), "0.00");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn set_precision(&self, grid_size: f64, flags: Precision) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSGeom_setPrecision_r(
                self.get_raw_context(),
                self.as_raw(),
                grid_size,
                flags.into(),
            );
            Geometry::new_from_raw(ptr, self.clone_context(), "set_precision")
        }
    }

    /// Returns the biggest X of the geometry.
    ///
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_max(), Ok(5.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
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
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_x_min(), Ok(1.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
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
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_max(), Ok(6.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
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
    /// Available using the `v3_7_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let line = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(line.get_y_min(), Ok(3.));
    /// ```
    #[cfg(any(feature = "v3_7_0", feature = "dox"))]
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

    /// Returns the smallest distance by which a vertex of `self` could be moved to produce an
    /// invalid geometry.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("LINESTRING(1 3 4, 5 6 7)").expect("Invalid WKT");
    /// assert_eq!(geom.minimum_clearance().map(|x| format!("{:.8}", x)).unwrap(), "5.00000000");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn minimum_clearance(&self) -> GResult<f64> {
        unsafe {
            let mut value = 0.;
            if GEOSMinimumClearance_r(self.get_raw_context(), self.as_raw(), &mut value) != 0 {
                Err(Error::GenericError(
                    "GEOSMinimumClearance_r failed".to_owned(),
                ))
            } else {
                Ok(value)
            }
        }
    }

    /// Returns the two-point LineString spanning of `self`'s minimum clearance.
    ///
    /// Available using the `v3_6_0` feature.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom = Geometry::new_from_wkt("POLYGON ((0 0, 1 0, 1 1, 0.5 3.2e-4, 0 0))")
    ///                     .expect("Invalid WKT");
    /// let line = geom.minimum_clearance_line().expect("minimum_clearance_line failed");
    /// assert_eq!(line.to_wkt_precision(1).unwrap(), "LINESTRING (0.5 0.0, 0.5 0.0)");
    /// ```
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn minimum_clearance_line(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumClearanceLine_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_clearance_line")
        }
    }

    /// Returns the minimum rotated rectangle inside of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn minimum_rotated_rectangle(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumRotatedRectangle_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_rotated_rectangle")
        }
    }

    /// Returns the minimum width inside of `self`.
    ///
    /// Available using the `v3_6_0` feature.
    #[cfg(any(feature = "v3_6_0", feature = "dox"))]
    pub fn minimum_width(&self) -> GResult<Geometry<'a>> {
        unsafe {
            let ptr = GEOSMinimumWidth_r(self.get_raw_context(), self.as_raw());
            Geometry::new_from_raw(ptr, self.clone_context(), "minimum_width")
        }
    }

    /// Returns a [delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation)
    /// around the vertices of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use geos::Geometry;
    ///
    /// let geom1 = Geometry::new_from_wkt("POLYGON((175 150, 20 40, 50 60, 125 100, 175 150))")
    ///                      .expect("Invalid WKT");
    /// let geom2 = Geometry::new_from_wkt("POINT(110 170)").expect("Invalid WKT");
    /// let geom2 = geom2.buffer(20., 8).expect("buffer failed");
    ///
    /// let geom = geom1.union(&geom2).expect("union failed");
    ///
    /// let final_geom = geom.delaunay_triangulation(0.001, false).expect("delaunay_triangulation failed");
    /// ```
    pub fn delaunay_triangulation(
        &self,
        tolerance: f64,
        only_edges: bool,
    ) -> GResult<Geometry<'a>> {
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
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSInterpolate_r(self.get_raw_context(), self.as_raw(), d);
            Geometry::new_from_raw(ptr, self.clone_context(), "interpolate")
        }
    }

    pub fn interpolate_normalized(&self, d: f64) -> GResult<Geometry<'a>> {
        if self.geometry_type() != GeometryTypes::LineString {
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSInterpolateNormalized_r(self.get_raw_context(), self.as_raw(), d);
            Geometry::new_from_raw(ptr, self.clone_context(), "interpolate_normalized")
        }
    }

    pub fn project(&self, p: &Geometry<'_>) -> GResult<f64> {
        if p.geometry_type() != GeometryTypes::Point {
            return Err(Error::GenericError(
                "Second geometry must be a Point".to_owned(),
            ));
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
            return Err(Error::GenericError(
                "Second geometry must be a Point".to_owned(),
            ));
        }
        unsafe {
            let ret = GEOSProjectNormalized_r(self.get_raw_context(), self.as_raw(), p.as_raw());
            if ret == -1. {
                Err(Error::GenericError(
                    "GEOSProjectNormalized_r failed".to_owned(),
                ))
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
            return Err(Error::GenericError(
                "Geometry must be a LineString".to_owned(),
            ));
        }
        unsafe {
            let ptr = GEOSOffsetCurve_r(
                self.get_raw_context(),
                self.as_raw(),
                width,
                quadrant_segments,
                join_style.into(),
                mitre_limit,
            );
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
    ) -> GResult<(
        Geometry<'a>,
        Option<Geometry<'a>>,
        Option<Geometry<'a>>,
        Option<Geometry<'a>>,
    )> {
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
