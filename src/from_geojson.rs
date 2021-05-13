use crate::{CoordDimensions, CoordSeq, Geometry as GGeometry};
use error::{Error, GResult};
use geojson::{Geometry, Value};

use std::convert::{TryFrom, TryInto};
use std::iter;

fn create_coord_seq_from_vec<'a, 'b>(coords: &'a [Vec<f64>]) -> Result<CoordSeq<'b>, Error> {
    create_coord_seq(coords.iter(), coords.len())
}

fn create_coord_seq<'a, 'b, It>(points: It, len: usize) -> Result<CoordSeq<'b>, Error>
where
    It: Iterator<Item = &'a Vec<f64>>,
{
    let mut coord_seq =
        CoordSeq::new(len as u32, CoordDimensions::TwoD).expect("failed to create CoordSeq");

    for (i, p) in points.enumerate() {
        coord_seq.set_x(i, p[0])?;
        coord_seq.set_y(i, p[1])?;
    }
    Ok(coord_seq)
}

// We need to ensure that rings of polygons are closed
// to create valid GEOS LinearRings (geojson crate doesn't enforce this for now)
fn create_closed_coord_seq_from_vec<'a, 'b>(points: &'a [Vec<f64>]) -> Result<CoordSeq<'b>, Error> {
    let nb_points = points.len();
    // if the geom is not closed we close it
    let is_closed = nb_points > 0 && points.first() == points.last();
    // Note: we also need to close a 2 points closed linearring,
    // as in `from_geo` module
    let need_closing = nb_points > 0 && (!is_closed || nb_points == 3);
    if need_closing {
        create_coord_seq(points.iter().chain(iter::once(&points[0])), nb_points + 1)
    } else {
        create_coord_seq(points.iter(), nb_points)
    }
}

impl<'a, 'b> TryFrom<&'a Geometry> for GGeometry<'b> {
    type Error = Error;

    fn try_from(other: &'a Geometry) -> Result<GGeometry<'b>, Self::Error> {
        match other.value {
            Value::Point(ref c) => GGeometry::create_point(create_coord_seq(iter::once(c), 1)?),
            Value::MultiPoint(ref pts) => {
                let ggpts = pts
                    .iter()
                    .map(|pt| GGeometry::create_point(create_coord_seq(iter::once(pt), 1)?))
                    .collect::<GResult<Vec<GGeometry>>>()?;
                GGeometry::create_multipoint(ggpts)
            }
            Value::LineString(ref line) => {
                let coord_seq = create_coord_seq_from_vec(line.as_slice())?;
                GGeometry::create_line_string(coord_seq)
            }
            Value::MultiLineString(ref lines) => {
                let gglines = lines
                    .iter()
                    .map(|line| {
                        let coord_seq = create_coord_seq_from_vec(line.as_slice())?;
                        GGeometry::create_line_string(coord_seq)
                    })
                    .collect::<GResult<Vec<GGeometry>>>()?;
                GGeometry::create_multiline_string(gglines)
            }
            Value::Polygon(ref rings) => {
                let exterior_ring = GGeometry::create_linear_ring(
                    create_closed_coord_seq_from_vec(rings[0].as_slice())?,
                )?;
                let interiors = rings
                    .iter()
                    .skip(1)
                    .map(|r| {
                        GGeometry::create_linear_ring(create_closed_coord_seq_from_vec(
                            r.as_slice(),
                        )?)
                    })
                    .collect::<GResult<Vec<GGeometry>>>()?;
                GGeometry::create_polygon(exterior_ring, interiors)
            }
            Value::MultiPolygon(ref polygons) => {
                let ggpolys = polygons
                    .iter()
                    .map(|rings| {
                        let exterior_ring = GGeometry::create_linear_ring(
                            create_closed_coord_seq_from_vec(rings[0].as_slice())?,
                        )?;
                        let interiors = rings
                            .iter()
                            .skip(1)
                            .map(|r| {
                                GGeometry::create_linear_ring(create_closed_coord_seq_from_vec(
                                    r.as_slice(),
                                )?)
                            })
                            .collect::<GResult<Vec<GGeometry>>>()?;
                        GGeometry::create_polygon(exterior_ring, interiors)
                    })
                    .collect::<GResult<Vec<GGeometry>>>()?;
                GGeometry::create_multipolygon(ggpolys)
            }
            Value::GeometryCollection(ref geoms) => {
                let _geoms = geoms
                    .iter()
                    .map(|geom| geom.try_into())
                    .collect::<GResult<Vec<GGeometry>>>()?;
                GGeometry::create_geometry_collection(_geoms)
            }
        }
    }
}

impl<'a> TryFrom<Geometry> for GGeometry<'a> {
    type Error = Error;

    fn try_from(other: Geometry) -> Result<GGeometry<'a>, Self::Error> {
        GGeometry::try_from(&other)
    }
}

#[cfg(test)]
mod test {
    use crate::{Geom, Geometry as GGeometry};
    use geojson::{Geometry, Value};

    use std::convert::TryInto;

    #[test]
    fn geom_from_geojson_point() {
        let geojson_pt = Geometry::new(Value::Point(vec![1., 1.]));
        let gpoint: GGeometry = (&geojson_pt).try_into().unwrap();

        assert_eq!(gpoint.to_wkt_precision(0), Ok("POINT (1 1)".to_string()));
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_pt.try_into().unwrap();
        assert_eq!(tmp.to_wkt_precision(0), Ok("POINT (1 1)".to_string()),);
    }

    #[test]
    fn geom_from_geojson_multipoint() {
        let geojson_pts = Geometry::new(Value::MultiPoint(vec![vec![1., 1.], vec![2., 2.]]));
        let gpts: GGeometry = (&geojson_pts).try_into().unwrap();
        assert_eq!(
            gpts.to_wkt_precision(0),
            Ok("MULTIPOINT (1 1, 2 2)".to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_pts.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(0),
            Ok("MULTIPOINT (1 1, 2 2)".to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_line() {
        let geojson_line = Geometry::new(Value::LineString(vec![vec![1., 1.], vec![2., 2.]]));
        let gline: GGeometry = (&geojson_line).try_into().unwrap();
        assert_eq!(
            gline.to_wkt_precision(0),
            Ok("LINESTRING (1 1, 2 2)".to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_line.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(0),
            Ok("LINESTRING (1 1, 2 2)".to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_multiline() {
        let geojson_lines = Geometry::new(Value::MultiLineString(vec![
            vec![vec![1., 1.], vec![2., 2.]],
            vec![vec![3., 3.], vec![4., 4.]],
        ]));
        let glines: GGeometry = (&geojson_lines).try_into().unwrap();
        assert_eq!(
            glines.to_wkt_precision(0),
            Ok("MULTILINESTRING ((1 1, 2 2), (3 3, 4 4))".to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_lines.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(0),
            Ok("MULTILINESTRING ((1 1, 2 2), (3 3, 4 4))".to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_polygon() {
        let geojson_polygon = Geometry::new(Value::Polygon(vec![
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
        let gpolygon: GGeometry = (&geojson_polygon).try_into().unwrap();
        assert_eq!(
            gpolygon.to_wkt_precision(1),
            Ok("POLYGON ((0.0 0.0, 0.0 3.0, 3.0 3.0, 3.0 0.0, 0.0 0.0), (0.2 0.2, 0.2 2.0, 2.0 2.0, 2.0 0.2, 0.2 0.2))"
                .to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_polygon.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(1),
            Ok("POLYGON ((0.0 0.0, 0.0 3.0, 3.0 3.0, 3.0 0.0, 0.0 0.0), (0.2 0.2, 0.2 2.0, 2.0 2.0, 2.0 0.2, 0.2 0.2))"
                .to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_polygon_with_unclosed_interior_ring() {
        let geojson_polygon = Geometry::new(Value::Polygon(vec![
            vec![
                vec![0., 0.],
                vec![0., 3.],
                vec![3., 3.],
                vec![3., 0.],
                vec![0., 0.],
            ],
            vec![vec![0.2, 0.2], vec![0.2, 2.], vec![2., 2.], vec![2., 0.2]],
        ]));
        let gpolygon: GGeometry = (&geojson_polygon).try_into().unwrap();
        assert_eq!(
            gpolygon.to_wkt_precision(1),
            Ok("POLYGON ((0.0 0.0, 0.0 3.0, 3.0 3.0, 3.0 0.0, 0.0 0.0), (0.2 0.2, 0.2 2.0, 2.0 2.0, 2.0 0.2, 0.2 0.2))"
                .to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_polygon.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(1),
            Ok("POLYGON ((0.0 0.0, 0.0 3.0, 3.0 3.0, 3.0 0.0, 0.0 0.0), (0.2 0.2, 0.2 2.0, 2.0 2.0, 2.0 0.2, 0.2 0.2))"
                .to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_multipolygon() {
        let geojson_multipolygon = Geometry::new(Value::MultiPolygon(vec![vec![vec![
            vec![0., 0.],
            vec![0., 1.],
            vec![1., 1.],
            vec![1., 0.],
            vec![0., 0.],
        ]]]));
        let gmultipolygon: GGeometry = (&geojson_multipolygon).try_into().unwrap();
        assert_eq!(
            gmultipolygon.to_wkt_precision(0),
            Ok("MULTIPOLYGON (((0 0, 0 1, 1 1, 1 0, 0 0)))".to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_multipolygon.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(0),
            Ok("MULTIPOLYGON (((0 0, 0 1, 1 1, 1 0, 0 0)))".to_string()),
        );
    }

    #[test]
    fn geom_from_geojson_geometry_collection() {
        let geojson_gc = Geometry::new(Value::GeometryCollection(vec![
            Geometry::new(Value::Point(vec![1., 1.])),
            Geometry::new(Value::LineString(vec![vec![1., 1.], vec![2., 2.]])),
        ]));
        let gc: GGeometry = (&geojson_gc).try_into().unwrap();
        assert_eq!(
            gc.to_wkt_precision(0),
            Ok("GEOMETRYCOLLECTION (POINT (1 1), LINESTRING (1 1, 2 2))".to_string()),
        );
        // This check ensures that `TryFrom` is implemented for both reference and value.
        let tmp: GGeometry = geojson_gc.try_into().unwrap();
        assert_eq!(
            tmp.to_wkt_precision(0),
            Ok("GEOMETRYCOLLECTION (POINT (1 1), LINESTRING (1 1, 2 2))".to_string()),
        );
    }
}
