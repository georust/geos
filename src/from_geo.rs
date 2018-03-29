extern crate geo;

use self::geo::{LineString, MultiPolygon, Polygon, Point};
use ffi::{CoordSeq, GGeom};
use error::Error;

// define our own TryInto while the std trait is not stable
pub trait TryInto<T> {
    type Err;
    fn try_into(self) -> Result<T, Self::Err>;
}

fn create_coord_seq<'a>(points: &'a Vec<Point<f64>>) -> CoordSeq {
    let nb_pts = points.len();
    let coord_seq = CoordSeq::new(nb_pts as u32, 2);
    for i in 0..nb_pts {
        let j = i as u32;
        coord_seq.set_x(j, points[i].x());
        coord_seq.set_y(j, points[i].y());
    }
    coord_seq
}

impl<'a> TryInto<GGeom> for &'a LineString<f64> {
    type Err = Error;

    fn try_into(self) -> Result<GGeom, Self::Err> {
        let coord_seq = create_coord_seq(&self.0);

        Ok(GGeom::create_line_string(coord_seq))
    }
}

// rust geo does not have the distinction LineString/LineRing, so we create a wrapper 

struct LineRing<'a>(&'a LineString<f64>);

impl<'a> TryInto<GGeom> for &'a LineRing<'a> {
    type Err = Error;

    fn try_into(self) -> Result<GGeom, Self::Err> {
        let points = &(self.0).0;
        let coord_seq = create_coord_seq(&points);

        if points.len() == 1 {
            Err(Error::InvalidGeometry("impossible to create a linering from one point".into()))
        } else if points.len() > 1 && points.first() != points.last() {
            // the linestring need to be closed else geos will crash
            Err(Error::InvalidGeometry("impossible to create a linering with an unclosed geometry".into()))
        } else {
            Ok(GGeom::create_linear_ring(coord_seq))
        }
    }
}

impl<'a> TryInto<GGeom> for &'a Polygon<f64> {
    type Err = Error;

    fn try_into(self) -> Result<GGeom, Self::Err> {
        let geom_exterior: GGeom = LineRing(&self.exterior).try_into()?;

        let interiors: Vec<_> = self.interiors
            .iter()
            .map(|i| LineRing(i).try_into())
            .collect::<Result<Vec<_>, _>>()?;

        GGeom::create_polygon(geom_exterior, interiors)
    }
}

impl<'a> TryInto<GGeom> for &'a MultiPolygon<f64> {
    type Err = Error;

    fn try_into(self) -> Result<GGeom, Self::Err> {
        let polygons: Vec<_> = self.0
            .iter()
            .map(|p| p.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        GGeom::create_multipolygon(polygons)
    }
}

#[cfg(test)]
mod test {
    use from_geo::geo::{LineString, MultiPolygon, Point, Polygon};
    use ffi::GGeom;
    use from_geo::TryInto;

    #[test]
    fn polygon_contains_test() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 1.),
            Point::new(1., 1.),
            Point::new(1., 0.),
            Point::new(0., 0.),
        ]);
        let interiors = vec![
            LineString(vec![
                Point::new(0.1, 0.1),
                Point::new(0.1, 0.9),
                Point::new(0.9, 0.9),
                Point::new(0.9, 0.1),
                Point::new(0.1, 0.1),
            ]),
        ];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior, exterior);
        assert_eq!(p.interiors, interiors);

        let geom: GGeom = (&p).try_into().unwrap();

        assert!(geom.contains(&geom));
        assert!(!geom.contains(&(&exterior).try_into().unwrap()));

        assert!(geom.covers((&(&exterior).try_into().unwrap())));
        assert!(geom.touches(&(&exterior).try_into().unwrap()));
    }

    #[test]
    fn multipolygon_contains_test() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 1.),
            Point::new(1., 1.),
            Point::new(1., 0.),
            Point::new(0., 0.),
        ]);
        let interiors = vec![
            LineString(vec![
                Point::new(0.1, 0.1),
                Point::new(0.1, 0.9),
                Point::new(0.9, 0.9),
                Point::new(0.9, 0.1),
                Point::new(0.1, 0.1),
            ]),
        ];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p.clone()]);

        let geom: GGeom = (&mp).try_into().unwrap();

        assert!(geom.contains(&geom));
        assert!(geom.contains(&(&p).try_into().unwrap()));
    }

    #[test]
    fn incorrect_multipolygon_test() {
        let exterior = LineString(vec![
            Point::new(0., 0.)
        ]);
        let interiors = vec![];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p.clone()]);

        let geom = (&mp).try_into();

        assert!(geom.is_err());
    }    
    
    #[test]
    fn incorrect_polygon_not_closed() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 2.),
            Point::new(2., 2.),
            Point::new(2., 0.),
            Point::new(0., 0.),
        ]);
        let interiors = vec![
            LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 1.),
            Point::new(1., 1.),
            Point::new(1., 0.),
            Point::new(0., 10.),
            ]),
        ];
        let p = Polygon::new(exterior, interiors);
        let mp = MultiPolygon(vec![p]);

        let geom = (&mp).try_into();
        let error = geom.err().unwrap();

        assert_eq!(format!("{}", error), "Invalid geometry, impossible to create a linering with an unclosed geometry".to_string());
    }
}
