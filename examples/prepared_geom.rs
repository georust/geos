extern crate geos;
use geos::{version, Error, GGeom, PreparedGGeom};

fn fun() -> Result<(), Error> {
    println!("geos_c version: {}", version());
    let g1 = GGeom::new_from_wkt("POLYGON ((0 0, 0 5, 5 5, 5 0, 0 0))")?;
    let g2 = GGeom::new_from_wkt("POLYGON ((1 1, 1 3, 5 5, 5 0, 1 1))")?;
    let pg1 = PreparedGGeom::new(&g1)?;
    let result = pg1.intersects(&g2)?;
    let result2 = pg1.contains(&g2.get_centroid()?)?;
    println!("Prepared geometry intersects test polygon : {:?}", result);
    println!(
        "Prepared geometry contains centroid other polygon : {:?}",
        result2
    );
    println!("Prepared geometry intersects each geometry from a vec of GeosGeometry :");
    let vec_geoms = vec![
        GGeom::new_from_wkt("POINT (1.3 2.4)").unwrap(),
        GGeom::new_from_wkt("POINT (2.1 0.3)").unwrap(),
        GGeom::new_from_wkt("POINT (3.1 4.7)").unwrap(),
        GGeom::new_from_wkt("POINT (0.4 4.1)").unwrap(),
    ];
    for geom in &vec_geoms {
        print!("{:?} ", pg1.intersects(&geom)?);
    }
    println!("");
    Ok(())
}

fn main() {
    fun().unwrap();
}
