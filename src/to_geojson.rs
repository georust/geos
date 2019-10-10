use crate::{CoordSeq, Geometry as GGeom, GeometryTypes};
use error::{Error, GResult};
use geojson::{Geometry, Value};


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
            GeometryTypes::LineString | GeometryTypes::LinearRing => {
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
                        polyg.get_exterior_ring()?.get_coord_seq()?))?);
                    // Interior rings to coordinates
                    for ix_interior in 0..nb_interiors {
                        rings.push(coords_seq_to_vec_position(
                            &(polyg.get_interior_ring_n(ix_interior as u32)?.get_coord_seq()?))?);
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
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::GGeom;
    use crate::to_geojson::TryInto;
    use geojson::{Geometry, Value};

    #[test]
    fn geom_to_geojson_point() {
        let pt = "POINT(1 1)";
        let pt = GGeom::new_from_wkt(pt).unwrap();

        let geojson_pt: Geometry = pt.try_into().unwrap();

        let expected_pt = Geometry::new(Value::Point(vec![1., 1.]));
        assert_eq!(geojson_pt, expected_pt);
    }

    #[test]
    fn geom_to_geojson_multipoint() {
        let pts = "MULTIPOINT((1 1), (2 2))";
        let pts = GGeom::new_from_wkt(pts).unwrap();

        let geojson_pts: Geometry = pts.try_into().unwrap();

        let expected_pts = Geometry::new(Value::MultiPoint(vec![
            vec![1., 1.],
            vec![2., 2.],
        ]));
        assert_eq!(geojson_pts, expected_pts);
    }

    #[test]
    fn geom_to_geojson_line() {
        let line = "LINESTRING(1 1, 2 2)";
        let line = GGeom::new_from_wkt(line).unwrap();

        let geojson_line: Geometry = line.try_into().unwrap();

        let expected_line = Geometry::new(Value::LineString(vec![
            vec![1., 1.],
            vec![2., 2.],
        ]));
        assert_eq!(geojson_line, expected_line);
    }

    #[test]
    fn geom_to_geojson_linearring() {
        let line = "LINEARRING(1 1, 2 1, 2 2, 1 1)";
        let line = GGeom::new_from_wkt(line).unwrap();

        let geojson_line: Geometry = line.try_into().unwrap();

        let expected_line = Geometry::new(Value::LineString(vec![
            vec![1., 1.],
            vec![2., 1.],
            vec![2., 2.],
            vec![1., 1.],
        ]));
        assert_eq!(geojson_line, expected_line);
    }

    #[test]
    fn geom_to_geojson_multiline() {
        let line = "MULTILINESTRING((1 1, 2 2), (3 3, 4 4))";
        let line = GGeom::new_from_wkt(line).unwrap();

        let geojson_line: Geometry = line.try_into().unwrap();

        let expected_line = Geometry::new(Value::MultiLineString(vec![
            vec![
                vec![1., 1.],
                vec![2., 2.],
            ],
            vec![
                vec![3., 3.],
                vec![4., 4.],
            ],
        ]));
        assert_eq!(geojson_line, expected_line);
    }


    #[test]
    fn geom_to_geojson_polygon() {
        let poly = "POLYGON((0 0, 0 3, 3 3, 3 0, 0 0) ,(0.2 0.2, 0.2 2, 2 2, 2 0.2, 0.2 0.2))";
        let poly = GGeom::new_from_wkt(poly).unwrap();

        let geojson_polygon: Geometry = poly.try_into().unwrap();

        let expected_polygon = Geometry::new(Value::Polygon(
            vec![
                vec![
                    vec![0., 0.],
                    vec![0., 3.],
                    vec![3., 3.],
                    vec![3., 0.],
                    vec![0., 0.],
                ],
                vec![
                    vec![0.2, 0.2],
                    vec![0.2, 2.],
                    vec![2., 2.],
                    vec![2., 0.2],
                    vec![0.2, 0.2],
                ],
            ]
        ));
        assert_eq!(geojson_polygon, expected_polygon);
    }

    #[test]
    fn geom_to_geojson_multipolygon() {
        let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
        let poly = GGeom::new_from_wkt(poly).unwrap();

        let geojson_polygon: Geometry = poly.try_into().unwrap();

        let expected_polygon = Geometry::new(Value::MultiPolygon(
            vec![vec![vec![
                vec![0., 0.],
                vec![0., 1.],
                vec![1., 1.],
                vec![1., 0.],
                vec![0., 0.],
            ]]]
        ));
        assert_eq!(geojson_polygon, expected_polygon);
    }

    #[test]
    fn geom_to_geojson_geometry_collection() {
        let gc = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
        let gc = GGeom::new_from_wkt(gc).unwrap();

        let geojson_gc: Geometry = gc.try_into().unwrap();

        let expected_gc = Geometry::new(Value::GeometryCollection(
            vec![
                Geometry::new(Value::Point(vec![1., 1.])),
                Geometry::new(Value::LineString(vec![vec![1., 1.], vec![2., 2.]])),
            ]
        ));
        assert_eq!(geojson_gc, expected_gc);
    }

}
