geos
====

[![Build Status](https://travis-ci.org/georust/geos.svg?branch=master)](https://travis-ci.org/georust/geos)

Rust bindings for [GEOS](https://trac.osgeo.org/geos/) C API.

### Disclaimer

Work in progress (currently it's probably poorly designed, incomplete and containing beginners errors)

GEOS can be a tad strict on the validity on the input geometry and is prone to crash on invalid input, so they need to be checked in the wrapper.
This project is checked with valgrind, but if you stumble on a crash feel free to open an issue explaining the problem.

### Usage example

You can check the examples in the `examples/` directory.

### Constructing geometries from WKT:

```rust,skt-template
let gg1 = geos::GGeom::new("POLYGON ((0 0, 0 5, 6 6, 6 0, 0 0))")?;
let gg2 = geos::GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 1, 1 1))")?;
let gg3 = gg1.difference(&gg2)?;
assert_eq!(
  gg3.to_wkt_precision(Some(0)),
  "POLYGON ((0 0, 0 5, 6 6, 6 0, 0 0), (1 1, 5 1, 5 5, 1 3, 1 1))");

```


### "Preparing" the geometries for faster predicates (intersects, contains, etc.) computation on repetitive calls:

```rust,skt-template
let g1 = geos::GGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))")?;
let g2 = geos::GGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))")?;

let pg1 = geos::PreparedGGeom::new(&g1);
let result = pg1.intersects(&g2)?;
assert_eq!(result, true);
```

### Conversion from [rust-geo](https://github.com/georust/rust-geo)

[rust-geo](https://github.com/georust/rust-geo)'s objects can be converted into [GEOS](https://trac.osgeo.org/geos/)
to use all geos algorithms.

Complete example can be found in `examples/from_geo.rs`

```rust,skt-template
use geos::from_geo::TryInto;
use geo_types::{LineString, Point, Polygon};

// first we create a Geo object
let exterior = LineString(vec![
    Point::new(0., 0.),
    Point::new(0., 1.),
    Point::new(1., 1.),
]);
let interiors = vec![
    LineString(vec![
        Point::new(0.1, 0.1),
        Point::new(0.1, 0.9),
        Point::new(0.9, 0.9),
    ]),
];
let p = Polygon::new(exterior, interiors);
// and we can create a Geos geometry from this object
let _geom: geos::GGeom = (&p).try_into()?;
// do some stuff with _geom
```

## Contributing

Only a subset of geos has been implemented, feel free to add wrappers for missing features.

All added features needs to be tested and this being a C wrapper, valgrind runs on all examples/tests to check that
no bugs / memory leaks are lurking.
