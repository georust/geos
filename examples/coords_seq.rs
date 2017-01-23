extern crate geos;
use geos::{version, GGeom, CoordSeq, _point, _lineString};
use geos::types_geom::{Point, LineString, Ring, Polygon};

fn main() {
    println!("geos_c version: {}", version());
    let mut sequence = CoordSeq::new(1, 2);
    let mut ret_val = sequence.set_x(0, 12.36);
    assert_eq!(ret_val, 1);
    ret_val = sequence.set_y(0, 43.21);
    assert_eq!(ret_val, 1);
    let geom_point = _point(&mut sequence);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_point.to_wkt());
    let mut sequence2 = CoordSeq::new(2, 2);
    ret_val = sequence2.set_x(0, 12.36);
    assert_eq!(ret_val, 1);
    ret_val = sequence2.set_y(0, 43.21);
    assert_eq!(ret_val, 1);
    ret_val = sequence2.set_x(1, 12.78);
    assert_eq!(ret_val, 1);
    ret_val = sequence2.set_y(1, 42.80);
    assert_eq!(ret_val, 1);
    let geom_line = _lineString(&mut sequence2);
    let x2 = sequence2.get_x(0);
    println!("x2 : {}", x2);
    println!("GeosGeom from Coordinates sequence, to WKT : {:?}", geom_line.to_wkt());
    let pt = Point::new((22.33, 44.55));
    println!("GeosGeom from coordinates : {:?}", pt.to_wkt());
    let pt2 = GGeom::new("POINT (1.3 2.4)");
    println!("CoordSeq from GeosGeometry :", );
    let coord_seq = pt.get_coord_seq().unwrap();
    let mut x = coord_seq.get_x(0);
    let mut y = coord_seq.get_y(0);
    assert_eq!(x, 22.33);
    assert_eq!(y, 44.55);
    println!("{:?}, {:?}", x, y);
    let coord_seq2 = pt2.get_coord_seq().unwrap();
    x = coord_seq2.get_x(0);
    y = coord_seq2.get_y(0);
    println!("{:?}, {:?}", x, y);
    assert_eq!(x, 1.3);
    assert_eq!(y, 2.4);
    let l_geom = LineString::new(&[(12.78, 78.08), (55.77, 77.55), (22.77, 88.99)]);
    println!("GeosGeom Linestring from coordinates : {:?}", l_geom.to_wkt());
    // let exterior_ring = Ring::new(&[(0.0, 0.0), (0.0, 8.0), (8.0, 8.0), (8.0, 0.0), (0.0, 0.0)]);
    // let interior = Ring::new(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)]);
    // let poly_geom = Polygon::new(&exterior_ring, &[interior]);
    // println!("GeosGeom Polygon from ring coordinates : {:?}", poly_geom.to_wkt());

    let poly_geom = Polygon::new(
        &Ring::new(&[(0.0, 0.0), (0.0, 8.0), (8.0, 8.0), (8.0, 0.0), (0.0, 0.0)]),
        &[Ring::new(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)]), Ring::new(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)])]);
    println!("GeosGeom Polygon from ring coordinates : {:?}", poly_geom.to_wkt());
	assert!(!poly_geom.contains(&pt));
	assert!(!l_geom.intersects(&poly_geom));
}
