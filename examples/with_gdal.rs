extern crate geos;
extern crate gdal;

use std::path::Path;
use gdal::vector::{Dataset, Geometry};
use geos::GGeom;

trait ToGdal {
    fn to_gdal(&self) -> gdal::vector::Geometry;
}

impl ToGdal for GGeom {
    fn to_gdal(&self) -> gdal::vector::Geometry {
        Geometry::from_wkt(&self.to_wkt())
    }
}

fn main() {
    let mut dataset_a = Dataset::open(Path::new("examples/GrandParisMunicipalities.geojson")).unwrap();
    let layer_a = dataset_a.layer(0).unwrap();
    let mut dataset_b = Dataset::open(Path::new("examples/quartier_paris.geojson")).unwrap();
    let layer_b = dataset_b.layer(0).unwrap();
    let mut count_a: u32 = 0;
    let mut count_b: u32 = 0;
    let mut intersections: u32 = 0;
    let mut stack = Vec::new();
    let mut stack2 = Vec::new();
    for feature_a in layer_a.features() {
        let ggeom_a = GGeom::new(&feature_a.geometry().wkt());
        for feature_b in layer_b.features() {
            let ggeom_b = GGeom::new(&feature_b.geometry().wkt());
            if ggeom_b.intersects(&ggeom_a) {
                let new_geom = ggeom_b.difference(&ggeom_a);
                let new_geom2 = new_geom.to_gdal();
                stack.push(new_geom);
                stack2.push(new_geom2);
                intersections = intersections + 1;
            }
            count_b = count_b + 1;
        }
        count_a = count_a + 1;
    }
    println!("{} features in layer A - {} features in layer B", count_a, count_b);
    println!("{} intersections", intersections);

}
