use crate::{CoordDimensions, CoordSeq, Geometry as GGeom};
use error::Error;
use geo_types::{Coordinate, LineString, MultiPolygon, Point, Polygon};
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};

fn create_coord_seq<'a, 'b, It>(points: It, len: usize) -> Result<CoordSeq<'b>, Error>
where
    It: Iterator<Item = &'a Coordinate<f64>>,
{
    let mut coord_seq =
        CoordSeq::new(len as u32, CoordDimensions::TwoD).expect("failed to create CoordSeq");
    for (i, p) in points.enumerate() {
        coord_seq.set_x(i, p.x)?;
        coord_seq.set_y(i, p.y)?;
    }
    Ok(coord_seq)
}

impl<'a> TryInto<GGeom<'a>> for &'a Point<f64> {
    type Error = Error;

    fn try_into(self) -> Result<GGeom<'a>, Self::Error> {
        let coord_seq = create_coord_seq(std::iter::once(&self.0), 1)?;

        GGeom::create_point(coord_seq)
    }
}

impl<'a, T: Borrow<Point<f64>>> TryFrom<&'a [T]> for GGeom<'a> {
    type Error = Error;

    fn try_from(points: &'a [T]) -> Result<GGeom<'a>, Self::Error> {
        let geom_points = points
            .into_iter()
            .map(|p| p.borrow().try_into())
            .collect::<Result<Vec<_>, _>>()?;

        GGeom::create_multipoint(geom_points)
    }
}

impl<'a> TryFrom<&'a LineString<f64>> for GGeom<'a> {
    type Error = Error;

    fn try_from(linestring: &'a LineString<f64>) -> Result<GGeom<'a>, Self::Error> {
        let mut coords = CoordSeq::new(linestring.num_coords() as u32, CoordDimensions::TwoD)?;
        linestring
            .points_iter()
            .enumerate()
            .try_for_each(|(i, p)| {
                coords.set_x(i, p.x())?;
                coords.set_y(i, p.y())?;
                Ok(())
            })?;
        GGeom::create_line_string(coords)
    }
}

// rust geo does not have the distinction LineString/LineRing, so we create a wrapper

struct LineRing<'a>(&'a LineString<f64>);

/// Convert a geo_types::LineString to a geos LinearRing
/// Empty LineRing are valid
/// The rules for validation and construction are those followed by
/// [Shapely](https://shapely.readthedocs.io/en/latest/manual.html#linearrings)
/// If the input LineString is not closed, then the resulting LineRing closes the construction
/// by copying the first point after the last, as seen in the diagram below.
///
///  [(0,0), (1,0), (1,1)] => [(0,0) , (1,0), (1,1), (0,0)]
///
///   1        2                  1,4      2
///   ■ ────── ■                  ■ ────── ■
///            │           =>     │        │
///            │                  │        │
///            ■ 3                └─────── ■ 3
///
///  There is a special case of a closed LineString of 3 points...
///  In that case, since it only contains 3 points, we consider it needs closing,
///  and we add a fourth point. See test closed_2_points_linear_ring below.
impl<'a, 'b> TryFrom<&'a LineRing<'b>> for GGeom<'b> {
    type Error = Error;

    fn try_from(linering: &'a LineRing<'b>) -> Result<GGeom<'b>, Self::Error> {
        let nb_points = linering.0.num_coords();
        if nb_points > 0 && nb_points < 3 {
            return Err(Error::InvalidGeometry(
                "impossible to create a LinearRing, A LinearRing must have at least 3 coordinates"
                    .into(),
            ));
        }

        if nb_points == 0 {
            let coords = CoordSeq::new(0, CoordDimensions::TwoD)?;
            return GGeom::create_linear_ring(coords);
        }

        let mut points = linering.0.points_iter();

        // The following expect is OK because we took care of the case where there is no points
        let first = points.next().expect("At least one point");

        // This expect is OK too, because if there is at least one point, there is at least 3 points
        // because of the constraint above.
        let last = points.last().expect("No last point");

        // if the geom is not closed we close it
        let is_closed = nb_points > 0 && first == last;

        // Note: we also need to close a 2 points closed linearring, cf test closed_2_points_linear_ring
        let need_closing = nb_points > 0 && (!is_closed || nb_points == 3);

        let coords = if need_closing {
            let mut coords = CoordSeq::new(nb_points as u32 + 1, CoordDimensions::TwoD)?;
            linering
                .0
                .points_iter()
                .enumerate()
                .try_for_each(|(i, p)| {
                    coords.set_x(i, p.x())?;
                    coords.set_y(i, p.y())?;
                    Ok(())
                })?;
            coords.set_x(nb_points, first.x())?;
            coords.set_y(nb_points, first.y())?;
            coords
        } else {
            let mut coords = CoordSeq::new(nb_points as u32, CoordDimensions::TwoD)?;
            linering
                .0
                .points_iter()
                .enumerate()
                .try_for_each(|(i, p)| {
                    coords.set_x(i, p.x())?;
                    coords.set_y(i, p.y())?;
                    Ok(())
                })?;
            coords
        };

        GGeom::create_linear_ring(coords)
    }
}

impl<'a> TryFrom<&'a Polygon<f64>> for GGeom<'a> {
    type Error = Error;

    fn try_from(polygon: &'a Polygon<f64>) -> Result<GGeom<'a>, Self::Error> {
        let ring = LineRing(polygon.exterior());
        let geom_exterior: GGeom = GGeom::try_from(&ring)?;

        let interiors: Vec<_> = polygon
            .interiors()
            .iter()
            .map(|i| GGeom::try_from(&LineRing(i)))
            .collect::<Result<Vec<_>, _>>()?;

        GGeom::create_polygon(geom_exterior, interiors)
    }
}

impl<'a> TryFrom<&'a MultiPolygon<f64>> for GGeom<'a> {
    type Error = Error;

    fn try_from(multipolygon: &'a MultiPolygon<f64>) -> Result<GGeom<'a>, Self::Error> {
        let polygons: Vec<_> = multipolygon
            .0
            .iter()
            .map(|p| GGeom::try_from(p))
            .collect::<Result<Vec<_>, _>>()?;

        GGeom::create_multipolygon(polygons)
    }
}

#[cfg(test)]
mod test {
    use super::GGeom;
    use super::LineRing;
    use geo_types::{Coordinate, LineString, MultiPolygon, Polygon};
    use std::convert::{TryFrom, TryInto};

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
        tuples.into_iter().map(Coordinate::from).collect()
    }

    #[test]
    fn polygon_contains_test() {
        let exterior = LineString(coords(vec![
            (0., 0.),
            (0., 1.),
            (1., 1.),
            (1., 0.),
            (0., 0.),
        ]));
        let interiors = vec![LineString(coords(vec![
            (0.1, 0.1),
            (0.1, 0.9),
            (0.9, 0.9),
            (0.9, 0.1),
            (0.1, 0.1),
        ]))];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior(), &exterior);
        assert_eq!(p.interiors(), interiors.as_slice());

        let geom: GGeom = (&p).try_into().unwrap();

        assert!(geom.contains(&geom).unwrap());
        assert!(!geom.contains(&(&exterior).try_into().unwrap()).unwrap());

        assert!(geom.covers(&(&exterior).try_into().unwrap()).unwrap());
        assert!(geom.touches(&(&exterior).try_into().unwrap()).unwrap());
    }

    #[test]
    fn multipolygon_contains_test() {
        let exterior = LineString(coords(vec![
            (0., 0.),
            (0., 1.),
            (1., 1.),
            (1., 0.),
            (0., 0.),
        ]));
        let interiors = vec![LineString(coords(vec![
            (0.1, 0.1),
            (0.1, 0.9),
            (0.9, 0.9),
            (0.9, 0.1),
            (0.1, 0.1),
        ]))];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p.clone()]);

        let geom: GGeom = (&mp).try_into().unwrap();

        assert!(geom.contains(&geom).unwrap());
        assert!(geom.contains(&(&p).try_into().unwrap()).unwrap());
    }

    #[test]
    fn incorrect_multipolygon_test() {
        let exterior = LineString(coords(vec![(0., 0.)]));
        let interiors = vec![];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p.clone()]);

        let geom = GGeom::try_from(&mp);

        assert!(geom.is_err());
    }

    #[test]
    fn incorrect_polygon_not_closed() {
        // even if the polygon is not closed we can convert it to geos (we close it)
        let exterior = LineString(coords(vec![
            (0., 0.),
            (0., 2.),
            (2., 2.),
            (2., 0.),
            (0., 0.),
        ]));
        let interiors = vec![LineString(coords(vec![
            (0., 0.),
            (0., 1.),
            (1., 1.),
            (1., 0.),
            (0., 10.),
        ]))];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p]);

        let _g = GGeom::try_from(&mp).unwrap(); // no error
    }

    /// a linear ring can be empty
    #[test]
    fn empty_linear_ring() {
        let ls = LineString(vec![]);
        let geom = GGeom::try_from(&LineRing(&ls)).unwrap();

        assert!(geom.is_valid());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 0);
    }

    /// a linear ring should have at least 3 elements
    #[test]
    fn one_elt_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.)]));
        let geom = GGeom::try_from(&LineRing(&ls));
        let error = geom.err().unwrap();
        assert_eq!(format!("{}", error), "Invalid geometry, impossible to create a LinearRing, A LinearRing must have at least 3 coordinates".to_string());
    }

    /// a linear ring should have at least 3 elements
    #[test]
    fn two_elt_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.)]));
        let geom = GGeom::try_from(&LineRing(&ls));
        let error = geom.err().unwrap();
        assert_eq!(format!("{}", error), "Invalid geometry, impossible to create a LinearRing, A LinearRing must have at least 3 coordinates".to_string());
    }

    /// an unclosed linearring is valid since we close it before giving it to geos
    #[test]
    fn unclosed_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.)]));
        let geom = GGeom::try_from(&LineRing(&ls)).unwrap();

        assert!(geom.is_valid());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }

    /// a tricky corner case: a closed linearstring of 3 points....
    ///
    /// Here we'll follow the behavior of Shapely, which adds a fourth point,
    /// as seen in the diagram below:
    ///
    ///   1,3      2                  1,3,4    2
    ///   ■ ────── ■                  ■ ────── ■
    ///
    ///  [(0,0), (1,0), (0,0)] => [(0,0) , (1,0), (0,0), (0,0)]
    ///
    #[test]
    fn closed_2_points_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (0., 0.)]));
        let geom = GGeom::try_from(&LineRing(&ls)).unwrap();

        assert!(geom.is_valid());
        assert!(geom.is_ring().expect("is_ring failed"));
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }

    /// a linear ring can be empty
    #[test]
    fn good_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.), (0., 0.)]));
        let geom = GGeom::try_from(&LineRing(&ls)).unwrap();

        assert!(geom.is_valid());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }
}
