extern crate geos;
use geos::{version, init, finish, CoordSeq, Point, LineString};

fn main() {
    init();
    println!("geos_c version: {}", version());
    let sequence = CoordSeq::new(1, 2);
    let ret_val = sequence.set_x(0, 12.36);
    println!("{:?}", ret_val);
    let ret_val = sequence.set_y(0, 43.21);
    println!("{:?}", ret_val);
    let geom_point = Point(&sequence);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_point.to_wkt());
    let sequence2 = CoordSeq::new(2, 2);
    let ret_val = sequence2.set_x(0, 12.36);
    println!("{:?}", ret_val);
    let ret_val = sequence2.set_y(0, 43.21);
    println!("{:?}", ret_val);
    let ret_val = sequence2.set_x(1, 12.78);
    println!("{:?}", ret_val);
    let ret_val = sequence2.set_y(1, 42.80);
    println!("{:?}", ret_val);
    let geom_line = LineString(&sequence2);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_line.to_wkt());
    finish();
}
