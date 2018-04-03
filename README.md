rust-geos
=========

[![Build Status](https://travis-ci.org/georust/rust-geos.svg?branch=master)](https://travis-ci.org/georust/rust-geos)

Rust bindings for [GEOS](https://trac.osgeo.org/geos/) C API.

## Disclaimer

Work in progress (currently it's probably poorly designed, incomplete and containing beginners errors)

GEOS can be a tad strict on the validity on the input geometry and is prone to crash on invalid input, so they need to be checked in the wrapper.
This project is checked with valgrind, but if you stumble on a crash feel free to open an issue explaining the problem.

### Usage example

### Constructing geometries from WKT:

```rust
extern crate geos;
use geos::GGeom;

fn main() {
    let gg1 = GGeom::new("POLYGON ((0 0, 0 5, 6 6, 6 0, 0 0))");
    let gg2 = GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 1, 1 1))");
    let gg3 = gg1.difference(&gg2);
    println!("{:?}", gg3.to_wkt());
}
```

### Constructing geometries from coordinates:

```rust
extern crate geos;
// Theses convenience methods returns the same GGeom instances as in the previous example :
use geos::types_geom::{Point, LineString, Polygon};

fn main(){
    let pt = Point::new((22.33, 44.55));
    println!("{:?}", pt.to_wkt());

    let l_geom = LineString::new(&[(12.78, 78.08), (55.77, 77.55), (22.77, 88.99)]);
    println!("GeosGeom Linestring from coordinates : {:?}", l_geom.to_wkt());

    let exterior_ring = Ring::new(&[(0.0, 0.0), (0.0, 8.0), (8.0, 8.0), (8.0, 0.0), (0.0, 0.0)]);
    let interior = Ring::new(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)]);
    let poly_geom = Polygon::new(&exterior_ring, &[interior]);
    println!("GeosGeom Polygon from ring coordinates : {:?}", poly_geom.to_wkt());

    assert!(!poly_geom.contains(&pt));
    assert!(!l_geom.intersects(&poly_geom));
    // The underlying CoordinateSequence of point(s) can also be fetched :
    let coord_seq = pt.get_coord_seq().unwrap();
    let mut x = coord_seq.get_x(0);
    let mut y = coord_seq.get_y(0);
    assert_eq!(x, 22.33);
    assert_eq!(y, 44.55);
}

```

### "Preparing" the geometries for faster predicates (intersects, contains, etc.) computation on repetitive calls:

```rust
extern crate geos;
use geos::{version, GGeom, PreparedGGeom};

fn main() {
    let g1 = GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
    let g2 = GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))");


    let pg1 = PreparedGGeom::new(&g1);
    let result = pg1.intersects(&g2);
    assert_eq!(result, true);

    let vec_geoms = vec![
        GGeom::new("POINT (1.3 2.4)"),
        GGeom::new("POINT (2.1 0.3)"),
        GGeom::new("POINT (3.1 4.7)"),
        GGeom::new("POINT (0.4 4.1)")
    ];
    for geom in &vec_geoms {
        if pg1.intersects(&geom) {
            // do some stuff
        }
    }
}
```

### Conversion from [rust-geo](https://github.com/georust/rust-geo)

[rust-geo](https://github.com/georust/rust-geo)'s objects can be converted into [GEOS](https://trac.osgeo.org/geos/)
to use all geos algorithms.

```rust
extern crate geos;
extern crate geo;
use geos::GGeom;
use geo::{LineString, Point, Polygon};
use geos::from_geo::TryInto;

fn main() {
    // first we create a Geo object
    let exterior = LineString(vec![
        Point::new(0., 0.),
        Point::new(0., 1.),
        Point::new(0., 0.),
    ]);
    let interiors = vec![
        LineString(vec![
            Point::new(0.1, 0.1),
            Point::new(0.1, 0.9),
            Point::new(0.1, 0.1),
        ]),
    ];
    let p = Polygon::new(exterior, interiors);
    // and we can create a Geos geometry from this object
    let geom: GGeom = (&p).try_into().unwrap();
    // do some stuff with geom
}
```

## Contributing

Only a subset of geos has been implemented, feel free to add wrappers for missing features.

All added features needs to be tested and this being a C wrapper, valgrind runs on all examples/tests to check that
no bugs / memory leaks are lurking.
