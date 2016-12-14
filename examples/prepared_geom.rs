extern crate geos;
use geos::{version, init, finish, GeosGeom, GeosPrepGeom};

fn main() {
    init();
    println!("geos_c version: {}", version());
    let g1 = GeosGeom::new("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))");
    let g2 = GeosGeom::new("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))");
    let pg1 = GeosPrepGeom::new(&g1);
    let result = pg1.intersects(&g2);
    let result2 = pg1.contains(&g2.get_centroid());
    println!("Prepared geometry intersects test polygon : {:?}", result);
    println!("Prepared geometry contains centroid other polygon : {:?}", result2);
    finish();
}
