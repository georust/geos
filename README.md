rust-geos
=========

[![Build Status](https://travis-ci.org/mthh/rust-geos.svg?branch=master)](https://travis-ci.org/mthh/rust-geos)  

Rust bindings for [GEOS](https://trac.osgeo.org/geos/) C API.  
Work in progress (currently it's probably poorly designed, incomplete and containing beginners errors)  


##### Usage example #####

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
