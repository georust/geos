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

## Adding a new GEOS version

### 1. Generate new bindings

By default, the bindings are generated against your installed version of GEOS:

```bash
cargo run
```

You can also use the `-h` / `--header` command line flag to specify the location
of the GEOS header file:

```bash
cargo run -- --header <path_to_geos_c.h>
```

This will produce a new binding in
`geos-sys/prebuilt-bindings/geos_<major>.<minor>.rs` based on the major and minor
version of your system-installed GEOS.

Review the contents of this file to determine if there are new bindings that
will be problematic to integrate in Rust, such as data types that vary by
architecture. Common data types are provided using `libc`. You can compare to
bindings from a previous version of GEOS for reference.

### 2. Add entry to `build.rs`

Add a new entry with the following pattern toward the end of `build.rs` to
enable this binding version:

```rust
if cfg!(feature = "v<major>_<minor>_0") {
    binding_version = Version::new(<major>, <minor>, 0);
}
```

### 3. Update `lib.rs`

Add a new cfg entry to `lib.rs` with the following pattern to enable binding
against this version:

```rust
#[cfg(geos_sys_<major>_<minor>)]
include!("../prebuilt-bindings/geos_<major>.<minor>.rs");
```

Update the GEOS version number in the docstring that is used for referencing the
version of GEOS that is included in the docs; it should be based on the latest
version of GEOS.

### 4. Add feature entry for new version

Add a new feature entry for this GEOS version with the pattern
`"v<major>_<minor>_0"` to `Cargo.toml` in the root of this repository and
`sys/Cargo.toml`. The feature for each newer version of GEOS depends on the
previous version.

### 5. Update included version of GEOS

* update the GEOS submodule to the latest available GEOS version
* update `BUNDLED_GEOS_VERSION` in `sys/build.rs` to match this version