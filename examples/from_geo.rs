#[cfg(feature = "geo")]
extern crate geo_types;
extern crate geos;

#[cfg(feature = "geo")]
use geo_types::{Coord, LineString, Polygon};
#[cfg(feature = "geo")]
use geos::{Error, Geom, Geometry};
#[cfg(feature = "geo")]
use std::convert::TryInto;

#[cfg(feature = "geo")]
fn fun() -> Result<(), Error> {
    let exterior = LineString(vec![
        Coord::from((0., 0.)),
        Coord::from((0., 1.)),
        Coord::from((1., 1.)),
        Coord::from((1., 0.)),
        Coord::from((0., 0.)),
    ]);
    let interiors = vec![LineString(vec![
        Coord::from((0.1, 0.1)),
        Coord::from((0.1, 0.9)),
        Coord::from((0.9, 0.9)),
        Coord::from((0.9, 0.1)),
        Coord::from((0.1, 0.1)),
    ])];
    let p = Polygon::new(exterior.clone(), interiors.clone());

    assert_eq!(p.exterior(), &exterior);
    assert_eq!(p.interiors(), interiors.as_slice());

    let geom: Geometry = (&p).try_into()?;

    assert!(geom.contains(&geom)?);
    let tmp: Geometry = (&exterior).try_into()?;
    assert!(!geom.contains(&tmp)?);

    assert!(geom.covers(&tmp)?);
    assert!(geom.touches(&tmp)?);
    Ok(())
}

#[cfg(feature = "geo")]
fn main() {
    fun().unwrap();
}

#[cfg(not(feature = "geo"))]
fn main() {
    eprintln!("You need to enable the \"geo\" feature to run this example!",);
}
