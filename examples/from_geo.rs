extern crate geos;
extern crate geo;
extern crate failure;
use geos::GGeom;
use geo::{LineString, Point, Polygon};
use geos::from_geo::TryInto;
use failure::Error;

fn fun() -> Result<(), Error> {
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

    let geom: GGeom = (&p).try_into()?;

    assert!(geom.contains(&geom)?);
    assert!(!geom.contains(&(&exterior).try_into()?)?);

    assert!(geom.covers(&(&exterior).try_into()?)?);
    assert!(geom.touches(&(&exterior).try_into()?)?);
    Ok(())
}

fn main() {
    fun().unwrap();
}
