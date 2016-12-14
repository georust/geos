#![cfg(not(test))]
extern crate geos;
use geos::{version, init, finish, gg};

fn main() {
    init();
    println!("geos_c version: {}", version());
    let g1 = gg::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
    print!("Geometry 1 created" );
    let a = g1.Area();
    println!("Area : {:?}", a);
    let l = g1.Length();
    println!("Length : {:?}", l);
    let g2 = gg::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))");
    println!("Geometry 2 created");
    let result = g1.Intersects(&g2);
    println!("Geom1 intersects geom2 : {:?}\n", result);
    let g3 = g1.Buffer(-100.0, 8);
    println!("Previous area = {} \nNew area = {}", g2.Area(), g3.Area());
    let result = g1.Within(&g2);
    println!("Geom1 within geom2 : {:?}\n", result);
    println!("Geom1 to wkt : {:?}", g1.toWKT());
    println!("Geom2 to wkt : {:?}", g2.toWKT());
    println!("Geom3 to wkt : {:?}", g3.toWKT());
    finish();
}
