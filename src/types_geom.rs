use libc::c_uint;
use std::str;
use num::{Float, ToPrimitive};
use ffi::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Coordinate<T>
    where T: Float + ToPrimitive
{
    pub x: T,
    pub y: T,
}

pub struct Point;

impl Point {
    pub fn new(coords: (f64, f64)) -> GGeom {
        let sequence = CoordSeq::new(1, 2);
        sequence.set_x(0, coords.0);
        sequence.set_y(0, coords.1);
        _point(&sequence)
    }
}

pub struct LineString;

impl LineString {
    pub fn new(coords: &[(f64, f64)]) -> GGeom {
        let nb_pts = coords.len();
        let sequence = CoordSeq::new(nb_pts as u32, 2);
        for i in 0..nb_pts {
            let j = i as u32;
            sequence.set_x(j, coords[i].0);
            sequence.set_y(j, coords[i].1);
        }
        _lineString(&sequence)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Ring<T> (pub Vec<Coordinate<T>>) where T: Float + ToPrimitive;

impl<T> Ring<T> where T: Float + ToPrimitive {
    pub fn new(coords: &[(T, T)]) -> Ring<T> {
        let nb_coords = coords.len();
        let mut ring = Vec::new();
        for i in 0..nb_coords {
            ring.push(Coordinate {x: coords[i].0, y: coords[i].1});
        }
        Ring(ring)
    }
}

pub struct Polygon;

impl Polygon {
    pub fn new<T: Float>(exterior: &Ring<T>, interiors: &[Ring<T>]) -> GGeom {
        let nb_pts = exterior.0.len();
        let coord_seq_ext = CoordSeq::new(nb_pts as u32, 2);
        for i in 0..nb_pts {
            let j = i as u32;
            coord_seq_ext.set_x(j, exterior.0[i].x.to_f64().unwrap());
            coord_seq_ext.set_y(j, exterior.0[i].y.to_f64().unwrap());
        }
        let geom_exterior = _linearRing(&coord_seq_ext);
        let mut rings = Vec::new();
        println!("Exterior ring created");
        let nb_interiors_rings = interiors.len();
        for i in 0..nb_interiors_rings {
            let nb_pts = interiors[i].0.len();
            let coord_seq_interior = CoordSeq::new(nb_pts as u32, 2);
            for j in 0..nb_pts {
                let ix = j as u32;
                coord_seq_interior.set_x(ix, interiors[i].0[j].x.to_f64().unwrap());
                coord_seq_interior.set_y(ix, interiors[i].0[j].y.to_f64().unwrap());
            }
            rings.push( unsafe { GEOSGeom_clone(_linearRing(&coord_seq_interior).c_obj) } );
        }
        let t = unsafe { GEOSGeom_createPolygon(GEOSGeom_clone(geom_exterior.c_obj), &rings[..], nb_interiors_rings as c_uint) };
        GGeom::new_from_c_obj(t).clone()
    }
}
