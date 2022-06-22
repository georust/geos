# geos-sys-bin

This create builds low level [GEOS](https://libgeos.org/) C API bindings for use
in `geos-sys`.

## Creating bindings

[bindgen](https://docs.rs/bindgen/latest/bindgen/) is used to automatically
create bindings from the GEOS C API.

You need to have the GEOS version for which you want to generate bindings
installed on your system. At minimum, you need to have the `geos_c.h` header
file for that version available (this is created by the GEOS build process).

This crate will attempt to automatically detect your installation of GEOS:

-   `pkg-config` is used to automatically detect GEOS >= 3.9
-   `geos-config` is used to automatically detect GEOS < 3.9

If GEOS is in a custom location, you can instead use environment variables to
configure GEOS detection (both must be set):

-   `GEOS_INCLUDE_DIR`
-   `GEOS_VERSION`

## Adding a new GEOS version

### 1. Generate new bindings

Install the desired GEOS version on your system and then run:

```bash
cargo run
```

This will produce a new binding in
`geos-sys/prebuilt-bindings/geos_<major>.<minor>.rs` based on the major and minor
version of your system-installed GEOS.

Review the contents of this file to determine if there are new bindings that
will be problematic to integrate in Rust, such as data types that vary by
architecture. Common data types are provided using `libc`. You can compare to
bindings from a previous version of GEOS for reference.

### 2. Add new feature

Add a new version feature for this GEOS version with the pattern
`"v<major>_<minor>_0"` to `Cargo.toml` in the root of this repository and
`sys/Cargo.toml`. The feature for each newer version of GEOS depends on the
previous version.

### 3. Update included version of GEOS

* update the GEOS submodule to the latest available GEOS version
* update `BUNDLED_GEOS_VERSION` in `sys/build.rs` to match this version