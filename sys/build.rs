use semver::Version;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const MINIMUM_GEOS_VERSION: &str = "3.6.0";
const BUNDLED_GEOS_VERSION: &str = "3.11.0";

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
fn detect_geos_via_geos_config() -> Option<Version> {
    let geos_config = Command::new("geos-config")
        .args(["--ldflags", "--version"])
        .output();

    match geos_config {
        Ok(config_output) => {
            let geos_config: Vec<&str> = std::str::from_utf8(&config_output.stdout)
                .unwrap()
                .split_whitespace()
                .collect();
            assert!(geos_config.len() == 2);

            println!("cargo:rustc-link-lib=dylib=geos_c");

            println!(
                "cargo:rustc-link-search=native={}",
                geos_config[0].replace("-L", "")
            );

            Some(parse_geos_version(geos_config[1]))
        }
        Err(_) => None,
    }
}

/// Detect GEOS config parameters using pkg-config (not available for all GEOS
/// versions)
fn detect_geos_via_pkg_config() -> Option<Version> {
    use pkg_config::Config;

    let geos_pkg_config = Config::new()
        .atleast_version(MINIMUM_GEOS_VERSION)
        .probe("geos");

    match &geos_pkg_config {
        Ok(geos) => {
            // GEOS should only have one include path for geos_c.h header
            Some(parse_geos_version(&geos.version))
        }
        Err(pkg_config_err) => {
            if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
            {
                panic!("Could not find `pkg-config` in your path. Please install it before running geos-sys-bind.");
            }

            None
        }
    }
}

#[cfg(feature = "dox")]
fn main() {
    let binding_version = Version::parse(BUNDLED_GEOS_VERSION).expect("Could not parse bundled GEOS version");

    println!(
        "cargo:rustc-cfg=geos_sys_{}_{}",
        binding_version.major, binding_version.minor
    );
}

#[cfg(not(feature = "dox"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GEOS_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GEOS_VERSION");

    let mut version: Option<Version>;
    let lib_dir_env = env::var_os("GEOS_LIB_DIR");
    let version_env = env::var_os("GEOS_VERSION");

    // static feature includes building the included GEOS prior to this build step.
    // The statically-linked GEOS is the version pinned in the GEOS submodule
    // in geos-src/source
    if cfg!(feature = "static") {
        let geos_path = std::env::var("DEP_GEOSSRC_SEARCH").unwrap();

        // Note: static lib "geos_c" isn't available for GEOS 3.7.x
        println!("cargo:rustc-link-lib=static=geos_c");
        println!("cargo:rustc-link-lib=static=geos");
        println!("cargo:rustc-link-search=native={}", geos_path);
        println!("cargo:includedir={}/include", geos_path);

        version = Some(
            Version::parse(BUNDLED_GEOS_VERSION).expect("Could not parse bundled GEOS version"),
        );
    } else if lib_dir_env.is_some() || version_env.is_some() {
        // if any env vars are set, all must be set
        println!("cargo:rustc-link-lib=dylib=geos_c");

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
                version = Some(parse_geos_version(&raw_version.to_string_lossy()));
            }
            None => {
                panic!("GEOS_VERSION must be set");
            }
        }
    } else {
        // try to detect using pkg-config, if available
        version = detect_geos_via_pkg_config();

        // fall back to try using geos-config
        if version.is_none() {
            version = detect_geos_via_geos_config();
        }

        if version.is_none() {
            panic!("Could not detect GEOS using pkg-config or geos-config");
        }
    }

    let version = version.unwrap();

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

    // resolve user-requested version (via specific version feature, e.g., "v3_10")
    // to the correct pre-built binding; their available GEOS must be >= requested
    // pre-built binding version

    let mut binding_version = Version::parse(MINIMUM_GEOS_VERSION).unwrap();

    if cfg!(feature = "v3_7_0") {
        binding_version = Version::new(3, 7, 0);
    }

    if cfg!(feature = "v3_8_0") {
        binding_version = Version::new(3, 8, 0);
    }

    if cfg!(feature = "v3_9_0") {
        binding_version = Version::new(3, 9, 0);
    }

    if cfg!(feature = "v3_10_0") {
        binding_version = Version::new(3, 10, 0);
    }

    if cfg!(feature = "v3_11_0") {
        binding_version = Version::new(3, 11, 0);
    }

    if version < binding_version {
        panic!("You requested a version of GEOS ({}.{}) that is greater than your installed GEOS version ({}.{}.{})", binding_version.major, binding_version.minor, version.major, version.minor, version.patch);
    }

    println!(
        "cargo:rustc-cfg=geos_sys_{}_{}",
        binding_version.major, binding_version.minor
    );
}
