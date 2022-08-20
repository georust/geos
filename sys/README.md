# geos-sys

Low level [GEOS](https://libgeos.org/) C API bindings for GEOS >= 3.6.0.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [georust/geos](https://github.com/georust/geos) crate.

You can also find it on [crates.io](https://crates.io/crates/geos).

## Build

By default, the build will use system-installed GEOS if available:

-   `pkg-config` is used to automatically detect GEOS >= 3.9
-   `geos-config` is used to automatically detect GEOS < 3.9

If GEOS is in a custom location, you can instead use environment variables to
configure GEOS detection (both must be set):

-   `GEOS_LIB_DIR`
-   `GEOS_VERSION`

If `GEOS_LIB_DIR` is not also in your system's standard dynamic library search
path, you may need to add it to the dynamic library search path before
running the tests or executable produced by `cargo build`.

Linux:

```bash
LD_LIBRARY_PATH=<path to GEOS>/lib GEOS_LIB_DIR=<path to GEOS>/lib GEOS_VERSION=<version> cargo test

```

MacOS:

```bash
DYLD_FALLBACK_LIBRARY_PATH=<path to GEOS>/lib GEOS_LIB_DIR=<path to GEOS>/lib GEOS_VERSION=<version> cargo test

```

You can build the included version of GEOS using the `static` feature, which
will also statically link libgeos to this crate. In order to build GEOS, you
need to have `cmake` and a C++ compiler. Building GEOS may take several minutes.

## Bindings

Pre-built bindings are available for all supported GEOS versions.

Use the version feature for the version of GEOS that you want to target; your
installed version of GEOS must be greater than or equal to this version.

Example:

```bash
cargo build --features v3_8_0
```

New bindings can be created using the sibling `geos-sys-bind` crate.
