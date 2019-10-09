use crate::{CoordDimensions, CoordSeq, Geometry as GGeom};
use error::{GResult, Error};
use geojson::{Value, Geometry};
use std;
// use std::borrow::Borrow;

// define our own TryInto while the std trait is not stable
pub trait TryInto<T> {
    type Err;
    fn try_into(self) -> Result<T, Self::Err>;
}

fn create_coord_seq_from_vec<'a>(coords: &'a [Vec<f64>]) -> Result<CoordSeq, Error> {
    create_coord_seq(coords.iter(), coords.len())
}

fn create_coord_seq<'a, 'b, It>(points: It, len: usize) -> Result<CoordSeq<'b>, Error>
where
    It: Iterator<Item = &'a Vec<f64>>,
{
    let mut coord_seq = CoordSeq::new(len as u32, CoordDimensions::TwoD)
        .expect("failed to create CoordSeq");

    for (i, p) in points.enumerate() {
        coord_seq.set_x(i, p[0])?;
        coord_seq.set_y(i, p[1])?;
    }
    Ok(coord_seq)
}

impl<'a> TryInto<GGeom<'a>> for &'a Geometry {
    type Err = Error;

    fn try_into(self) -> Result<GGeom<'a>, Self::Err> {
        match self.value {
            Value::Point(ref c) => {
                GGeom::create_point(create_coord_seq(std::iter::once(c), 1)?)
            },
            Value::MultiPoint(ref pts) =>  {
                let ggpts = pts.iter()
                    .map(|pt| {
                        GGeom::create_point(create_coord_seq(std::iter::once(pt), 1)?)
                    })
                    .collect::<GResult<Vec<GGeom>>>()?;
                GGeom::create_multipoint(ggpts)
            },
            Value::LineString(ref line) => {
                let coord_seq = create_coord_seq_from_vec(line.as_slice())?;
                GGeom::create_line_string(coord_seq)
            },
            Value::MultiLineString(ref lines) => {
                let gglines = lines.iter()
                    .map(|line| {
                        let coord_seq = create_coord_seq_from_vec(line.as_slice())?;
                        GGeom::create_line_string(coord_seq)
                    })
                    .collect::<GResult<Vec<GGeom>>>()?;
                GGeom::create_multiline_string(gglines)
            },
            Value::Polygon(ref rings) => {
                let exterior_ring = GGeom::create_linear_ring(
                    create_coord_seq_from_vec(rings[0].as_slice())?
                )?;
                let interiors = rings.iter()
                    .skip(1)
                    .map(|r| {
                        GGeom::create_linear_ring(
                            create_coord_seq_from_vec(r.as_slice())?)
                    })
                    .collect::<GResult<Vec<GGeom>>>()?;
                GGeom::create_polygon(exterior_ring, interiors)
            },
            Value::MultiPolygon(ref polygons) => {
                let ggpolys = polygons.iter()
                    .map(|rings|{
                        let exterior_ring = GGeom::create_linear_ring(
                            create_coord_seq_from_vec(rings[0].as_slice())?
                        )?;
                        let interiors = rings.iter()
                            .skip(1)
                            .map(|r| {
                                GGeom::create_linear_ring(
                                    create_coord_seq_from_vec(r.as_slice())?)
                            })
                            .collect::<GResult<Vec<GGeom>>>()?;
                        GGeom::create_polygon(exterior_ring, interiors)
                    })
                    .collect::<GResult<Vec<GGeom>>>()?;
                GGeom::create_multipolygon(ggpolys)
            },
            Value::GeometryCollection(ref geoms) => {
                let _geoms = geoms
                    .iter()
                    .map(|ref geom| geom.try_into())
                    .collect::<GResult<Vec<GGeom>>>()?;
                GGeom::create_geometry_collection(_geoms)
            }
        }
    }
}


// #[cfg(test)]
// mod test {
//     use super::GGeom;
//     use super::LineRing;
//     use from_geojson::TryInto;
//     use geojson::{Geometry, Value};
//
//     fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
//         tuples.into_iter().map(Coordinate::from).collect()
//     }
//
//     #[test]
//     fn polygon_contains_test() {
//         let exterior = LineString(coords(vec![
//             (0., 0.),
//             (0., 1.),
//             (1., 1.),
//             (1., 0.),
//             (0., 0.),
//         ]));
//         let interiors = vec![LineString(coords(vec![
//             (0.1, 0.1),
//             (0.1, 0.9),
//             (0.9, 0.9),
//             (0.9, 0.1),
//             (0.1, 0.1),
//         ]))];
//         let p = Polygon::new(exterior.clone(), interiors.clone());
//
//         assert_eq!(p.exterior(), &exterior);
//         assert_eq!(p.interiors(), interiors.as_slice());
//
//         let geom: GGeom = (&p).try_into().unwrap();
//
//         assert!(geom.contains(&geom).unwrap());
//         assert!(!geom.contains(&(&exterior).try_into().unwrap()).unwrap());
//
//         assert!(geom.covers(&(&exterior).try_into().unwrap()).unwrap());
//         assert!(geom.touches(&(&exterior).try_into().unwrap()).unwrap());
//     }
//
//     #[test]
//     fn multipolygon_contains_test() {
//         let exterior = LineString(coords(vec![
//             (0., 0.),
//             (0., 1.),
//             (1., 1.),
//             (1., 0.),
//             (0., 0.),
//         ]));
//         let interiors = vec![LineString(coords(vec![
//             (0.1, 0.1),
//             (0.1, 0.9),
//             (0.9, 0.9),
//             (0.9, 0.1),
//             (0.1, 0.1),
//         ]))];
//         let p = Polygon::new(exterior, interiors);
//         let mp = MultiPolygon(vec![p.clone()]);
//
//         let geom: GGeom = (&mp).try_into().unwrap();
//
//         assert!(geom.contains(&geom).unwrap());
//         assert!(geom.contains(&(&p).try_into().unwrap()).unwrap());
//     }
//
//     #[test]
//     fn incorrect_multipolygon_test() {
//         let exterior = LineString(coords(vec![(0., 0.)]));
//         let interiors = vec![];
//         let p = Polygon::new(exterior, interiors);
//         let mp = MultiPolygon(vec![p.clone()]);
//
//         let geom = (&mp).try_into();
//
//         assert!(geom.is_err());
//     }
//
//     #[test]
//     fn incorrect_polygon_not_closed() {
//         // even if the polygon is not closed we can convert it to geos (we close it)
//         let exterior = LineString(coords(vec![
//             (0., 0.),
//             (0., 2.),
//             (2., 2.),
//             (2., 0.),
//             (0., 0.),
//         ]));
//         let interiors = vec![LineString(coords(vec![
//             (0., 0.),
//             (0., 1.),
//             (1., 1.),
//             (1., 0.),
//             (0., 10.),
//         ]))];
//         let p = Polygon::new(exterior, interiors);
//         let mp = MultiPolygon(vec![p]);
//
//         let _g = (&mp).try_into().unwrap(); // no error
//     }
//
//     /// a linear ring can be empty
//     #[test]
//     fn empty_linear_ring() {
//         let ls = LineString(vec![]);
//         let geom: GGeom = LineRing(&ls).try_into().unwrap();
//
//         assert!(geom.is_valid());
//         assert!(geom.is_ring().unwrap());
//         assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 0);
//     }
//
//     /// a linear ring should have at least 3 elements
//     #[test]
//     fn one_elt_linear_ring() {
//         let ls = LineString(coords(vec![(0., 0.)]));
//         let geom: Result<GGeom, _> = LineRing(&ls).try_into();
//         let error = geom.err().unwrap();
//         assert_eq!(format!("{}", error), "Invalid geometry, impossible to create a LinearRing, A LinearRing must have at least 3 coordinates".to_string());
//     }
//
//     /// a linear ring should have at least 3 elements
//     #[test]
//     fn two_elt_linear_ring() {
//         let ls = LineString(coords(vec![(0., 0.), (0., 1.)]));
//         let geom: Result<GGeom, _> = LineRing(&ls).try_into();
//         let error = geom.err().unwrap();
//         assert_eq!(format!("{}", error), "Invalid geometry, impossible to create a LinearRing, A LinearRing must have at least 3 coordinates".to_string());
//     }
//
//     /// an unclosed linearring is valid since we close it before giving it to geos
//     #[test]
//     fn unclosed_linear_ring() {
//         let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.)]));
//         let geom: GGeom = LineRing(&ls).try_into().unwrap();
//
//         assert!(geom.is_valid());
//         assert!(geom.is_ring().unwrap());
//         assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
//     }
//
//     /// a bit tricky
//     /// a ring should have at least 3 points.
//     /// in the case of a closed ring with only element eg:
//     ///
//     /// let's take a point list: [p1, p2, p1]
//     ///
//     /// p1 ----- p2
//     ///  ^-------|
//     ///
//     /// we consider it like a 3 points not closed ring (with the 2 last elements being equals...)
//     ///
//     /// shapely (the python geos wrapper) considers that too
//     #[test]
//     fn closed_2_points_linear_ring() {
//         let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 1.)]));
//         let geom: GGeom = LineRing(&ls).try_into().unwrap();
//
//         assert!(geom.is_valid());
//         assert!(geom.is_ring().expect("is_ring failed"));
//         assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
//     }
//
//     /// a linear ring can be empty
//     #[test]
//     fn good_linear_ring() {
//         let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.), (0., 0.)]));
//         let geom: GGeom = LineRing(&ls).try_into().unwrap();
//
//         assert!(geom.is_valid());
//         assert!(geom.is_ring().unwrap());
//         assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
//     }
// }
