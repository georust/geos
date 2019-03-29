extern crate geo_types;
extern crate geos;

use geo_types::{Coordinate, LineString, Polygon};
use geos::from_geo::TryInto;
use geos::{Error, GGeom};

fn fun() -> Result<(), Error> {
    let exterior = LineString(vec![
        Coordinate::from((0., 0.)),
        Coordinate::from((0., 1.)),
        Coordinate::from((1., 1.)),
        Coordinate::from((1., 0.)),
        Coordinate::from((0., 0.)),
    ]);
    let interiors = vec![LineString(vec![
        Coordinate::from((0.1, 0.1)),
        Coordinate::from((0.1, 0.9)),
        Coordinate::from((0.9, 0.9)),
        Coordinate::from((0.9, 0.1)),
        Coordinate::from((0.1, 0.1)),
    ])];
    let p = Polygon::new(exterior.clone(), interiors.clone());

    assert_eq!(p.exterior(), &exterior);
    assert_eq!(p.interiors(), interiors.as_slice());

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
