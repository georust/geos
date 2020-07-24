#[cfg(test)]
mod test {
    use crate::{Geom, Geometry, PreparedGeometry};
    use enums::GeometryTypes;

    #[test]
    fn test_relationship() {
        let pt_geom = Geometry::new_from_wkt("POINT (2.5 2.5)").unwrap();
        let line_geom = Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap();
        let polygon_geom = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();

        assert_eq!(true, polygon_geom.covers(&pt_geom).unwrap());
        assert_eq!(true, polygon_geom.intersects(&pt_geom).unwrap());
        assert_eq!(false, polygon_geom.covered_by(&pt_geom).unwrap());
        assert_eq!(false, polygon_geom.equals(&pt_geom).unwrap());
        assert_eq!(false, polygon_geom.within(&pt_geom).unwrap());

        assert_eq!(false, pt_geom.covers(&polygon_geom).unwrap());
        assert_eq!(true, pt_geom.intersects(&polygon_geom).unwrap());
        assert_eq!(true, pt_geom.covered_by(&polygon_geom).unwrap());
        assert_eq!(false, pt_geom.equals(&polygon_geom).unwrap());
        assert_eq!(true, pt_geom.within(&polygon_geom).unwrap());

        assert_eq!(false, line_geom.covers(&pt_geom).unwrap());
        assert_eq!(false, line_geom.intersects(&pt_geom).unwrap());
        assert_eq!(false, line_geom.covered_by(&pt_geom).unwrap());
        assert_eq!(false, pt_geom.covered_by(&line_geom).unwrap());
        assert_eq!(true, line_geom.intersects(&polygon_geom).unwrap());
        assert_eq!(true, line_geom.crosses(&polygon_geom).unwrap());
        assert_eq!(false, line_geom.equals(&pt_geom).unwrap());
    }

    #[test]
    fn test_geom_creation_from_geoms() {
        let polygon_geom = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
        let new_geom = polygon_geom.buffer(100.0, 12).expect("buffer failed");
        let g1 = new_geom.difference(&polygon_geom).expect("difference failed");
        let g2 = polygon_geom.sym_difference(&new_geom).expect("sym difference failed");
        let g3 = new_geom.sym_difference(&polygon_geom).expect("sym difference 2 faileed");
        assert_almost_eq(g1.area().expect("area 1.1 failed"), g2.area().expect("area 1.2 failed"));
        assert_almost_eq(g2.area().expect("area 2.1 failed"), g3.area().expect("area 2.2 failed"));
        let g4 = g3.get_centroid().expect("get_centroid failed");
        assert_eq!(GeometryTypes::Point, g4.geometry_type());
        let g5 = g4.buffer(200.0, 12).expect("buffer 2 failed");

        assert!(g5.area().expect("area 3.1 failed") > g4.area().expect("area 3.2 failed"));
        assert_eq!(GeometryTypes::Polygon, g5.geometry_type());
    }

    #[test]
    fn test_prepared_geoms() {
        let g1 = Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
        let g2 = Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))").unwrap();
        let pg1 = PreparedGeometry::new(&g1).expect("failed to create prepared geom");
        assert_eq!(true, pg1.intersects(&g2).unwrap());
        assert_eq!(true, pg1.contains(&g2.get_centroid().unwrap()).unwrap());
        let vec_geoms = vec![
            Geometry::new_from_wkt("POINT (1.3 2.4)").unwrap(),
            Geometry::new_from_wkt("POINT (2.1 0.3)").unwrap(),
            Geometry::new_from_wkt("POINT (3.1 4.7)").unwrap(),
            Geometry::new_from_wkt("POINT (0.4 4.1)").unwrap(),
        ];
        for geom in &vec_geoms {
            assert_eq!(true, pg1.intersects(geom).unwrap());
        }
    }

    #[test]
    fn test_wkt_rounding_precision() {
        let g = Geometry::new_from_wkt("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)").unwrap();
        let wkt = g.to_wkt_precision(0);
        assert_eq!(wkt, Ok("LINESTRING (0 0, 7 7, 45 50, 100 100)".to_owned()));
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
        assert_eq!(
            multi_point.to_wkt_precision(1),
            Ok("MULTIPOINT (1.3 2.4, 2.1 0.3, 3.1 4.7, 0.4 4.1)".to_owned()),
        )
    }

    #[test]
    fn test_multilinestring_from_vec_single() {
        let vec_geoms = vec![
            Geometry::new_from_wkt("LINESTRING(1 1,10 50,20 25)").unwrap(),
            Geometry::new_from_wkt("LINESTRING (0 0, 7 7, 45 50, 100 100)").unwrap(),
        ];
        let multi_linestring = Geometry::create_multiline_string(vec_geoms).unwrap();
        assert_eq!(
            multi_linestring.to_wkt_precision(0),
            Ok("MULTILINESTRING ((1 1, 10 50, 20 25), (0 0, 7 7, 45 50, 100 100))".to_owned()),
        )
    }

    #[test]
    fn test_multipolygon_from_vec_single() {
        let vec_geoms = vec![
            Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap(),
            Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))").unwrap(),
        ];
        let multi_polygon = Geometry::create_multipolygon(vec_geoms).unwrap();
        assert_eq!(
            multi_polygon.to_wkt_precision(0),
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
            gc.to_wkt_precision(0),
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
            format!("{}", e),
            "Impossible operation, all the provided geometry have to be of type Point".to_string(),
        );
    }

    #[test]
    fn test_get_geometry_n() {
        let multilinestring = Geometry::new_from_wkt("MULTILINESTRING ((1 1, 10 50, 20 25), (0 0, 7 7, 45 50, 100 100))").unwrap();
        let l0 = multilinestring.get_geometry_n(0).unwrap();
        let l1 = multilinestring.get_geometry_n(1).unwrap();

        assert_eq!(
            l0.to_wkt_precision(0),
            Ok("LINESTRING (1 1, 10 50, 20 25)".to_owned()),
        );
        assert_eq!(
            l1.to_wkt_precision(0),
            Ok("LINESTRING (0 0, 7 7, 45 50, 100 100)".to_owned()),
        );
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.0001);
        assert!(f > 0.9999);
    }
}
