fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut libgeos_config = cmake::Config::new("source");
    libgeos_config
        .define("BUILD_BENCHMARKS", "OFF")
        // BUILD_TESTING may need to be set ON for GEOS 3.8.x / 3.9.x due to CMake issues
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_DOCUMENTATION", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("BUILD_SHARED_LIBS", "OFF") // GEOS >= 3.8
        .define("GEOS_BUILD_STATIC", "ON") // GEOS <= 3.7
        .define("GEOS_BUILD_SHARED", "OFF") // GEOS <= 3.7
        .profile("Release");

    let libgeos = libgeos_config.build();

    println!("cargo:lib=geos_c");
    println!("cargo:lib=geos");

    let search_path = format!("{}/lib", libgeos.display());
    assert!(std::path::Path::new(&search_path).exists());
    println!("cargo:search={}", search_path);
}
