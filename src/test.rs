#[cfg(test)]
mod test {
    use ffi::{_point, CoordSeq, GEOSGeomTypes, GGeom, PreparedGGeom, _lineString, _linearRing};

    #[test]
    fn test_new_geometry_from_wkt_wkb() {
        let geom = GGeom::new("POINT (2.5 2.5)");
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, geom._type);
        assert_eq!(true, geom.is_simple());
        assert_eq!(true, geom.is_valid());
        assert_eq!(false, geom.is_empty());
        let line_geom = GGeom::new("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)");
        assert_eq!(GEOSGeomTypes::GEOS_LINESTRING as i32, line_geom._type);
        let (wkb_geom, size) = geom.to_wkb();
        let g3 = GGeom::new_from_wkb(wkb_geom, size);
        assert_eq!(true, g3.equals(&geom));
    }

    #[test]
    fn test_relationship() {
        let pt_geom = GGeom::new("POINT (2.5 2.5)");
        let line_geom = GGeom::new("LINESTRING(1 1,10 50,20 25)");
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");

        assert_eq!(true, polygon_geom.covers(&pt_geom));
        assert_eq!(true, polygon_geom.intersects(&pt_geom));
        assert_eq!(false, polygon_geom.covered_by(&pt_geom));
        assert_eq!(false, polygon_geom.equals(&pt_geom));
        assert_eq!(false, polygon_geom.within(&pt_geom));

        assert_eq!(false, pt_geom.covers(&polygon_geom));
        assert_eq!(true, pt_geom.intersects(&polygon_geom));
        assert_eq!(true, pt_geom.covered_by(&polygon_geom));
        assert_eq!(false, pt_geom.equals(&polygon_geom));
        assert_eq!(true, pt_geom.within(&polygon_geom));

        assert_eq!(false, line_geom.covers(&pt_geom));
        assert_eq!(false, line_geom.intersects(&pt_geom));
        assert_eq!(false, line_geom.covered_by(&pt_geom));
        assert_eq!(false, pt_geom.covered_by(&line_geom));
        assert_eq!(true, line_geom.intersects(&polygon_geom));
        assert_eq!(true, line_geom.crosses(&polygon_geom));
        assert_eq!(false, line_geom.equals(&pt_geom));
    }

    #[test]
    fn test_geom_creation_from_geoms() {
        let polygon_geom = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
        let new_geom = polygon_geom.buffer(100.0, 12);
        let g1 = new_geom.difference(&polygon_geom);
        let g2 = polygon_geom.sym_difference(&new_geom);
        let g3 = new_geom.sym_difference(&polygon_geom);
        assert_almost_eq(g1.area, g2.area);
        assert_almost_eq(g2.area, g3.area);
        let g4 = g3.get_centroid();
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, g4._type);
        let g5 = g4.buffer(200.0, 12);

        assert!(g5.area > g4.area);
        assert_eq!(GEOSGeomTypes::GEOS_POLYGON as i32, g5._type);
    }

    #[test]
    fn test_prepared_geoms() {
        let g1 = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
        let g2 = GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))");
        let pg1 = PreparedGGeom::new(&g1);
        assert_eq!(true, pg1.intersects(&g2));
        assert_eq!(true, pg1.contains(&g2.get_centroid()));
        let vec_geoms = vec![
            GGeom::new("POINT (1.3 2.4)"),
            GGeom::new("POINT (2.1 0.3)"),
            GGeom::new("POINT (3.1 4.7)"),
            GGeom::new("POINT (0.4 4.1)"),
        ];
        for geom in &vec_geoms {
            assert_eq!(true, pg1.intersects(&geom));
        }
    }

    #[test]
    fn test_wkt_rounding_precision() {
        let g = GGeom::new("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)");
        let wkt = g.to_wkt_precison(Some(0));
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
