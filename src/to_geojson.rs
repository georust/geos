use crate::error::{Error, GResult};
use crate::{ConstGeometry, CoordSeq, Geom, Geometry as GGeometry, GeometryTypes};
use geojson::{Geometry, Value};

use std::convert::{TryFrom, TryInto};

fn coords_seq_to_vec_position(cs: &CoordSeq) -> GResult<Vec<Vec<f64>>> {
    let n_coords = cs.size()?;
    let mut coords = Vec::with_capacity(n_coords);
    for i in 0..n_coords {
        coords.push(vec![cs.get_x(i)?, cs.get_y(i)?]);
    }
    Ok(coords)
}

fn to_geojson<T: Geom>(other: T) -> Result<Geometry, Error> {
    let _type = other.geometry_type();
    match _type {
        GeometryTypes::Point => {
            let coord_seq = other.get_coord_seq()?;
            Ok(Geometry::new(Value::Point(vec![
                coord_seq.get_x(0)?,
                coord_seq.get_y(0)?,
            ])))
        }
        GeometryTypes::MultiPoint => {
            let n_pts = other.get_num_geometries()?;
            let mut coords = Vec::with_capacity(n_pts);
            for i in 0..n_pts {
                let coord_seq = other.get_geometry_n(i)?.get_coord_seq()?;
                coords.push(vec![coord_seq.get_x(0)?, coord_seq.get_y(0)?]);
            }
            Ok(Geometry::new(Value::MultiPoint(coords)))
        }
        GeometryTypes::LineString | GeometryTypes::LinearRing => {
            let cs = other.get_coord_seq()?;
            let coords = coords_seq_to_vec_position(&cs)?;
            Ok(Geometry::new(Value::LineString(coords)))
        }
        GeometryTypes::MultiLineString => {
            let n_lines = other.get_num_geometries()?;
            let mut result_lines = Vec::with_capacity(n_lines);
            for i in 0..n_lines {
                let cs = other.get_geometry_n(i)?.get_coord_seq()?;
                result_lines.push(coords_seq_to_vec_position(&(cs))?);
            }
            Ok(Geometry::new(Value::MultiLineString(result_lines)))
        }
        GeometryTypes::Polygon => {
            let nb_interiors = other.get_num_interior_rings()?;

            let mut rings = Vec::with_capacity(nb_interiors + 1usize);
            // Exterior ring to coordinates
            rings.push(coords_seq_to_vec_position(
                &(other.get_exterior_ring()?.get_coord_seq()?),
            )?);
            // Interior rings to coordinates
            for ix_interior in 0..nb_interiors {
                rings.push(coords_seq_to_vec_position(
                    &(other.get_interior_ring_n(ix_interior)?.get_coord_seq()?),
                )?);
            }
            Ok(Geometry::new(Value::Polygon(rings)))
        }
        GeometryTypes::MultiPolygon => {
            let n_polygs = other.get_num_geometries()?;
            let mut result_polygs = Vec::with_capacity(n_polygs);
            for i in 0..n_polygs {
                let polyg = other.get_geometry_n(i)?;
                let nb_interiors = polyg.get_num_interior_rings()?;

                let mut rings = Vec::with_capacity(nb_interiors + 1usize);
                // Exterior ring to coordinates
                rings.push(coords_seq_to_vec_position(
                    &(polyg.get_exterior_ring()?.get_coord_seq()?),
                )?);
                // Interior rings to coordinates
                for ix_interior in 0..nb_interiors {
                    rings.push(coords_seq_to_vec_position(
                        &(polyg.get_interior_ring_n(ix_interior)?.get_coord_seq()?),
                    )?);
                }
                result_polygs.push(rings);
            }
            Ok(Geometry::new(Value::MultiPolygon(result_polygs)))
        }
        GeometryTypes::GeometryCollection => {
            let n_geoms = other.get_num_geometries()?;
            let mut result_geoms = Vec::with_capacity(n_geoms);
            for i in 0..n_geoms {
                let g = other.get_geometry_n(i)?;
                let geojsongeom: Geometry = g.try_into()?;
                result_geoms.push(geojsongeom);
            }
            Ok(Geometry::new(Value::GeometryCollection(result_geoms)))
        }
        #[cfg(feature = "v3_13_0")]
        _ => Err(Error::GenericError("invalid type for GeoJSON".into())),
    }
}

impl TryFrom<GGeometry> for Geometry {
    type Error = Error;

    fn try_from(other: GGeometry) -> Result<Geometry, Self::Error> {
        to_geojson(other)
    }
}

impl TryFrom<ConstGeometry<'_>> for Geometry {
    type Error = Error;

    fn try_from(other: ConstGeometry<'_>) -> Result<Geometry, Self::Error> {
        to_geojson(other)
    }
}

#[cfg(test)]
mod test {
    use crate::Geometry as GGeometry;
    use geojson::{Geometry, Value};

    use std::convert::TryInto;

    #[test]
    fn geom_to_geojson_point() {
        let pt = "POINT(1 1)";
        let pt = GGeometry::new_from_wkt(pt).unwrap();

        let geojson_pt: Geometry = pt.try_into().unwrap();

        let expected_pt = Geometry::new(Value::Point(vec![1., 1.]));
        assert_eq!(geojson_pt, expected_pt);
    }

    #[test]
    fn geom_to_geojson_multipoint() {
        let pts = "MULTIPOINT((1 1), (2 2))";
        let pts = GGeometry::new_from_wkt(pts).unwrap();

        let geojson_pts: Geometry = pts.try_into().unwrap();

        let expected_pts = Geometry::new(Value::MultiPoint(vec![vec![1., 1.], vec![2., 2.]]));
        assert_eq!(geojson_pts, expected_pts);
    }

    #[test]
    fn geom_to_geojson_line() {
        let line = "LINESTRING(1 1, 2 2)";
        let line = GGeometry::new_from_wkt(line).unwrap();

        let geojson_line: Geometry = line.try_into().unwrap();

        let expected_line = Geometry::new(Value::LineString(vec![vec![1., 1.], vec![2., 2.]]));
        assert_eq!(geojson_line, expected_line);
    }

    #[test]
    fn geom_to_geojson_linearring() {
        let line = "LINEARRING(1 1, 2 1, 2 2, 1 1)";
        let line = GGeometry::new_from_wkt(line).unwrap();

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
        let line = GGeometry::new_from_wkt(line).unwrap();

        let geojson_line: Geometry = line.try_into().unwrap();

        let expected_line = Geometry::new(Value::MultiLineString(vec![
            vec![vec![1., 1.], vec![2., 2.]],
            vec![vec![3., 3.], vec![4., 4.]],
        ]));
        assert_eq!(geojson_line, expected_line);
    }

    #[test]
    fn geom_to_geojson_polygon() {
        let poly = "POLYGON((0 0, 0 3, 3 3, 3 0, 0 0) ,(0.2 0.2, 0.2 2, 2 2, 2 0.2, 0.2 0.2))";
        let poly = GGeometry::new_from_wkt(poly).unwrap();

        let geojson_polygon: Geometry = poly.try_into().unwrap();

        let expected_polygon = Geometry::new(Value::Polygon(vec![
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
        ]));
        assert_eq!(geojson_polygon, expected_polygon);
    }

    #[test]
    fn geom_to_geojson_multipolygon() {
        let poly = "MULTIPOLYGON(((0 0, 0 1, 1 1, 1 0, 0 0)))";
        let poly = GGeometry::new_from_wkt(poly).unwrap();

        let geojson_polygon: Geometry = poly.try_into().unwrap();

        let expected_polygon = Geometry::new(Value::MultiPolygon(vec![vec![vec![
            vec![0., 0.],
            vec![0., 1.],
            vec![1., 1.],
            vec![1., 0.],
            vec![0., 0.],
        ]]]));
        assert_eq!(geojson_polygon, expected_polygon);
    }

    #[test]
    fn geom_to_geojson_geometry_collection() {
        let gc = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
        let gc = GGeometry::new_from_wkt(gc).unwrap();

        let geojson_gc: Geometry = gc.try_into().unwrap();

        let expected_gc = Geometry::new(Value::GeometryCollection(vec![
            Geometry::new(Value::Point(vec![1., 1.])),
            Geometry::new(Value::LineString(vec![vec![1., 1.], vec![2., 2.]])),
        ]));
        assert_eq!(geojson_gc, expected_gc);
    }
}
