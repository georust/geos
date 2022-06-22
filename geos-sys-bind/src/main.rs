use bindgen::Builder;
use pkg_config::Config;
use semver::Version;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const MINIMUM_GEOS_VERSION: &str = "3.6.0";

struct GEOSConfig {
    include_dir: PathBuf,
    version: Version,
}

/// standardize GEOS prerelease versions to match semver format:
fn parse_geos_version(raw_version: &str) -> Version {
    Version::parse(
        &raw_version
            .trim()
            .replace("alpha", "-alpha")
            .replace("beta", "-beta")
            .replace("dev", "-dev"),
    )
    .expect("Could not parse GEOS version")
}

/// Detect GEOS config parameters using geos-config tool shipped with all compatible
/// versions of GEOS.
fn detect_geos_via_geos_config() -> Option<GEOSConfig> {
    let geos_config = Command::new("geos-config")
        .args(["--includes", "--version"])
        .output();

    match geos_config {
        Ok(config_output) => {
            let geos_config: Vec<&str> = std::str::from_utf8(&config_output.stdout)
                .unwrap()
                .split_whitespace()
                .collect();
            assert!(geos_config.len() == 2);

            Some(GEOSConfig {
                include_dir: PathBuf::from(geos_config[0].trim()),
                version: parse_geos_version(geos_config[1]),
            })
        }
        Err(_) => None,
    }
}

/// Detect GEOS config parameters using pkg-config (not available for all GEOS
/// versions)
fn detect_geos_via_pkg_config() -> Option<GEOSConfig> {
    let geos_pkg_config = Config::new()
        .atleast_version(MINIMUM_GEOS_VERSION)
        .cargo_metadata(false)
        .env_metadata(false)
        .probe("geos");

    match &geos_pkg_config {
        Ok(geos) => {
            // GEOS should only have one include path for geos_c.h header
            // include_path = PathBuf::from(geos.include_paths.first().unwrap());
            // version = parse_geos_version(&geos.version);
            Some(GEOSConfig {
                include_dir: PathBuf::from(geos.include_paths.first().unwrap()),
                version: parse_geos_version(&geos.version),
            })
        }
        Err(pkg_config_err) => {
            if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
            {
                println!("Could not find `pkg-config` in your path. Please install it before running geos-sys-bind.");
                exit(1);
            }

            None
        }
    }
}

/// Generate bindings based on GEOS header file
fn write_bindings(include_dir: &Path, out_path: &Path) {
    let geos_header = include_dir.join("geos_c.h").to_str().unwrap().to_string();

    println!("Generating bindings using GEOS header: {}", geos_header);

    Builder::default()
        .size_t_is_usize(true)
        .header(geos_header)
        .clang_arg("-I")
        .clang_arg(include_dir.to_str().unwrap())
        // use libc instead of default std::os::raw
        .ctypes_prefix("libc")
        // drop GEOS comments due to license constraints
        .generate_comments(false)
        // block deprecated APIs (both plain and "_r" variants)
        .blocklist_function("initGEOS")
        .blocklist_function("initGEOS_r")
        .blocklist_function("finishGEOS")
        .blocklist_function("finishGEOS_r")
        .blocklist_function("GEOSGeomFromWKT")
        .blocklist_function("GEOSGeomFromWKT_r")
        .blocklist_function("GEOSGeomToWKT")
        .blocklist_function("GEOSGeomToWKT_r")
        .blocklist_function("GEOSSingleSidedBuffer")
        .blocklist_function("GEOSSingleSidedBuffer_r")
        .blocklist_function("GEOSUnionCascaded")
        .blocklist_function("GEOSUnionCascaded_r")
        // TODO: remove; these were deprecated a long time ago but are still used here
        // .blocklist_function("GEOS_getWKBOutputDims")
        // .blocklist_function("GEOS_getWKBOutputDims_r")
        // .blocklist_function("GEOS_setWKBOutputDims")
        // .blocklist_function("GEOS_setWKBOutputDims_r")
        // .blocklist_function("GEOS_getWKBByteOrder")
        // .blocklist_function("GEOS_getWKBByteOrder_r")
        // .blocklist_function("GEOS_setWKBByteOrder")
        // .blocklist_function("GEOS_setWKBByteOrder_r")
        // .blocklist_function("GEOSGeomFromWKB_buf")
        // .blocklist_function("GEOSGeomFromWKB_buf_r")
        // .blocklist_function("GEOSGeomToWKB_buf")
        // .blocklist_function("GEOSGeomToWKB_buf_r")
        // .blocklist_function("GEOSGeomFromHEX_buf")
        // .blocklist_function("GEOSGeomFromHEX_buf_r")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");

    // replace f64 => c_double, f32 => c_float
    let mut content = fs::read_to_string(out_path).expect("Could not read generated bindings");
    content = content
        .replace(" f64", " libc::c_double")
        .replace(" f32", " libc::c_float");
    fs::write(out_path, content).expect("Unable to write bindings to file");

    println!("Bindings generated successfully; please review the results");
}

fn main() {
    let mut config: Option<GEOSConfig>;

    let include_dir_env = env::var_os("GEOS_INCLUDE_DIR");
    let version_env = env::var_os("GEOS_VERSION");

    if include_dir_env.is_some() || version_env.is_some() {
        // GEOS_INCLUDE_DIR
        let include_dir = match include_dir_env {
            Some(path) => PathBuf::from(path),
            None => {
                println!("GEOS_INCLUDE_DIR must be set");
                exit(1);
            }
        };

        // GEOS_VERSION
        let version = match version_env {
            Some(raw_version) => parse_geos_version(&raw_version.to_string_lossy().to_string()),
            None => {
                println!("GEOS_VERSION must be set");
                exit(1);
            }
        };

        config = Some(GEOSConfig {
            include_dir,
            version,
        })
    } else {
        // try to detect using pkg-config, if available
        config = detect_geos_via_pkg_config();

        // fall back to try using geos-config
        if config.is_none() {
            config = detect_geos_via_geos_config();
        }

        if config.is_none() {
            println!("ERROR: could not detect GEOS using pkg-config or geos-config");
            exit(1);
        }
    }

    let detected = config.unwrap();
    let version = detected.version;

    let min_geos_version = Version::parse(MINIMUM_GEOS_VERSION).unwrap();
    if version < min_geos_version {
        println!(
            "ERROR: GEOS version {}.{}.{} is older than the minimum supported version {}.{}.{}",
            version.major,
            version.minor,
            version.patch,
            min_geos_version.major,
            min_geos_version.minor,
            min_geos_version.patch
        );
        exit(1);
    }

    let out_path = PathBuf::from(format!(
        "../sys/prebuilt-bindings/geos_{}.{}.rs",
        version.major, version.minor
    ));

    // confirm if output already exists
    if out_path.exists() {
        println!("\n\n=======================");
        println!(
            "Prebuilt bindings already exist for GEOS {}.{}\nDo you want to overwrite it (y/n)?",
            version.major, version.minor
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.to_string().to_lowercase().trim() != "y" {
            println!("exiting...");
            exit(0);
        }
    }

    write_bindings(&detected.include_dir, &out_path);
}
