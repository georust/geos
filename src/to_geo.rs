use error::Error;
use ffi::GGeom;
use from_geo::TryInto;
use geo_types::Geometry;
use wkt;
use wkt::conversion::try_into_geometry;

impl<'a> TryInto<Geometry<f64>> for GGeom {
    type Err = Error;

    fn try_into(self) -> Result<Geometry<f64>, Self::Err> {
        let wkt_str = self.to_wkt();
        let wkt_obj = wkt::Wkt::from_str(&wkt_str)
            .map_err(|e| Error::ConversionError(format!("impossible to read wkt: {}", e)))?;

        let o: &wkt::Geometry = wkt_obj.items.iter().next().ok_or(Error::ConversionError("invalid wkt".into()))?;

        try_into_geometry(o)
            .map_err(|e| Error::ConversionError(format!("impossible to built from wkt: {}", e)))
    }
}

#[cfg(test)]
mod test {
    use ffi::GGeom;
    use from_geo::TryInto;
    use geo_types::{Geometry, LineString, MultiPolygon, Point, Polygon};

    #[test]
    fn geom_to_geo_polygon() {
        let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
        let poly = GGeom::new(poly).unwrap();

        let geo_polygon: Geometry<f64> = poly.try_into().unwrap();

        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 1.),
            Point::new(1., 1.),
            Point::new(1., 0.),
            Point::new(0., 0.),
        ]);
        let expected_poly = MultiPolygon(vec![Polygon::new(exterior, vec![])]);
        let expected: Geometry<_> = expected_poly.into();
        assert_eq!(expected, geo_polygon);
    }
}
