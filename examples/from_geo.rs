#[cfg(feature = "geo")]
extern crate geo_types;
extern crate geos;

#[cfg(feature = "geo")]
use geo_types::{Coordinate, LineString, Polygon};
#[cfg(feature = "geo")]
use geos::from_geo::TryInto;
#[cfg(feature = "geo")]
use geos::{Error, Geometry};

#[cfg(feature = "geo")]
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

    let geom: Geometry = (&p).try_into()?;

    assert!(geom.contains(&geom)?);
    assert!(!geom.contains(&(&exterior).try_into()?)?);

    assert!(geom.covers(&(&exterior).try_into()?)?);
    assert!(geom.touches(&(&exterior).try_into()?)?);
    Ok(())
}

#[cfg(feature = "geo")]
fn main() {
    fun().unwrap();
}


#[cfg(not(feature = "geo"))]
fn main() {
    eprintln!("You need to enable the \"geo\" feature to run this example!", );
}
