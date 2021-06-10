extern crate geos;
use geos::{Geom, Geometry, PreparedGeometry};

#[test]
fn test_prep_no_drop_geom() {
    pub struct Boo {
        #[allow(dead_code)]
        geom: Geometry<'static>,
        pub prep: PreparedGeometry<'static>,
    }

    let boo = {
        let geom = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
            .expect("Invalid geometry");
        let prep = geom
            .to_prepared_geom()
            .expect("failed to create prepared geom");

        Boo { geom, prep }
    };

    let pt = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");

    assert!(boo.prep.contains(&pt).unwrap());
}

#[test]
fn test_prep_drop_geom() {
    pub struct Boo {
        pub prep: PreparedGeometry<'static>,
    }

    let boo = {
        let geom1 = Geometry::new_from_wkt("POLYGON((0 0, 10 0, 10 6, 0 6, 0 0))")
            .expect("Invalid geometry");
        let prep = geom1
            .to_prepared_geom()
            .expect("failed to create prepared geom");

        Boo { prep }
    };

    let pt = Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");

    assert!(boo.prep.contains(&pt).unwrap());
}
