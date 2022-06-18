# geos-sys

Low level [GEOS](https://libgeos.org/) C API bindings for GEOS >= 3.7.0.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [georust/geos](https://github.com/georust/geos) crate.

You can also find it on [crates.io](https://crates.io/crates/geos).


## Build

By default, the build will use system-installed GEOS if available.  `pkg-config`
is used to automatically detect GEOS >= 3.9.

If using system-installed GEOS not discoverable by `pkg-config` (GEOS <= 3.8 or
in a custom location), it will attempt to use `geos-config` instead.

The build can also be configured with a few environment
variables (all must be set):
* `GEOS_INCLUDE_DIR`
* `GEOS_LIB_DIR`
* `GEOS_VERSION`

You can build the included version of GEOS using the `static` feature, which
will also statically link libgeos to this crate.  In order to build GEOS, you
need to have `cmake` and a C++ compiler.  Building GEOS may take several minutes.


## Bindings

By default, prebuilt bindings are used if they match your version of GEOS.

If a prebuilt binding is not available, you can generate your own bindings using
the `bindgen` feature.

### Adding a new GEOS version

Install the desired GEOS version on your system and then run

```bash
cargo build --features bindgen
```

This will produce a new binding in `target/debug/build/geos-sys-<hash>/out/bindings.rs`.

Copy this to `prebuilt-bindings/geos_<major>.<minor>.rs`.


Alternatively, you can check the GEOS submodule in out `geos-src/source` out
to a particular version, and then use the `static` feature:

```bash
cargo build --features bindgen,static
```

Note that this may encounter build errors depending on the version of GEOS due
to CMake configuration issues.  You may need to switch
`.define("BUILD_TESTING", "OFF")` in `geos-src/src/build.rs` to `"ON"` in order
to successfully build using CMake.