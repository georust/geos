use crate::{CoordSeq, Geometry as GGeom, GeometryTypes};
use error::{Error, GResult};
use geojson::{Geometry, Value};
// use std::convert::TryInto;


pub trait TryInto<T> {
    type Err;
    fn try_into(self) -> Result<T, Self::Err>;
}


fn coords_seq_to_vec_position(cs: &CoordSeq) -> GResult<Vec<Vec<f64>>> {
    let n_coords = cs.size()?;
    let mut coords = Vec::with_capacity(n_coords);
    for i in 0..n_coords {
        coords.push(vec![
            cs.get_x(i)?,
            cs.get_y(i)?,
        ]);
    }
    Ok(coords)
}

impl<'a> TryInto<Geometry> for GGeom<'a> {
    type Err = Error;

    fn try_into(self) -> Result<Geometry, Self::Err> {
        let _type = self.geometry_type();
        match _type {
            GeometryTypes::Point => {
                let coord_seq = self.get_coord_seq()?;
                Ok(Geometry::new(
                    Value::Point(
                        vec![
                            coord_seq.get_x(0)?,
                            coord_seq.get_y(0)?,
                        ]
                    )
                ))
            },
            GeometryTypes::MultiPoint => {
                let n_pts = self.get_num_geometries()?;
                let mut coords = Vec::with_capacity(n_pts);
                for i in 0..n_pts {
                    let coord_seq = self.get_geometry_n(i)?.get_coord_seq()?;
                    coords.push(vec![
                        coord_seq.get_x(0)?,
                        coord_seq.get_y(0)?,
                    ]);
                }
                Ok(Geometry::new(Value::MultiPoint(coords)))
            },
            GeometryTypes::LineString => {
                let cs = self.get_coord_seq()?;
                let coords = coords_seq_to_vec_position(&cs)?;
                Ok(Geometry::new(Value::LineString(coords)))
            },
            GeometryTypes::MultiLineString => {
                let n_lines = self.get_num_geometries()?;
                let mut result_lines = Vec::with_capacity(n_lines);
                for i in 0..n_lines {
                    let cs = self.get_geometry_n(i)?.get_coord_seq()?;
                    result_lines.push(coords_seq_to_vec_position(&(cs))?);
                }
                Ok(Geometry::new(Value::MultiLineString(result_lines)))
            },
            GeometryTypes::Polygon => {
                let nb_interiors = self.get_num_interior_rings()?;

                let mut rings = Vec::with_capacity(nb_interiors + 1usize);
                // Exterior ring to coordinates
                rings.push(coords_seq_to_vec_position(&(
                    self.get_exterior_ring()?.get_coord_seq()?))?);
                // Interior rings to coordinates
                for ix_interior in 0..nb_interiors {
                    rings.push(coords_seq_to_vec_position(
                        &(self.get_interior_ring_n(ix_interior as u32)?.get_coord_seq()?))?);
                }
                Ok(Geometry::new(Value::Polygon(rings)))
            },
            GeometryTypes::MultiPolygon => {
                let n_polygs = self.get_num_geometries()?;
                let mut result_polygs = Vec::with_capacity(n_polygs);
                for i in 0..n_polygs {
                    let polyg = self.get_geometry_n(i)?;
                    let nb_interiors = polyg.get_num_interior_rings()?;

                    let mut rings = Vec::with_capacity(nb_interiors + 1usize);
                    // Exterior ring to coordinates
                    rings.push(coords_seq_to_vec_position(&(
                        self.get_exterior_ring()?.get_coord_seq()?))?);
                    // Interior rings to coordinates
                    for ix_interior in 0..nb_interiors {
                        rings.push(coords_seq_to_vec_position(
                            &(self.get_interior_ring_n(ix_interior as u32)?.get_coord_seq()?))?);
                    }
                    result_polygs.push(rings);
                }
                Ok(Geometry::new(Value::MultiPolygon(result_polygs)))
            },
            GeometryTypes::GeometryCollection => {
                let n_geoms = self.get_num_geometries()?;
                let mut result_geoms = Vec::with_capacity(n_geoms);
                for i in 0..n_geoms {
                    let g = self.get_geometry_n(i)?;
                    let geojsongeom: Geometry = g.try_into()?;
                    result_geoms.push(geojsongeom);
                }
                Ok(Geometry::new(Value::GeometryCollection(result_geoms)))
            },
            _ => unreachable!()
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::GGeom;
//     use from_geo::TryInto;
//     use geo_types::{Coordinate, Geometry, LineString, Ok(MultiPolygon, Polygon};
//
//     fn coords(tuples: Vec<(f64, f64)>) -> Vec<Coordinate<f64>> {
//         tuples.into_iter().map(Coordinate::from).collect()
//     }
//
//     #[test]
//     fn geom_to_geo_polygon() {
//         let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
//         let poly = GGeom::new_from_wkt(poly).unwrap();
//
//         let geo_polygon: Geometry<f64> = poly.try_into().unwrap();
//
//         let exterior = LineString(coords(vec![
//             (0., 0.),
//             (0., 1.),
//             (1., 1.),
//             (1., 0.),
//             (0., 0.),
//         ]));
//         let expected_poly = MultiPolygon(vec![Polygon::new(exterior, vec![])]);
//         let expected: Geometry<_> = expected_poly.into();
//         assert_eq!(expected, geo_polygon);
//     }
// }
