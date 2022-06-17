use semver::Version;
use std::env;
use std::path::{Path, PathBuf};

const MINIMUM_GEOS_VERSION: &str = "3.7.0";
const BUNDLED_GEOS_VERSION: &str = "3.11.0"; // TODO: 3.10.0

/// Hardcode a prebuilt binding version while generating docs.
/// Otherwise docs.rs will explode due to not actually having libgeos installed.
fn set_bindings_for_docs(out_path: &PathBuf) {
    let version = Version::parse(BUNDLED_GEOS_VERSION).expect("invalid version for docs.rs");
    println!(
        "cargo:rustc-cfg=geos_sys_{}_{}_{}",
        version.major, version.minor, version.patch
    );

    let binding_path = PathBuf::from(format!(
        "prebuilt-bindings/geos_{}.{}.rs",
        version.major, version.minor
    ));

    if !binding_path.exists() {
        panic!("Missing bindings for docs.rs (version {})", version);
    }

    std::fs::copy(&binding_path, &out_path).expect("Can't copy bindings to output directory");
}


fn write_bindings(include_paths: Vec<String>, out_path: &Path) {
    let mut builder = bindgen::Builder::default()
        .size_t_is_usize(true)
        .header("wrapper.h");

    for path in include_paths {
        builder = builder.clang_arg("-I");
        builder = builder.clang_arg(path);
    }

    builder
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");
}

fn main() {
    println!("cargo:rerun-if-env-changed=GEOS_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GEOS_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GEOS_VERSION");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    let mut version = Version::new(0, 0, 0);
    let mut include_paths = Vec::new();

    if env::var("DOCS_RS").is_ok() {
        set_bindings_for_docs(&out_path);
        return;
    }

    // static feature includes building the included GEOS prior to this build step
    if cfg!(feature = "static"){
        let geos_path = std::env::var("DEP_GEOSSRC_SEARCH").unwrap();

        println!("cargo:rustc-link-lib=static=geos_c");
        println!("cargo:rustc-link-lib=static=geos");
        println!("cargo:rustc-link-search=native={}", geos_path);
        println!("cargo:includedir={}/include", geos_path);

        include_paths.push(geos_path + "/include");

        version = Version::parse(BUNDLED_GEOS_VERSION).unwrap();
    } else {
        use pkg_config::Config;

        let include_dir_env = env::var_os("GEOS_INCLUDE_DIR");
        let lib_dir_env = env::var_os("GEOS_LIB_DIR");
        let version_env = env::var_os("GEOS_VERSION");

        if include_dir_env.is_some() || lib_dir_env.is_some() || version_env.is_some() {
            // if any env vars are set, all must be set
            println!("cargo:rustc-link-lib=dylib=geos_c");

            // GEOS_INCLUDE_DIR
            match include_dir_env {
                Some(path) => {
                    let include_dir = PathBuf::from(path).as_path().to_str().unwrap().to_string();
                    include_paths.push(include_dir);
                }
                None => {
                    panic!("GEOS_INCLUDE_DIR must be set");
                }
            }

            // GEOS_LIB_DIR
            match lib_dir_env {
                Some(path) => {
                    let lib_dir = PathBuf::from(path).as_path().to_str().unwrap().to_string();
                    println!("cargo:rustc-link-search={}", lib_dir);
                }
                None => {
                    panic!("GEOS_LIB_DIR must be set");
                }
            }

            // GEOS_VERSION
            match version_env {
                Some(raw_version) => {
                    match Version::parse(raw_version.to_string_lossy().to_string().trim()) {
                        Ok(parsed_version) => {
                            version = parsed_version;
                        }
                        Err(_) => panic!("Could not parse version: {:?}", raw_version),
                    }
                }
                None => {
                    panic!("GEOS_VERSION must be set");
                }
            }
        } else {
            let geos_pkg_config = Config::new()
                // .cargo_metadata(need_metadata)
                .probe("geos");

            if let Ok(geos) = &geos_pkg_config {
                for dir in &geos.include_paths {
                    include_paths.push(dir.to_str().unwrap().to_string());
                }

                // standardize GEOS alpha / beta versions to match semver format:
                let raw_version = geos
                    .version
                    .trim()
                    .replace("alpha", "-alpha")
                    .replace("beta", "-beta");

                if let Ok(pkg_version) = Version::parse(&raw_version) {
                    version = pkg_version;
                }
            } else if let Err(pkg_config_err) = &geos_pkg_config {
                // Special case output for this common error
                if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
                {
                    panic!("Could not find `pkg-config` in your path. Please install it before building geos-sys.");
                } else {
                    panic!("Error while running `pkg-config`: {}", pkg_config_err);
                }
            } else {
                panic!("No GEOS version detected");
            }
        }

        let min_geos_version = Version::parse(MINIMUM_GEOS_VERSION).unwrap();
        if version < min_geos_version {
            panic!(
                "GEOS version {}.{}.{} is older than the minimum supported version {}.{}.{}",
                version.major,
                version.minor,
                version.patch,
                min_geos_version.major,
                min_geos_version.minor,
                min_geos_version.patch
            );
        }
    }

    if cfg!(feature = "bindgen") {
        write_bindings(include_paths, &out_path);
    } else {
        {
            println!(
                "cargo:rustc-cfg=geos_sys_{}_{}_{}",
                version.major, version.minor, version.patch
            );

            let binding_path = PathBuf::from(format!(
                "prebuilt-bindings/geos_{}.{}.rs",
                version.major, version.minor
            ));
            if !binding_path.exists() {
                panic!("No pre-built bindings available for GEOS version {}.{}. Use `--features bindgen` to generate your own bindings.", version.major, version.minor);
            }

            std::fs::copy(&binding_path, &out_path)
                .expect("Can't copy bindings to output directory");
        }
    }
}
