# geos

Rust bindings for [GEOS](https://libgeos.org/) C API.

The supported geos version is >= 3.5

### Disclaimer

GEOS can be a tad strict on the validity on the input geometry and is prone to crash on invalid input, so they need to be checked in the wrapper.
This project is checked with valgrind, but if you stumble on a crash feel free to open an issue explaining the problem.

### Usage example

You can check the examples in the `examples/` directory.

### Constructing geometries from WKT:

```rust
use geos::Geom;

let gg1 = geos::Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 6 6, 6 0, 0 0))")
                         .expect("invalid WKT");
let gg2 = geos::Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 1, 1 1))")
                         .expect("invalid WKT");
let mut gg3 = gg1.difference(&gg2).expect("difference failed");
// normalize is only used for consistent ordering of vertices
gg3.normalize().expect("normalize failed");
assert_eq!(
    gg3.to_wkt_precision(0).expect("to_wkt failed"),
    "POLYGON ((0 0, 0 5, 6 6, 6 0, 0 0), (1 1, 5 1, 5 5, 1 3, 1 1))",
);
```

### "Preparing" the geometries for faster predicates (intersects, contains, etc.) computation on repetitive calls:

```rust
let g1 = geos::Geometry::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))")
                        .expect("invalid WKT");
let g2 = geos::Geometry::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))")
                        .expect("invalid WKT");

let pg1 = geos::PreparedGeometry::new(&g1)
                                 .expect("PreparedGeometry::new failed");
let result = pg1.intersects(&g2).expect("intersects failed");
assert_eq!(result, true);
```

### Conversion from [geo](https://github.com/georust/geo)

[geo](https://github.com/georust/geo)'s objects can be converted into [GEOS](https://libgeos.org/)
to use all geos algorithms.

Complete example can be found in `examples/from_geo.rs`

```rust,ignore
use geos::geo_types::{LineString, Coordinate, Polygon};

// first we create a Geo object
let exterior = LineString(vec![
    Coordinate::from((0., 0.)),
    Coordinate::from((0., 1.)),
    Coordinate::from((1., 1.)),
]);
let interiors = vec![
    LineString(vec![
        Coordinate::from((0.1, 0.1)),
        Coordinate::from((0.1, 0.9)),
        Coordinate::from((0.9, 0.9)),
    ]),
];
let p = Polygon::new(exterior, interiors);
// and we can create a Geos geometry from this object
let geom: geos::Geometry = (&p).try_into()
                               .expect("failed conversion");
// do some stuff with geom
```

### Voronoi

[Voronoi](https://en.wikipedia.org/wiki/Voronoi_diagram) diagrams computation are available in the bindings.

For those to be easier to use with [geo](https://github.com/georust/geo) some helpers are available in `voronoi.rs`.

```rust,ignore
use geos::compute_voronoi;
use geos::geo_types::Point;

let points = vec![
    Point::new(0., 0.),
    Point::new(0., 1.),
    Point::new(1., 1.),
    Point::new(1., 0.),
];

let voronoi = compute_voronoi(&points, None, 0., false)
                  .expect("compute_voronoi failed");
```

## Static build

By default, this crate links dynamically to your system-installed GEOS or a
different version that you configure. See [sys/README.md](./sys/README.md) for
more information.

If you want to link GEOS statically, use the `static` feature.

The static build uses the GEOS version in the git submodule in`sys/geos-src/source`.
This is currently GEOS 3.11.2.

You will need to have a build environment supported for the static version of
GEOS. See [GEOS build instructions](https://libgeos.org/usage/download/#build-from-source)
for more information but please note that instructions may be for a newer
version of GEOS than is currently included in the static build.

## Contributing

Only a subset of geos has been implemented, feel free to add wrappers for missing features.

All added features needs to be tested and this being a C wrapper, valgrind runs on all examples/tests to check that
no bugs / memory leaks are lurking.
