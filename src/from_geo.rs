extern crate geo;

use libc::{c_int, c_uint};
use self::geo::{LineString, MultiPolygon, Polygon};
use ffi::{CoordSeq, GEOSGeomTypes, GEOSGeom_clone, GEOSGeom_createCollection,
          GEOSGeom_createPolygon, GGeom, _linearRing};
use std::convert::From;

impl<'a> From<&'a LineString<f64>> for GGeom {
    fn from(ls: &LineString<f64>) -> Self {
        let nb_pts = ls.0.len();
        let coord_seq_ext = CoordSeq::new(nb_pts as u32, 2);
        for i in 0..nb_pts {
            let j = i as u32;
            coord_seq_ext.set_x(j, ls.0[i].x());
            coord_seq_ext.set_y(j, ls.0[i].y());
        }
        _linearRing(&coord_seq_ext)
    }
}

impl<'a> From<&'a Polygon<f64>> for GGeom {
    fn from(p: &Polygon<f64>) -> Self {
        let geom_exterior: GGeom = (&p.exterior).into();
        let nb_interiors = p.interiors.len();

        let interiors: Vec<_> = p.interiors
            .iter()
            .map(|i| i.into())
            .map(|i: GGeom| unsafe { GEOSGeom_clone(i.c_obj) })
            .collect();

        let t = unsafe {
            GEOSGeom_createPolygon(
                GEOSGeom_clone(geom_exterior.c_obj),
                &interiors[..],
                nb_interiors as c_uint,
            )
        };
        GGeom::new_from_c_obj(t)
    }
}

impl<'a> From<&'a MultiPolygon<f64>> for GGeom {
    fn from(mp: &MultiPolygon<f64>) -> Self {
        let nb_polygons = mp.0.len();
        let polygons: Vec<_> = mp.0
            .iter()
            .map(|p| p.into())
            .map(|g: GGeom| unsafe { GEOSGeom_clone(g.c_obj) })
            .collect();

        let t = unsafe {
            GEOSGeom_createCollection(
                GEOSGeomTypes::GEOS_MULTIPOLYGON as c_int,
                &polygons[..],
                nb_polygons as c_uint,
            )
        };
        GGeom::new_from_c_obj(t)
    }
}

#[cfg(test)]
mod test {
    use from_geo::geo::{LineString, MultiPolygon, Point, Polygon};
    use ffi::GGeom;

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

        let geom: GGeom = (&p).into();

        assert!(geom.contains(&geom));
        assert!(!geom.contains(&(&exterior).into()));

        assert!(geom.covers((&(&exterior).into())));
        assert!(geom.touches(&(&exterior).into()));
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
        let p = Polygon::new(exterior.clone(), interiors.clone());
        let mp = MultiPolygon(vec![p.clone()]);

        let geom: GGeom = (&mp).into();

        assert!(geom.contains(&geom));
        assert!(geom.contains(&(&p).into()));
    }
}
