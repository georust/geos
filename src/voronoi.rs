use error::Error;
use ffi::GGeom;
use from_geo::TryInto;
use geo_types::{Geometry, GeometryCollection, Point, Polygon};

//TODO, change  &[] to IntoIterator
pub fn compute_voronoi(points: &[Point<f64>], tolerance: f64) -> Result<Vec<Polygon<f64>>, Error> {
    let geom_points: GGeom = points.try_into()?;

    geom_points
        .voronoi(None, tolerance, false)?
        .try_into()
        .and_then(|g: Geometry<f64>| match g {
            Geometry::GeometryCollection(gc) => Ok(gc),
            _ => Err(Error::ConversionError("invalid geometry type".into())),
        }).and_then(|gc: GeometryCollection<f64>| {
            gc.0.into_iter()
                .map(|g| {
                    g.as_polygon()
                        .ok_or(Error::ConversionError("invalid inner geometry type".into()))
                }).collect()
        })
}

#[cfg(test)]
mod test {
    use ffi::GGeom;
    use geo_types::{LineString, Point, Polygon, Coordinate};
    /// create a voronoi diagram. Same unit test as https://github.com/libgeos/geos/blob/master/tests/unit/triangulate/VoronoiTest.cpp#L118
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

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();

        let same = expected_output.equals(&voronoi).unwrap();
        assert!(same);
    }

    /// test precision. Same unit test as https://github.com/libgeos/geos/blob/master/tests/unit/triangulate/VoronoiTest.cpp#L160
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

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();

        let same = expected_output.equals(&voronoi).unwrap();
        assert!(same);
    }

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
        tuples.into_iter().map(Coordinate::from).collect()
    }


    // test the rust-geo voronoi wrapper
    #[test]
    fn geo_voronoi() {
        let points = vec![
            Point::new(0., 0.),
            Point::new(0., 1.),
            Point::new(1., 1.),
            Point::new(1., 0.),
        ];

        let voronoi = ::compute_voronoi(&points, 0.).unwrap();

        let poly = vec![
            Polygon::new(
                LineString(coords(vec![
                    (0.5, 2.0),
                    (2.0, 2.0),
                    (2.0, 0.5),
                    (0.5, 0.5),
                    (0.5, 2.0),
                ])),
                vec![],
            ),
            Polygon::new(
                LineString(coords(vec![
                    (-1.0, 0.5),
                    (-1.0, 2.0),
                    (0.5, 2.0),
                    (0.5, 0.5),
                    (-1.0, 0.5),
                ])),
                vec![],
            ),
            Polygon::new(
                LineString(coords(vec![
                    (0.5, -1.0),
                    (-1.0, -1.0),
                    (-1.0, 0.5),
                    (0.5, 0.5),
                    (0.5, -1.0),
                ])),
                vec![],
            ),
            Polygon::new(
                LineString(coords(vec![
                    (2.0, 0.5),
                    (2.0, -1.0),
                    (0.5, -1.0),
                    (0.5, 0.5),
                    (2.0, 0.5),
                ])),
                vec![],
            ),
        ];

        assert_eq!(poly, voronoi);
    }
}
