use crate::{ConstGeometry, Geom, Geometry as GGeometry};
use error::Error;
use geo_types::Geometry;
use wkt;
use wkt::conversion::try_into_geometry;

use std::convert::TryFrom;

macro_rules! impl_try_into {
    ($ty_name:ident $(,$lt:lifetime)?) => (
impl<'a$(,$lt)?> TryFrom<$ty_name<'a$(,$lt)?>> for Geometry<f64> {
    type Error = Error;

    fn try_from(other: $ty_name<'a$(,$lt)?>) -> Result<Geometry<f64>, Self::Error> {
        // This is a first draft, it's very inefficient, we use wkt as a pivot format to
        // translate the geometry.
        // We should at least use wkb, or even better implement a direct translation
        let wkt_str = other.to_wkt()?;
        let wkt_obj = wkt::Wkt::from_str(&wkt_str)
            .map_err(|e| Error::ConversionError(format!("impossible to read wkt: {}", e)))?;

        let o: &wkt::Geometry<f64> = wkt_obj
            .items
            .iter()
            .next()
            .ok_or(Error::ConversionError("invalid wkt".into()))?;

        try_into_geometry(o)
            .map_err(|e| Error::ConversionError(format!("impossible to built from wkt: {}", e)))
    }
}
    );
}

impl_try_into!(GGeometry);
impl_try_into!(ConstGeometry, 'c);

#[cfg(test)]
mod test {
    use crate::Geometry as GGeometry;
    use geo_types::{Coordinate, Geometry, LineString, MultiPolygon, Polygon};
    use std::convert::TryInto;

    fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
        tuples.into_iter().map(Coordinate::from).collect()
    }

    #[test]
    fn geom_to_geo_polygon() {
        let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
        let poly = GGeometry::new_from_wkt(poly).unwrap();

        let geo_polygon: Geometry<f64> = poly.try_into().unwrap();

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
    }
}
