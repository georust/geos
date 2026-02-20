use crate::enums::GeometryTypes;
use crate::{Geom, Geometry, PreparedGeometry};

#[test]
fn test_relationship() {
    let pt_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").unwrap();
    let line_geom = Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap();
    let polygon_geom = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();

    assert!(polygon_geom.covers(&pt_geom).unwrap());
    assert!(polygon_geom.intersects(&pt_geom).unwrap());
    assert!(!polygon_geom.covered_by(&pt_geom).unwrap());
    assert!(!polygon_geom.equals(&pt_geom).unwrap());
    assert!(!polygon_geom.within(&pt_geom).unwrap());

    assert!(!pt_geom.covers(&polygon_geom).unwrap());
    assert!(pt_geom.intersects(&polygon_geom).unwrap());
    assert!(pt_geom.covered_by(&polygon_geom).unwrap());
    assert!(!pt_geom.equals(&polygon_geom).unwrap());
    assert!(pt_geom.within(&polygon_geom).unwrap());

    assert!(!line_geom.covers(&pt_geom).unwrap());
    assert!(!line_geom.intersects(&pt_geom).unwrap());
    assert!(!line_geom.covered_by(&pt_geom).unwrap());
    assert!(!pt_geom.covered_by(&line_geom).unwrap());
    assert!(line_geom.intersects(&polygon_geom).unwrap());
    assert!(line_geom.crosses(&polygon_geom).unwrap());
    assert!(!line_geom.equals(&pt_geom).unwrap());
}

#[test]
fn test_geom_creation_from_geoms() {
    let polygon_geom = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
    let new_geom = polygon_geom.buffer(100.0, 12).expect("buffer failed");
    let g1 = new_geom
        .difference(&polygon_geom)
        .expect("difference failed");
    let g2 = polygon_geom
        .sym_difference(&new_geom)
        .expect("sym difference failed");
    let g3 = new_geom
        .sym_difference(&polygon_geom)
        .expect("sym difference 2 faileed");
    assert_almost_eq(
        g1.area().expect("area 1.1 failed"),
        g2.area().expect("area 1.2 failed"),
    );
    assert_almost_eq(
        g2.area().expect("area 2.1 failed"),
        g3.area().expect("area 2.2 failed"),
    );
    let g4 = g3.get_centroid().expect("get_centroid failed");
    assert_eq!(
        GeometryTypes::Point,
        g4.geometry_type().expect("geometry_type failed")
    );
    let g5 = g4.buffer(200.0, 12).expect("buffer 2 failed");

    assert!(g5.area().expect("area 3.1 failed") > g4.area().expect("area 3.2 failed"));
    assert_eq!(
        GeometryTypes::Polygon,
        g5.geometry_type().expect("geometry_type failed")
    );
}

#[test]
fn test_prepared_geoms() {
    let g1 = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
    let g2 = Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))").unwrap();
    let pg1 = PreparedGeometry::new(&g1).unwrap();
    assert!(pg1.intersects(&g2).unwrap());
    assert!(pg1.contains(&g2.get_centroid().unwrap()).unwrap());
    let vec_geoms = vec![
        Geometry::new_from_wkt("POINT (1.3 2.4)").unwrap(),
        Geometry::new_from_wkt("POINT (2.1 0.3)").unwrap(),
        Geometry::new_from_wkt("POINT (3.1 4.7)").unwrap(),
        Geometry::new_from_wkt("POINT (0.4 4.1)").unwrap(),
    ];
    for geom in &vec_geoms {
        assert!(pg1.intersects(geom).unwrap());
    }
}

#[test]
fn test_wkt_rounding_precision() {
    let g = Geometry::new_from_wkt("LINESTRING(0.0 0.0, 7.1 7.2, 5.0 5.6, 9.0 9.0)").unwrap();
    let wkt = g.to_wkt_precision(0);
    assert_eq!(wkt, Ok("LINESTRING (0 0, 7 7, 5 6, 9 9)".to_owned()));
    let wkt2 = g.to_wkt();
    assert!(wkt2 != wkt);
}

#[test]
fn test_multipoint_from_vec_single() {
    let vec_geoms = vec![
        Geometry::new_from_wkt("POINT (1.3 2.4)").unwrap(),
        Geometry::new_from_wkt("POINT (2.1 0.3)").unwrap(),
        Geometry::new_from_wkt("POINT (3.1 4.7)").unwrap(),
        Geometry::new_from_wkt("POINT (0.4 4.1)").unwrap(),
    ];
    let multi_point = Geometry::create_multipoint(vec_geoms).unwrap();
    #[cfg(not(feature = "v3_12_0"))]
    let expected = "MULTIPOINT (1.3 2.4, 2.1 0.3, 3.1 4.7, 0.4 4.1)";
    #[cfg(feature = "v3_12_0")]
    let expected = "MULTIPOINT ((1.3 2.4), (2.1 0.3), (3.1 4.7), (0.4 4.1))";
    assert_eq!(multi_point.to_wkt(), Ok(expected.to_owned()));
}

#[test]
fn test_multilinestring_from_vec_single() {
    let vec_geoms = vec![
        Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap(),
        Geometry::new_from_wkt("LINESTRING (0 0, 7 7, 45 50, 100 100)").unwrap(),
    ];
    let multi_linestring = Geometry::create_multiline_string(vec_geoms).unwrap();
    assert_eq!(
        multi_linestring.to_wkt(),
        Ok("MULTILINESTRING ((1 1, 10 50, 20 25), (0 0, 7 7, 45 50, 100 100))".to_owned()),
    );
}

#[test]
fn test_multipolygon_from_vec_single() {
    let vec_geoms = vec![
        Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap(),
        Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))").unwrap(),
    ];
    let multi_polygon = Geometry::create_multipolygon(vec_geoms).unwrap();
    assert_eq!(
        multi_polygon.to_wkt(),
        Ok("MULTIPOLYGON (((0 0, 0 5, 5 5, 5 0, 0 0)), ((1 1, 1 3, 5 5, 5 0, 1 1)))".to_owned()),
    );
}

#[test]
fn test_geometrycollection_from_vec_geometry() {
    let vec_geoms = vec![
        Geometry::new_from_wkt("POINT (1 2)").unwrap(),
        Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap(),
        Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap(),
    ];
    let gc = Geometry::create_geometry_collection(vec_geoms).unwrap();
    assert_eq!(
        gc.to_wkt(),
        Ok("GEOMETRYCOLLECTION (POINT (1 2), LINESTRING (1 1, 10 50, 20 25), POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0)))".to_owned()),
    );
}

#[test]
fn test_error_multi_from_vec_single() {
    let vec_geoms = vec![
        Geometry::new_from_wkt("POINT (1.3 2.4)").unwrap(),
        Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap(),
    ];
    let multi_point = Geometry::create_multipoint(vec_geoms);
    let e = multi_point.err().unwrap();

    assert_eq!(
        e.to_string(),
        "impossible operation: all the provided geometry have to be of type Point".to_string(),
    );
}

#[test]
fn test_get_geometry_n() {
    let multilinestring =
        Geometry::new_from_wkt("MULTILINESTRING ((1 1, 10 50, 20 25), (0 0, 7 7, 45 50, 100 100))")
            .unwrap();
    let l0 = multilinestring.get_geometry_n(0).unwrap();
    let l1 = multilinestring.get_geometry_n(1).unwrap();

    assert_eq!(l0.to_wkt(), Ok("LINESTRING (1 1, 10 50, 20 25)".to_owned()),);
    assert_eq!(
        l1.to_wkt(),
        Ok("LINESTRING (0 0, 7 7, 45 50, 100 100)".to_owned()),
    );
}

#[rustfmt::skip]
#[test]
fn test_incompatible_types() {
    use crate::JoinStyle;

    let point = Geometry::new_from_wkt("POINT (0 0)").unwrap();
    let line = Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap();
    let ring = Geometry::new_from_wkt("LINEARRING(1 1,10 50,20 25, 1 1)").unwrap();
    let polygon = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0), (1 1, 1 2, 2 2, 2 1, 1 1))").unwrap();
    let multiline = Geometry::new_from_wkt("MULTILINESTRING ((1 1, 10 50, 20 25), (0 0, 7 7, 45 50))").unwrap();
    let multipolygon = Geometry::new_from_wkt("MULTIPOLYGON (((0 0, 0 5, 5 5, 5 0, 0 0)), ((1 1, 1 3, 5 5, 5 0, 1 1)))").unwrap();
    #[cfg(feature = "v3_13_0")]
    let curve = Geometry::new_from_wkt("CIRCULARSTRING(1 1,10 50,20 25)").unwrap();
    #[cfg(feature = "v3_13_0")]
    let multicurve = Geometry::new_from_wkt("MULTICURVE ((1 1, 10 50, 20 25), CIRCULARSTRING(0 0, 7 7, 45 50))").unwrap();

    assert!(point.get_start_point().is_err());
    assert!(line.get_start_point().is_ok());
    assert!(ring.get_start_point().is_ok());
    assert!(polygon.get_start_point().is_err());
    assert!(multiline.get_start_point().is_err());
    assert!(multipolygon.get_start_point().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        #[cfg(not(feature = "v3_14_0"))]
        assert!(curve.get_start_point().is_err());
        #[cfg(feature = "v3_14_0")]
        assert!(curve.get_start_point().is_ok());
        assert!(multicurve.get_start_point().is_err());
    }

    assert!(point.get_end_point().is_err());
    assert!(line.get_end_point().is_ok());
    assert!(ring.get_end_point().is_ok());
    assert!(polygon.get_end_point().is_err());
    assert!(multiline.get_end_point().is_err());
    assert!(multipolygon.get_end_point().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        #[cfg(not(feature = "v3_14_0"))]
        assert!(curve.get_end_point().is_err());
        #[cfg(feature = "v3_14_0")]
        assert!(curve.get_end_point().is_ok());
        assert!(multicurve.get_end_point().is_err());
    }

    assert!(point.get_point_n(0).is_err());
    assert!(line.get_point_n(0).is_ok());
    assert!(ring.get_point_n(0).is_ok());
    assert!(polygon.get_point_n(0).is_err());
    assert!(multiline.get_point_n(0).is_err());
    assert!(multipolygon.get_point_n(0).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        #[cfg(not(feature = "v3_14_0"))]
        assert!(curve.get_point_n(0).is_err());
        #[cfg(feature = "v3_14_0")]
        assert!(curve.get_point_n(0).is_ok());
        assert!(multicurve.get_point_n(0).is_err());
    }

    assert!(point.get_num_points().is_err());
    assert!(line.get_num_points().is_ok());
    assert!(ring.get_num_points().is_ok());
    assert!(polygon.get_num_points().is_err());
    assert!(multiline.get_num_points().is_err());
    assert!(multipolygon.get_num_points().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_num_points().is_ok());
        assert!(multicurve.get_num_points().is_err());
    }

    assert!(point.get_coord_seq().is_ok());
    assert!(line.get_coord_seq().is_ok());
    assert!(ring.get_coord_seq().is_ok());
    assert!(multiline.get_coord_seq().is_err());
    assert!(multipolygon.get_coord_seq().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_coord_seq().is_ok());
        assert!(multicurve.get_coord_seq().is_err());
    }

    assert!(point.is_closed().is_err());
    assert!(line.is_closed().is_ok());
    assert!(ring.is_closed().is_ok());
    assert!(polygon.is_closed().is_err());
    assert!(multiline.is_closed().is_ok());
    assert!(multipolygon.is_closed().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.is_closed().is_ok());
        assert!(multicurve.is_closed().is_ok());
    }

    assert!(point.interpolate(1.0).is_err());
    assert!(line.interpolate(1.0).is_ok());
    assert!(ring.interpolate(1.0).is_ok());
    assert!(polygon.interpolate(1.0).is_err());
    assert!(multiline.interpolate(1.0).is_ok());
    assert!(multipolygon.interpolate(1.0).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.interpolate(1.0).is_err());
        assert!(multicurve.interpolate(1.0).is_ok());
    }

    assert!(point.interpolate_normalized(1.0).is_err());
    assert!(line.interpolate_normalized(1.0).is_ok());
    assert!(ring.interpolate_normalized(1.0).is_ok());
    assert!(polygon.interpolate_normalized(1.0).is_err());
    assert!(multiline.interpolate_normalized(1.0).is_ok());
    assert!(multipolygon.interpolate_normalized(1.0).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.interpolate_normalized(1.0).is_err());
        assert!(multicurve.interpolate_normalized(1.0).is_err());
    }

    assert!(point.project(&point).is_err());
    assert!(line.project(&polygon).is_err());
    assert!(line.project(&point).is_ok());
    assert!(ring.project(&point).is_ok());
    assert!(polygon.project(&point).is_err());
    assert!(multiline.project(&point).is_ok());
    assert!(multipolygon.project(&point).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.project(&point).is_err());
        assert!(multicurve.project(&point).is_err());
    }

    #[cfg(not(feature = "v3_8_0"))]{
        assert!(point.project_normalized(&point).is_ok());
        assert!(line.project_normalized(&polygon).is_ok());
        assert!(line.project_normalized(&point).is_ok());
        assert!(ring.project_normalized(&point).is_ok());
        assert!(polygon.project_normalized(&point).is_ok());
        assert!(multiline.project_normalized(&point).is_ok());
        assert!(multipolygon.project_normalized(&point).is_ok());
    }
    #[cfg(feature = "v3_8_0")] {
        assert!(point.project_normalized(&point).is_err());
        assert!(line.project_normalized(&polygon).is_err());
        assert!(line.project_normalized(&point).is_ok());
        assert!(ring.project_normalized(&point).is_ok());
        assert!(polygon.project_normalized(&point).is_err());
        assert!(multiline.project_normalized(&point).is_ok());
        assert!(multipolygon.project_normalized(&point).is_err());
    }
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.project_normalized(&point).is_err());
        assert!(multicurve.project_normalized(&point).is_err());
    }

    assert!(line.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
    assert!(ring.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
    #[cfg(not(feature = "v3_11_0"))]
    {
        assert!(point.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
        assert!(polygon.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
        assert!(multiline.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
        assert!(multipolygon.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
    }
    #[cfg(feature = "v3_11_0")]
    {
        assert!(point.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
        assert!(polygon.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
        assert!(multiline.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
        assert!(multipolygon.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_ok());
    }
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
        assert!(multicurve.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
    }

    assert!(point.shared_paths(&line).is_err());
    assert!(line.shared_paths(&point).is_err());
    assert!(line.shared_paths(&line).is_ok());
    assert!(ring.shared_paths(&line).is_ok());
    assert!(polygon.shared_paths(&line).is_err());
    assert!(multiline.shared_paths(&line).is_ok());
    assert!(multipolygon.shared_paths(&line).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.shared_paths(&line).is_err());
        assert!(multicurve.shared_paths(&line).is_err());
    }

    assert!(point.get_exterior_ring().is_err());
    assert!(line.get_exterior_ring().is_err());
    assert!(ring.get_exterior_ring().is_err());
    assert!(polygon.get_exterior_ring().is_ok());
    assert!(multiline.get_exterior_ring().is_err());
    #[cfg(not(feature = "v3_11_0"))]
    assert!(multipolygon.offset_curve(1.0, 8, JoinStyle::Round, 5.0).is_err());
    #[cfg(feature = "v3_11_0")]
    assert!(multipolygon.get_exterior_ring().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_exterior_ring().is_err());
        assert!(multicurve.get_exterior_ring().is_err());
    }

    assert!(point.get_interior_ring_n(0).is_err());
    assert!(line.get_interior_ring_n(0).is_err());
    assert!(ring.get_interior_ring_n(0).is_err());
    assert!(polygon.get_interior_ring_n(0).is_ok());
    assert!(multiline.get_interior_ring_n(0).is_err());
    assert!(multipolygon.get_interior_ring_n(0).is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_interior_ring_n(0).is_err());
        assert!(multicurve.get_interior_ring_n(0).is_err());
    }

    assert!(point.get_num_interior_rings().is_err());
    assert!(line.get_num_interior_rings().is_err());
    assert!(ring.get_num_interior_rings().is_err());
    assert!(polygon.get_num_interior_rings().is_ok());
    assert!(multiline.get_num_interior_rings().is_err());
    assert!(multipolygon.get_num_interior_rings().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_num_interior_rings().is_err());
        assert!(multicurve.get_num_interior_rings().is_err());
    }

    assert!(point.get_x().is_ok());
    assert!(line.get_x().is_err());
    assert!(ring.get_x().is_err());
    assert!(polygon.get_x().is_err());
    assert!(multiline.get_x().is_err());
    assert!(multipolygon.get_x().is_err());
    #[cfg(feature = "v3_13_0")]
    {
        assert!(curve.get_x().is_err());
        assert!(multicurve.get_x().is_err());
    }
}

#[test]
#[cfg(feature = "v3_12_0")]
fn test_m_coordinates() {
    use crate::{CoordDimensions, CoordSeq, CoordType};

    let seq = CoordSeq::new_from_buffer(&[1.2, 2.4, 3.6], 1, CoordType::XYM).unwrap();
    assert_eq!(seq.dimensions(), Ok(CoordDimensions::ThreeD));

    let geom = Geometry::create_point(seq).unwrap();
    assert_eq!(geom.get_coordinate_dimension(), Ok(CoordDimensions::ThreeD));
    assert_eq!(geom.has_m(), Ok(true));
    assert_eq!(geom.has_z(), Ok(false));
    assert_eq!(geom.get_coordinate_type(), Ok(CoordType::XYM));
}

fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.0001);
    assert!(f > 0.9999);
}

#[test]
#[cfg(feature = "v3_10_0")]
fn test_make_valid_structure_method() {
    use crate::{MakeValidMethod, MakeValidParams};

    let invalid_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 0 1, 1 0, 0 0))").unwrap();
    assert!(!invalid_geom.is_valid().unwrap());

    let params = MakeValidParams::builder()
        .method(MakeValidMethod::Structure)
        .build()
        .unwrap();

    let valid_geom = invalid_geom
        .make_valid_with_params(&params)
        .expect("make_valid_with_params failed");

    assert!(valid_geom.is_valid().unwrap());
}

#[test]
#[cfg(feature = "v3_10_0")]
fn test_make_valid_linework_method() {
    use crate::{MakeValidMethod, MakeValidParams};

    let invalid_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 1, 0 1, 1 0, 0 0))").unwrap();
    assert!(!invalid_geom.is_valid().unwrap());

    let params = MakeValidParams::builder()
        .method(MakeValidMethod::Linework)
        .build()
        .unwrap();

    let valid_geom = invalid_geom
        .make_valid_with_params(&params)
        .expect("make_valid_with_params failed");

    assert!(valid_geom.is_valid().unwrap());
}

#[test]
#[cfg(feature = "v3_10_0")]
fn test_make_valid_keep_collapsed_true() {
    use crate::MakeValidParams;

    let collapsed_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 0, 1 0, 0 0))").unwrap();

    let params = MakeValidParams::builder()
        .keep_collapsed(true)
        .build()
        .unwrap();

    let valid_geom = collapsed_geom
        .make_valid_with_params(&params)
        .expect("make_valid_with_params failed");

    assert!(valid_geom.is_valid().unwrap());
}

#[test]
#[cfg(feature = "v3_10_0")]
fn test_make_valid_keep_collapsed_false() {
    use crate::MakeValidParams;

    let collapsed_geom = Geometry::new_from_wkt("POLYGON((0 0, 1 0, 1 0, 0 0))").unwrap();

    let params = MakeValidParams::builder()
        .keep_collapsed(false)
        .build()
        .unwrap();

    let valid_geom = collapsed_geom
        .make_valid_with_params(&params)
        .expect("make_valid_with_params failed");

    assert!(valid_geom.is_valid().unwrap());
}
