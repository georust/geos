#[cfg(test)]
mod test {
    use geo::{Point, LineString, Polygon, MultiPolygon, Coordinate};
    use num::traits::Float;

    fn p<T: Float>(x: T, y: T) -> Point<T> {
        Point(Coordinate { x: x, y: y })
    }

    #[test]
    fn test_ring_conversion() {
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        assert!(false);
    }
}