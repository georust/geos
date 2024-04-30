use crate::error::Error;
use crate::{ConstGeometry, Geom, Geometry as GGeometry};
use geo_types::Geometry;
use wkt;
use wkt::TryFromWkt;

use std::convert::TryFrom;

macro_rules! impl_try_into {
    ($ty_name:ident $(,$lt:lifetime)?) => (
impl<'b$(,$lt)?> TryFrom<&'b $ty_name$(<$lt>)?> for Geometry<f64> {
    type Error = Error;

    fn try_from(other: &'b $ty_name$(<$lt>)?) -> Result<Geometry<f64>, Self::Error> {
        // This is a first draft, it's very inefficient, we use wkt as a pivot format to
        // translate the geometry.
        // We should at least use wkb, or even better implement a direct translation
        let wkt_str = other.to_wkt()?;
        geo_types::Geometry::try_from_wkt_str(&wkt_str)
            .map_err(|e| Error::ConversionError(format!("impossible to read wkt: {}", e)))
    }
}
impl$(<$lt>)? TryFrom<$ty_name$(<$lt>)?> for Geometry<f64> {
    type Error = Error;

    fn try_from(other: $ty_name$(<$lt>)?) -> Result<Geometry<f64>, Self::Error> {
        Geometry::try_from(&other)
    }
}
    );
}

impl_try_into!(GGeometry);
impl_try_into!(ConstGeometry, 'c);

#[cfg(test)]
mod test {
    use crate::Geometry as GGeometry;
    use geo_types::{Coordinate, Geometry, LineString, MultiPoint, MultiPolygon, Point, Polygon};
    use std::convert::TryInto;

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
        tuples.into_iter().map(Coordinate::from).collect()
    }

    #[test]
    fn geom_to_geo_polygon() {
        let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
        let poly = GGeometry::new_from_wkt(poly).unwrap();

        let geo_polygon: Geometry<f64> = (&poly).try_into().unwrap();

        let exterior = LineString(coords(vec![
            (0., 0.),
            (0., 1.),
            (1., 1.),
            (1., 0.),
            (0., 0.),
        ]));
        let expected_poly = MultiPolygon(vec![Polygon::new(exterior, vec![])]);
        let expected: Geometry<_> = expected_poly.into();
        assert_eq!(expected, geo_polygon);
        // This check is to enforce that `TryFrom` is implemented for both reference and value.
        assert_eq!(expected, poly.try_into().unwrap());
    }

    #[test]
    fn geom_to_geo_multipoint() {
        let mp = "MULTIPOINT (0 0, 1 1)";
        let mp = GGeometry::new_from_wkt(mp).unwrap();

        let geo_multipoint: Geometry<f64> = (&mp).try_into().unwrap();

        let expected_multipoint = MultiPoint(vec![
            Point(Coordinate::from((0., 0.))),
            Point(Coordinate::from((1., 1.))),
        ]);
        let expected: Geometry<_> = expected_multipoint.into();
        assert_eq!(expected, geo_multipoint);
        // This check is to enforce that `TryFrom` is implemented for both reference and value.
        assert_eq!(expected, mp.try_into().unwrap());
    }
}
