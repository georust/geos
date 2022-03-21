fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut libgeos_config = cmake::Config::new("source");
    libgeos_config
        .define("BUILD_BENCHMARKS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_DOCUMENTATION", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .profile("Release");

    let libgeos = libgeos_config.build();

    println!("cargo:lib=geos_c");
    println!("cargo:lib=geos");

    let search_path = format!("{}/lib", libgeos.display());
    assert!(std::path::Path::new(&search_path).exists());
    println!("cargo:search={}", search_path);
}
