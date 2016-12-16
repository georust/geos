extern crate geos;
extern crate gdal;

use std::path::Path;
use std::fmt::Display;
use gdal::vector::{Dataset, FieldValue, Geometry, ToGdal};
use geos::{GGeom};

fn main() {
    let mut datasetA = Dataset::open(Path::new("examples/GrandParisMunicipalities.geojson")).unwrap();
    let layerA = datasetA.layer(0).unwrap();
    let mut datasetB = Dataset::open(Path::new("examples/quartier_paris.geojson")).unwrap();
    let layerB = datasetB.layer(0).unwrap();
    let mut countA: u32 = 0;
    let mut countB: u32 = 0;
    let mut intersections: u32 = 0;
    for featureA in layerA.features() {
        let geomA =  featureA.geometry();
        let ggeomA = GGeom::new(&geomA.wkt());
        for featureB in layerB.features() {
            let geomB = featureB.geometry();
            let ggeomB = GGeom::new(&geomB.wkt());
            if(ggeomB.intersects(&ggeomA)){
                intersections = intersections + 1;
            }
            countB = countB + 1;
        }
        countA = countA + 1;
    }
    println!("{} features in layer A - {} features in layer B", countA, countB);
    println!("{} intersections", intersections);
}
