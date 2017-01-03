extern crate geos;
use geos::{version, GGeom, CoordSeq, _Point, Point, _LineString, LineString, _LinearRing, _Polygon};


fn Ring(pts: &[(f64, f64)]) -> GGeom {
    let nb_pts = pts.len();
    let sequence = CoordSeq::new(nb_pts as u32, 2);
    for i in 0..nb_pts {
        let j = i as u32;
        sequence.set_x(j, pts[i].0);
        sequence.set_y(j, pts[i].1);
    }
    _LinearRing(&sequence).clone()
}


fn main() {
    println!("geos_c version: {}", version());
    let mut sequence = CoordSeq::new(1, 2);
    let mut ret_val = sequence.set_x(0, 12.36);
    assert_eq!(ret_val, 1);
    ret_val = sequence.set_y(0, 43.21);
    assert_eq!(ret_val, 1);
    let geom_point = _Point(&mut sequence);
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
    let geom_line = _LineString(&mut sequence2);
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
    let exterior_ring = Ring(&[(0.0, 0.0), (0.0, 8.0), (8.0, 8.0), (8.0, 0.0), (0.0, 0.0)]);
    let interior = Ring(&[(1.0, 1.0), (4.0, 1.0), (4.0, 4.0), (1.0, 4.0), (1.0, 1.0)]);
    let poly_geom = _Polygon(&exterior_ring, &[&interior], 1 as u32);
    println!("GeosGeom Polygons from ring coordinates : {:?}", poly_geom.to_wkt());
}
