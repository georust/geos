use crate::error::Error;
use crate::{CoordSeq, CoordType, Geometry as GGeometry};
use geo_types::{
    Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};

use std;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};

fn create_coord_seq_from_vec(coords: &[Coord<f64>]) -> Result<CoordSeq, Error> {
    create_coord_seq(coords.iter(), coords.len())
}

fn create_coord_seq<'a, It>(points: It, len: usize) -> Result<CoordSeq, Error>
where
    It: Iterator<Item = &'a Coord<f64>>,
{
    let mut coord_seq =
        CoordSeq::new(len as u32, CoordType::XY).expect("failed to create CoordSeq");
    for (i, p) in points.enumerate() {
        coord_seq.set_x(i, p.x)?;
        coord_seq.set_y(i, p.y)?;
    }
    Ok(coord_seq)
}

impl TryFrom<&Point<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &Point<f64>) -> Result<GGeometry, Self::Error> {
        let coord_seq = create_coord_seq(std::iter::once(&other.0), 1)?;

        GGeometry::create_point(coord_seq)
    }
}

impl TryFrom<Point<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: Point<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl<T: Borrow<Point<f64>>> TryFrom<&[T]> for GGeometry {
    type Error = Error;

    fn try_from(other: &[T]) -> Result<GGeometry, Self::Error> {
        let geom_points = other
            .iter()
            .map(|p| p.borrow().try_into())
            .collect::<Result<Vec<_>, _>>()?;

        GGeometry::create_multipoint(geom_points)
    }
}

impl TryFrom<&MultiPoint<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &MultiPoint<f64>) -> Result<GGeometry, Self::Error> {
        let points: Vec<_> = other
            .0
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        GGeometry::create_multipoint(points)
    }
}

impl TryFrom<MultiPoint<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: MultiPoint<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl TryFrom<&LineString<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &LineString<f64>) -> Result<GGeometry, Self::Error> {
        let coord_seq = create_coord_seq_from_vec(other.0.as_slice())?;

        GGeometry::create_line_string(coord_seq)
    }
}

impl TryFrom<LineString<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: LineString<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl TryFrom<&MultiLineString<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &MultiLineString<f64>) -> Result<GGeometry, Self::Error> {
        let lines: Vec<_> = other
            .0
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        GGeometry::create_multiline_string(lines)
    }
}

impl TryFrom<MultiLineString<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: MultiLineString<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

// rust geo does not have the distinction LineString/LineRing, so we create a wrapper

struct LineRing<'a>(&'a LineString<f64>);

/// Convert a geo_types::LineString to a geos LinearRing
/// a LinearRing should be closed so cloase the geometry if needed
impl TryFrom<LineRing<'_>> for GGeometry {
    type Error = Error;

    fn try_from(other: LineRing<'_>) -> Result<GGeometry, Self::Error> {
        let points = &(other.0).0;
        let nb_points = points.len();
        if nb_points > 0 && nb_points < 3 {
            return Err(Error::ConversionError(
                "a LinearRing must have at least 3 coordinates".into(),
            ));
        }

        // if the geom is not closed we close it
        let is_closed = nb_points > 0 && points.first() == points.last();
        // Note: we also need to close a 2 points closed linearring, cf test closed_2_points_linear_ring
        let need_closing = nb_points > 0 && (!is_closed || nb_points == 3);
        let coord_seq = if need_closing {
            create_coord_seq(
                points.iter().chain(std::iter::once(&points[0])),
                nb_points + 1,
            )?
        } else {
            create_coord_seq(points.iter(), nb_points)?
        };
        GGeometry::create_linear_ring(coord_seq)
    }
}

impl TryFrom<&Polygon<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &Polygon<f64>) -> Result<GGeometry, Self::Error> {
        let ring = LineRing(other.exterior());
        let geom_exterior: GGeometry = ring.try_into()?;

        let interiors: Vec<_> = other
            .interiors()
            .iter()
            .map(|i| LineRing(i).try_into())
            .collect::<Result<_, _>>()?;

        GGeometry::create_polygon(geom_exterior, interiors)
    }
}

impl TryFrom<Polygon<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: Polygon<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl TryFrom<&MultiPolygon<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &MultiPolygon<f64>) -> Result<GGeometry, Self::Error> {
        let polygons: Vec<_> = other
            .0
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        GGeometry::create_multipolygon(polygons)
    }
}

impl TryFrom<MultiPolygon<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: MultiPolygon<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl TryFrom<&GeometryCollection<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &GeometryCollection<f64>) -> Result<GGeometry, Self::Error> {
        let geoms: Vec<_> = other
            .0
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        GGeometry::create_geometry_collection(geoms)
    }
}

impl TryFrom<GeometryCollection<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: GeometryCollection<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

impl TryFrom<&Geometry<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: &Geometry<f64>) -> Result<GGeometry, Self::Error> {
        match other {
            Geometry::Point(inner) => GGeometry::try_from(inner),
            Geometry::MultiPoint(inner) => GGeometry::try_from(inner),
            Geometry::LineString(inner) => GGeometry::try_from(inner),
            Geometry::MultiLineString(inner) => GGeometry::try_from(inner),
            Geometry::Polygon(inner) => GGeometry::try_from(inner),
            Geometry::MultiPolygon(inner) => GGeometry::try_from(inner),
            Geometry::GeometryCollection(inner) => GGeometry::try_from(inner),
            // GEOS has equivalents of the types below, but they aren't subclasses of geos::Geometry
            Geometry::Triangle(_) => Err(Error::ConversionError(
                "Cannot convert Triangle to GEOS Geometry".to_string(),
            )),
            Geometry::Rect(_) => Err(Error::ConversionError(
                "Cannot convert Rect to GEOS Geometry".to_string(),
            )),
            Geometry::Line(_) => Err(Error::ConversionError(
                "Cannot convert Line to GEOS Geometry".to_string(),
            )),
        }
    }
}

impl TryFrom<Geometry<f64>> for GGeometry {
    type Error = Error;

    fn try_from(other: Geometry<f64>) -> Result<GGeometry, Self::Error> {
        GGeometry::try_from(&other)
    }
}

#[cfg(test)]
mod test {
    use super::LineRing;
    use crate::{Geom, Geometry as GGeometry};
    use geo_types::{
        Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon, Triangle,
    };
    use std::convert::TryInto;

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coord<f64>> {
        tuples.into_iter().map(Coord::from).collect()
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

        let geom: GGeometry = p.try_into().unwrap();

        assert!(geom.contains(&geom).unwrap());

        let tmp: GGeometry = exterior.try_into().unwrap();

        assert!(!geom.contains(&tmp).unwrap());
        assert!(geom.covers(&tmp).unwrap());
        assert!(geom.touches(&tmp).unwrap());
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

        let geom: GGeometry = (&mp).try_into().unwrap();

        assert!(geom.contains(&geom).unwrap());
        assert!(geom
            .contains::<GGeometry>(&(&p).try_into().unwrap())
            .unwrap());
    }

    #[test]
    fn incorrect_multipolygon_test() {
        let exterior = LineString(coords(vec![(0., 0.)]));
        let interiors = vec![];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p.clone()]);

        let geom: Result<GGeometry, _> = mp.try_into();

        assert!(geom.is_err());
    }

    #[test]
    fn geometry_collection_test() {
        let a = Polygon::new(
            LineString(coords(vec![
                (0., 0.),
                (0., 1.),
                (1., 1.),
                (1., 0.),
                (0., 0.),
            ])),
            vec![],
        );
        let b = Polygon::new(
            LineString(coords(vec![
                (2., 1.),
                (2., 2.),
                (5., 2.),
                (5., 1.),
                (2., 1.),
            ])),
            vec![],
        );

        let collection = GeometryCollection::new_from(vec![
            Geometry::Polygon(a.clone()),
            Geometry::Polygon(b.clone()),
        ]);

        let geos_collection: GGeometry = collection.try_into().unwrap();
        let geos_polygon_a: GGeometry = a.try_into().unwrap();
        let geos_polygon_b: GGeometry = b.try_into().unwrap();

        assert!(geos_collection.contains(&geos_polygon_a).unwrap());
        assert!(geos_collection.contains(&geos_polygon_b).unwrap());
    }

    #[test]
    fn unsupported_geometry_type_test() {
        let tri = Triangle::new(
            Coord::from((0., 0.)),
            Coord::from((3., 5.)),
            Coord::from((3., 0.)),
        );
        assert!(GGeometry::try_from(Geometry::Triangle(tri)).is_err());
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

        let _g: GGeometry = mp.try_into().unwrap(); // no error
    }

    /// a linear ring can be empty
    #[test]
    fn empty_linear_ring() {
        let ls = LineString(vec![]);
        let geom: GGeometry = LineRing(&ls).try_into().unwrap();

        assert!(geom.is_valid().unwrap());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 0);
    }

    /// a linear ring should have at least 3 elements
    #[test]
    fn one_elt_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.)]));
        let geom: Result<GGeometry, _> = LineRing(&ls).try_into();
        let error = geom.err().unwrap();
        assert_eq!(
            error.to_string(),
            "impossible to convert geometry: a LinearRing must have at least 3 coordinates"
                .to_string()
        );
    }

    /// a linear ring should have at least 3 elements
    #[test]
    fn two_elt_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.)]));
        let geom: Result<GGeometry, _> = LineRing(&ls).try_into();
        let error = geom.err().unwrap();
        assert_eq!(
            error.to_string(),
            "impossible to convert geometry: a LinearRing must have at least 3 coordinates"
                .to_string()
        );
    }

    /// an unclosed linearring is valid since we close it before giving it to geos
    #[test]
    fn unclosed_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.)]));
        let geom: GGeometry = LineRing(&ls).try_into().unwrap();

        assert!(geom.is_valid().unwrap());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }

    /// a bit tricky
    /// a ring should have at least 3 points.
    /// in the case of a closed ring with only element eg:
    ///
    /// let's take a point list: [p1, p2, p1]
    ///
    /// p1 ----- p2
    ///  ^-------|
    ///
    /// we consider it like a 3 points not closed ring (with the 2 last elements being equals...)
    ///
    /// shapely (the python geos wrapper) considers that too
    #[test]
    fn closed_2_points_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 1.)]));
        let geom: GGeometry = LineRing(&ls).try_into().unwrap();

        assert!(geom.is_valid().unwrap());
        assert!(geom.is_ring().expect("is_ring failed"));
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }

    /// a linear ring can be empty
    #[test]
    fn good_linear_ring() {
        let ls = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.), (0., 0.)]));
        let geom: GGeometry = LineRing(&ls).try_into().unwrap();

        assert!(geom.is_valid().unwrap());
        assert!(geom.is_ring().unwrap());
        assert_eq!(geom.get_coord_seq().unwrap().size().unwrap(), 4);
    }

    #[test]
    fn test_conversion_multilinestring() {
        let ls1 = LineString(coords(vec![(0., 0.), (0., 1.), (1., 2.)]));
        let ls2 = LineString(coords(vec![(2., 2.), (3., 3.), (3., 2.)]));
        let geom: GGeometry = MultiLineString(vec![ls1, ls2]).try_into().unwrap();
        assert!(geom.is_valid().unwrap());
    }

    #[test]
    fn test_conversion_multipoint() {
        let p1 = Point::new(0., 0.);
        let p2 = Point::new(0., 1.);
        let p3 = Point::new(1., 2.);
        let geom: GGeometry = MultiPoint(vec![p1, p2, p3]).try_into().unwrap();
        assert!(geom.is_valid().unwrap());
    }
}
