extern crate pkg_config;

fn main() {
    let lib = "geos_c";

    if std::env::var("CARGO_FEATURE_STATIC").is_ok() {
        let geos_path = std::env::var("DEP_GEOSSRC_SEARCH").unwrap();

        println!("cargo:rustc-link-lib=static=geos_c");
        println!("cargo:rustc-link-lib=static=geos");
        println!("cargo:rustc-link-search=native={}", geos_path);
        println!("cargo:includedir={}/include", geos_path)
    } else {
        match pkg_config::Config::new().probe(lib) {
            Ok(_) => {}
            Err(_) => {
                println!("cargo:rustc-link-lib=dylib={}", lib);
            }
        }
    }

    // TODO: handle library versions like this!
    //
    // pkg_config::probe_library("geos_c")
    //     .map(|lib| {
    //         if lib.version.starts_with("2.") {
    //             println!(r#"cargo:rustc-cfg=feature="v2""#);
    //         }
    //     })
    //     .expect("GEOS library not found");
}
