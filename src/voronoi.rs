use crate::error::Error;
use crate::{Geom, Geometry as GGeometry};
use geo_types::{Geometry, GeometryCollection, Point, Polygon};

use std::borrow::Borrow;
use std::convert::TryInto;

/// Available using the `geo` feature.
/// About the `tolerance` argument, the underlying C library mentions:
/// > snapping tolerance to use for improved robustness. A tolerance of 0.0 specifies that no snapping
/// > will take place. This argument can be finicky and is known to cause the algorithm to fail in
/// > several cases. If you're using tolerance and getting a failure, try setting it to 0.0.
pub fn compute_voronoi<T: Borrow<Point<f64>>>(
    points: &[T],
    envelope: Option<&GGeometry>,
    tolerance: f64,
    only_edges: bool,
) -> Result<Vec<Polygon<f64>>, Error> {
    let geom_points: GGeometry = points.try_into()?;

    let mut voronoi = geom_points.voronoi(envelope, tolerance, only_edges)?;

    voronoi.normalize()?;

    voronoi
        .try_into()
        .and_then(|g: Geometry<f64>| match g {
            Geometry::GeometryCollection(gc) => Ok(gc),
            _ => Err(Error::ConversionError("invalid geometry type".into())),
        })
        .and_then(|gc: GeometryCollection<f64>| {
            gc.0.into_iter()
                .map(|g| {
                    g.try_into().map_err(|e| {
                        Error::ConversionError(format!("invalid inner geometry type: {e}"))
                    })
                })
                .collect()
        })
}

#[cfg(test)]
mod test {
    use crate::{Geom, Geometry as GGeometry};
    use geo_types::{Coord, LineString, Point, Polygon};
    // create a voronoi diagram. Same unit test as
    // https://github.com/libgeos/geos/blob/master/tests/unit/triangulate/VoronoiTest.cpp#L118
    #[test]
    fn simple_voronoi() {
        let points = "MULTIPOINT ((150 200), (180 270), (275 163))";
        let input = GGeometry::new_from_wkt(points).unwrap();

        let mut voronoi = input.voronoi(None::<&GGeometry>, 0., false).unwrap();

        let expected_output = "GEOMETRYCOLLECTION (
            POLYGON ((25 38, 25 295, 221.20588235294116 210.91176470588235, 170.024 38, 25 38)),
            POLYGON ((400 369.6542056074766, 400 38, 170.024 38, 221.20588235294116 210.91176470588235, 400 369.6542056074766)),
            POLYGON ((25 295, 25 395, 400 395, 400 369.6542056074766, 221.20588235294116 210.91176470588235, 25 295)))";

        let mut expected_output = GGeometry::new_from_wkt(expected_output).unwrap();

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();

        assert_eq!(
            voronoi.to_wkt_precision(10).unwrap(),
            expected_output.to_wkt_precision(10).unwrap()
        );
    }

    // test precision. Same unit test as
    // https://github.com/libgeos/geos/blob/master/tests/unit/triangulate/VoronoiTest.cpp#L181
    #[test]
    fn wkt_voronoi_precision() {
        let points = "MULTIPOINT ((100 200), (105 202), (110 200), (140 230),
        (210 240), (220 190), (170 170), (170 260), (213 245), (220 190))";
        let input = GGeometry::new_from_wkt(points).unwrap();

        let mut voronoi = input.voronoi(None::<&GGeometry>, 6., false).unwrap();

        let expected_output = "GEOMETRYCOLLECTION (
        POLYGON ((-20 50, -20 380, -3.75 380, 105 235, 105 115, 77.14285714285714 50, -20 50)),
        POLYGON ((247 50, 77.14285714285714 50, 105 115, 145 195, 178.33333333333334 211.66666666666666, 183.51851851851853 208.7037037037037, 247 50)),
        POLYGON ((-3.75 380, 20.000000000000007 380, 176.66666666666666 223.33333333333334, 178.33333333333334 211.66666666666666, 145 195, 105 235, -3.75 380)),
        POLYGON ((105 115, 105 235, 145 195, 105 115)),
        POLYGON ((20.000000000000007 380, 255 380, 176.66666666666666 223.33333333333334, 20.000000000000007 380)),
        POLYGON ((255 380, 340 380, 340 240, 183.51851851851853 208.7037037037037, 178.33333333333334 211.66666666666666, 176.66666666666666 223.33333333333334, 255 380)),
        POLYGON ((340 240, 340 50, 247 50, 183.51851851851853 208.7037037037037, 340 240)))";

        let mut expected_output = GGeometry::new_from_wkt(expected_output).unwrap();

        expected_output.normalize().unwrap();
        voronoi.normalize().unwrap();

        assert_eq!(
            voronoi.to_wkt_precision(10).unwrap(),
            expected_output.to_wkt_precision(10).unwrap()
        );
    }

    // #[test]
    // fn check() {
    //     let geom = GGeometry::new_from_wkt("MULTIPOLYGON (((
    //         4.5687299000000001 7.6963754000000000, 4.5687299000000001 7.6957645999999995,
    //         4.5687299000000001 7.6954739999999999, 4.5687299000000001 7.6950833999999997,
    //         4.5687299000000001 7.6906570999999992, 4.5717796999999996 7.6765523000000000,
    //         4.5713982999999994 7.6712154999999997, 4.5706362999999994 7.6632099000000000,
    //         4.5702547999999998 7.6559672000000001, 4.5691112999999994 7.6521548999999993,
    //         4.5654883000000002 7.6492094999999996, 4.5632057000000001 7.6469274000000000,
    //         4.5614872000000002 7.6456741999999993, 4.5569123999999999 7.6433872999999997,
    //         4.5508131999999994 7.6437682999999996, 4.5466198999999996 7.6426248999999995,
    //         4.5447134999999994 7.6380501000000001, 4.5424265999999998 7.6315698999999997,
    //         4.5355648999999998 7.6277574999999995, 4.5314893999999999 7.6243319999999999,
    //         4.5290841999999998 7.6208958999999998, 4.5283217000000002 7.6186084999999997,
    //         4.5283217000000002 7.6186084999999997, 4.5263529000000000 7.6179522999999998,
    //         4.5263529000000000 7.6179522999999998, 4.5226034999999998 7.6167026000000000,
    //         4.5069736999999996 7.6201333999999994, 4.4985870999999999 7.6197518999999998,
    //         4.4848637999999994 7.6163210999999995, 4.4741897999999996 7.6140341999999999,
    //         4.4528417999999999 7.6144151999999998, 4.4384321000000000 7.6126138999999995,
    //         4.4345435999999996 7.6121277999999997, 4.4177704000000002 7.6109842999999993,
    //         4.4013786000000001 7.6071719999999994, 4.3983207999999996 7.6065133999999999,
    //         4.3973613999999994 7.6063066999999993, 4.3944855000000000 7.6056925999999994,
    //         4.3907045999999994 7.6048850999999997, 4.3754562999999997 7.6060285999999993,
    //         4.3674507000000000 7.6106028999999999, 4.3602075999999999 7.6247077000000001,
    //         4.3552517999999996 7.6369065999999997, 4.3533458999999999 7.6430058000000001,
    //         4.3533458999999999 7.6430058000000001, 4.3518208999999999 7.6471991999999993,
    //         4.3506774999999998 7.6525363999999998, 4.3506774999999998 7.6525363999999998,
    //         4.3541083000000000 7.6548232999999994, 4.3590640999999994 7.6624479000000001,
    //         4.3605890000000000 7.6666411999999999, 4.3605890000000000 7.6696906000000000,
    //         4.3568949999999997 7.6785568999999994, 4.3568949999999997 7.6785568999999994,
    //         4.3590640999999994 7.6803645999999999, 4.3647822999999999 7.6868452999999999,
    //         4.3708815999999997 7.6906570999999992, 4.3744268000000002 7.6918391999999995,
    //         4.3765998000000002 7.6925634999999994, 4.3765998000000002 7.6925634999999994,
    //         4.3849863999999998 7.6925634999999994, 4.4009971999999999 7.6975192999999997,
    //         4.4162458999999998 7.7032370999999999, 4.4280634000000001 7.7081928000000000,
    //         4.4417868000000000 7.7108616999999997, 4.4577974999999999 7.7100991999999993,
    //         4.4760957000000001 7.7097178000000000, 4.4921063999999999 7.7085742999999995,
    //         4.5107860999999998 7.7039994999999992, 4.5325150000000001 7.7001876999999999,
    //         4.5492917999999998 7.6979505999999995, 4.5496692999999997 7.6979002999999997,
    //         4.5584372999999996 7.6959944000000000, 4.5637740999999998 7.6952318999999996,
    //         4.5687299000000001 7.6963754000000000)))").expect("new_from_wkt failed");
    //     let points = GGeometry::new_from_wkt("MULTIPOINT(
    //         (4.4333330000000002 7.6666669999999996),
    //         (4.4500000000000002 7.6166669999999996),
    //         (4.4666670000000002 7.6333329999999995),
    //         (4.4333330000000002 7.6333329999999995),
    //         (4.5000000000000000 7.6499999999999995),
    //         (4.4333330000000002 7.6166669999999996),
    //         (4.3833329999999995 7.6166669999999996))").expect("new_from_wk2t failed");
    //     points.voronoi(None, 0., false).expect("voronoi failed");
    // }

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coord<f64>> {
        tuples.into_iter().map(Coord::from).collect()
    }

    // test the rust-geo voronoi wrapper
    #[test]
    fn geo_voronoi() {
        let points = vec![
            Point::new(0f64, 0.),
            Point::new(0f64, 1.),
            Point::new(1f64, 1.),
            Point::new(1f64, 0.),
        ];

        let voronoi = crate::compute_voronoi(&points, None, 0., false).unwrap();

        let poly = vec![
            Polygon::new(
                LineString(coords(vec![
                    (0.5, 0.5),
                    (0.5, 2.0),
                    (2.0, 2.0),
                    (2.0, 0.5),
                    (0.5, 0.5),
                ])),
                vec![],
            ),
            Polygon::new(
                LineString(coords(vec![
                    (0.5, -1.0),
                    (0.5, 0.5),
                    (2.0, 0.5),
                    (2.0, -1.0),
                    (0.5, -1.0),
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
                    (-1.0, -1.0),
                    (-1.0, 0.5),
                    (0.5, 0.5),
                    (0.5, -1.0),
                    (-1.0, -1.0),
                ])),
                vec![],
            ),
        ];

        assert_eq!(poly, voronoi);
    }
}
