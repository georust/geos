#[cfg(test)]
mod test {
    use ffi::{GEOSGeomTypes, GGeom, PreparedGGeom};

    #[test]
    fn test_relationship() {
        let pt_geom = GGeom::new("POINT (2.5 2.5)").unwrap();
        let line_geom = GGeom::new("LINESTRING(1 1,10 50,20 25)").unwrap();
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();

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
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
        let new_geom = polygon_geom.buffer(100.0, 12).unwrap();
        let g1 = new_geom.difference(&polygon_geom).unwrap();
        let g2 = polygon_geom.sym_difference(&new_geom).unwrap();
        let g3 = new_geom.sym_difference(&polygon_geom).unwrap();
        assert_almost_eq(g1.area().unwrap(), g2.area().unwrap());
        assert_almost_eq(g2.area().unwrap(), g3.area().unwrap());
        let g4 = g3.get_centroid().unwrap();
        assert_eq!(GEOSGeomTypes::Point, g4.geometry_type().unwrap());
        let g5 = g4.buffer(200.0, 12).unwrap();

        assert!(g5.area().unwrap() > g4.area().unwrap());
        assert_eq!(GEOSGeomTypes::Polygon, g5.geometry_type().unwrap());
    }

    #[test]
    fn test_prepared_geoms() {
        let g1 = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))").unwrap();
        let g2 = GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))").unwrap();
        let pg1 = PreparedGGeom::new(&g1);
        assert_eq!(true, pg1.intersects(&g2).unwrap());
        assert_eq!(true, pg1.contains(&g2.get_centroid().unwrap()).unwrap());
        let vec_geoms = vec![
            GGeom::new("POINT (1.3 2.4)").unwrap(),
            GGeom::new("POINT (2.1 0.3)").unwrap(),
            GGeom::new("POINT (3.1 4.7)").unwrap(),
            GGeom::new("POINT (0.4 4.1)").unwrap(),
        ];
        for geom in &vec_geoms {
            assert_eq!(true, pg1.intersects(&geom).unwrap());
        }
    }

    #[test]
    fn test_wkt_rounding_precision() {
        let g = GGeom::new("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)").unwrap();
        let wkt = g.to_wkt_precision(Some(0));
        assert_eq!(true, wkt == "LINESTRING (0 0, 7 7, 45 50, 100 100)");
        let wkt2 = g.to_wkt();
        assert!(wkt2 != wkt);
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.0001);
        assert!(f > 0.9999);
    }
}
