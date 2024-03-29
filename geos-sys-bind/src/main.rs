use bindgen::Builder;
use clap::Parser;
use pkg_config::Config;
use regex::Regex;
use semver::Version;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const MINIMUM_GEOS_VERSION: &str = "3.6.0";

struct GEOSConfig {
    header: String,
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
                header: PathBuf::from(geos_config[0].trim())
                    .join("geos_c.h")
                    .to_str()
                    .unwrap()
                    .to_string(),
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
            Some(GEOSConfig {
                header: PathBuf::from(geos.include_paths.first().unwrap())
                    .join("geos_c.h")
                    .to_str()
                    .unwrap()
                    .to_string(),
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
fn write_bindings(geos_header: &str, out_path: &Path) {
    println!("Generating bindings using GEOS header: {}", geos_header);

    Builder::default()
        .size_t_is_usize(true)
        .header(geos_header)
        // use libc instead of default std::os::raw
        .ctypes_prefix("libc")
        // avoid converting double / float to f64 / f32
        .no_convert_floats()
        // drop GEOS comments due to license constraints
        .generate_comments(false)
        // block strings that aren't handed properly and can be trivially generated later
        .blocklist_item("GEOS_VERSION")
        .blocklist_item("GEOS_CAPI_VERSION")
        // block unnecessary consts
        .blocklist_item("GEOS_JTS_PORT")
        .blocklist_item("GEOS_CAPI_FIRST_INTERFACE")
        .blocklist_item("GEOS_CAPI_LAST_INTERFACE")
        // block deprecated APIs (both plain and "_r" variants)
        .blocklist_function("initGEOS.*")
        .blocklist_function("finishGEOS.*")
        .blocklist_function("GEOSGeomFromWKT.*")
        .blocklist_function("GEOSGeomToWKT.*")
        .blocklist_function("GEOSSingleSidedBuffer.*")
        .blocklist_function("GEOSUnionCascaded.*")
        // TODO: remove; these were deprecated a long time ago but are still used here
        // .blocklist_function("GEOS_getWKBOutputDims.*")
        // .blocklist_function("GEOS_setWKBOutputDims.*")
        // .blocklist_function("GEOS_getWKBByteOrder.*")
        // .blocklist_function("GEOS_setWKBByteOrder.*")
        // .blocklist_function("GEOSGeomFromWKB_buf.*")
        // .blocklist_function("GEOSGeomToWKB_buf.*")
        // .blocklist_function("GEOSGeomFromHEX_buf.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");

    println!("Bindings generated successfully; please review the results");
}

#[derive(Parser, Debug)]
#[clap(about)]
struct Args {
    /// GEOS geos_c.h header file path
    #[clap(short = 'h', long = "header")]
    header: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut config: Option<GEOSConfig>;

    if args.header.is_some() {
        let header_path = args.header.unwrap();

        if !header_path.exists() {
            println!("header path {:?} does not exist", header_path);
            exit(1);
        }

        if !header_path.is_file() {
            println!("header path {:?} is not a file", header_path);
            exit(1);
        }

        let header = header_path
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Extract version from header; always follows a consistent pattern
        let content = fs::read_to_string(&header).expect("Could not read GEOS header file");
        let re = Regex::new(r#"define GEOS_VERSION "\S+""#).unwrap();
        let raw_version = re
            .find(&content)
            .map(|x| {
                let mut split = x.as_str().split('"');
                split.next();
                split.next().unwrap()
            })
            .expect("Could not read GEOS_VERSION from GEOS header file");

        config = Some(GEOSConfig {
            header,
            version: parse_geos_version(raw_version),
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
            "Prebuilt bindings already exist for GEOS {}.{}\nDo you want to overwrite it (y/N)?",
            version.major, version.minor
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.to_string().to_lowercase().trim() != "y" {
            println!("exiting...");
            return;
        }
    }

    write_bindings(&detected.header, &out_path);
}
