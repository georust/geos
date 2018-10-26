use geo_types::{Polygon, Point};

pub fn compute_voronoi(points: &[Point<f64>]) -> Vec<Polygon<f64>> {
    // let GGeom = 
    unimplemented!()
}

#[cfg(test)]
mod test {
    use geo_types::{Polygon, Point};
    use ffi::GGeom;
    /// create a voronoi diagram 
    #[test]
    fn simple_voronoi() {
        let points = "MULTIPOINT ((150 200), (180 270), (275 163))";
        let input = GGeom::new(points).unwrap();

        let mut voronoi = input.voronoi(None, 0., false).unwrap();

        let expected_output = "GEOMETRYCOLLECTION (
            POLYGON ((25 38, 25 295, 221.20588235294116 210.91176470588235, 170.024 38, 25 38)), 
            POLYGON ((400 369.6542056074766, 400 38, 170.024 38, 221.20588235294116 210.91176470588235, 400 369.6542056074766)), 
            POLYGON ((25 295, 25 395, 400 395, 400 369.6542056074766, 221.20588235294116 210.91176470588235, 25 295)))";

        let mut expected_output = GGeom::new(expected_output).unwrap();
        let wkt_output = voronoi.to_wkt();

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();
        println!("voronoi: {}", wkt_output);

        let same = expected_output.equals_exact(&voronoi, 1e-3).unwrap();
        assert!(same);
    }

    #[test]
    fn wkt_voronoi_precision() {
        let points = "MULTIPOINT ((100 200), (105 202), (110 200), (140 230), 
        (210 240), (220 190), (170 170), (170 260), (213 245), (220 190))";
        let input = GGeom::new(points).unwrap();

        let mut voronoi = input.voronoi(None, 6., false).unwrap();

        let expected_output = "GEOMETRYCOLLECTION (
            POLYGON ((-20 50, -20 380, -3.75 380, 105 235, 105 115, 77.14285714285714 50, -20 50)),
        POLYGON ((247 50, 77.14285714285714 50, 105 115, 145 195, 178.33333333333334 211.66666666666666, 183.51851851851853 208.7037037037037, 247 50)), 
        POLYGON ((-3.75 380, 20.000000000000007 380, 176.66666666666666 223.33333333333334, 178.33333333333334 211.66666666666666, 145 195, 105 235, -3.75 380)), 
        POLYGON ((105 115, 105 235, 145 195, 105 115)), 
        POLYGON ((20.000000000000007 380, 255 380, 176.66666666666666 223.33333333333334, 20.000000000000007 380)), 
        POLYGON ((255 380, 340 380, 340 240, 183.51851851851853 208.7037037037037, 178.33333333333334 211.66666666666666, 176.66666666666666 223.33333333333334, 255 380)), 
        POLYGON ((340 240, 340 50, 247 50, 183.51851851851853 208.7037037037037, 340 240)))";

        let mut expected_output = GGeom::new(expected_output).unwrap();
        let wkt_output = voronoi.to_wkt();

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();
        println!("voronoi: {}", wkt_output);

        let same = expected_output.equals_exact(&voronoi, 1e-3).unwrap();
        assert!(same);
    }
}