#[cfg(test)]
mod test {
    use lib::*;
	use types_geom::*;

    #[test]
    fn test_new_geometry_from_wkt_wkb() {
        let geom = GGeom::new("POINT (2.5 2.5)");
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, geom._type);
        assert_eq!(true, geom.is_simple());
        assert_eq!(false, geom.is_empty());
        let line_geom = GGeom::new("LINESTRING(0.0 0.0, 7.0 7.0, 45.0 50.5, 100.0 100.0)");
        assert_eq!(GEOSGeomTypes::GEOS_LINESTRING as i32, line_geom._type);
        let (wkb_geom, size) = geom.to_wkb();
        let g3 = GGeom::new_from_wkb(wkb_geom, size);
        assert_eq!(true, g3.equals(&geom));
    }

    #[test]
    fn test_relationship(){
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
    fn test_geom_creation_from_geoms(){
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
            GGeom::new("POINT (0.4 4.1)")
            ];
        for geom in &vec_geoms {
            assert_eq!(true, pg1.intersects(&geom));
        }
    }

    #[test]
    fn test_geom_from_coord_seq(){
        let sequence = CoordSeq::new(1, 2);
        sequence.set_x(0, 12.36);
        sequence.set_y(0, 43.21);

        let geom_point = _Point(&sequence);
        assert_eq!(GEOSGeomTypes::GEOS_POINT as i32, geom_point._type);

        let sequence2 = CoordSeq::new(2, 2);
        sequence2.set_x(0, 12.36);
        sequence2.set_y(0, 43.21);

        sequence2.set_x(1, 12.78);
        sequence2.set_y(1, 42.80);

        let geom_line = _LineString(&sequence2);
        assert_eq!(GEOSGeomTypes::GEOS_LINESTRING as i32, geom_line._type);

        let x2 = sequence2.get_x(0);
        assert_almost_eq(12.36, x2);

        let exterior_ring = Ring::new(&[(0.0, 0.0), (0.0, 8.0), (8.0, 8.0), (8.0, 0.0), (0.0, 0.0)]);
        let interior = Ring::new(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)]);
        let poly_geom = Polygon::new(&exterior_ring, &[interior]);
        assert_eq!(GEOSGeomTypes::GEOS_POLYGON as i32, poly_geom._type);
    }

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.0001);
        assert!(f > 0.9999);
    }
}
