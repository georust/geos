extern crate geos;
use geos::{version, GGeom, CoordSeq, _Point, _LineString};
use std::ffi::{CString, CStr};

fn Point(pt: (f64, f64)) -> GGeom {
    let sequence = CoordSeq::new(1, 2);
    sequence.set_x(0, pt.0);
    sequence.set_y(0, pt.1);
    _Point(&sequence).clone()
}

fn main() {
    println!("geos_c version: {}", version());
    let sequence = CoordSeq::new(1, 2);
    let ret_val = sequence.set_x(0, 12.36);
    println!("{:?}", ret_val);
    let ret_val = sequence.set_y(0, 43.21);
    println!("{:?}", ret_val);
    let geom_point = _Point(&sequence);
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
    let geom_line = _LineString(&sequence2);
    let x2 = sequence2.get_x(0);
    println!("x2 : {}", x2);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_line.to_wkt());
    let mut pt = Point((22.33, 44.55));
    println!("GeosGeom from coordinates : {:?}", pt.to_wkt());
}
