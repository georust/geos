extern crate geos;
use geos::{version, init, finish, GeosGeom, CoordSeq, _Point, LineString, GEOSGeomTypes};
use std::ffi::{CString, CStr};

pub fn Point(pt: (f64, f64)) -> CString {
    let mut sequence = CoordSeq::new(1, 2);
    sequence.set_x(0, pt.0);
    sequence.set_y(0, pt.1);
    let geom = _Point(&sequence);
    return geom.to_wkt();
}

fn main() {
    init();
    println!("geos_c version: {}", version());
    let mut sequence = CoordSeq::new(1, 2);
    let ret_val = sequence.set_x(0, 12.36);
    println!("{:?}", ret_val);
    let ret_val = sequence.set_y(0, 43.21);
    println!("{:?}", ret_val);
    let geom_point = _Point(&sequence);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_point.to_wkt());
    let mut sequence2 = CoordSeq::new(2, 2);
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
    let wkt_pt = Point((22.33, 44.55));
    println!("GeosGeom from coordinates : {:?}", wkt_pt);
    println!("{:?}", GEOSGeomTypes::GEOS_MULTIPOINT);
    finish();
}
