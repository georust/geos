# geos-sys

Low level [GEOS](https://libgeos.org/) C API bindings for GEOS >= 3.7.0.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [georust/geos](https://github.com/georust/geos) crate.

You can also find it on [crates.io](https://crates.io/crates/geos).


## Build

By default, the build will use system-installed GEOS if available.

If using system-installed GEOS, the build can be configured with a few
environment variables:
* If `GEOS_INCLUDE_DIR`, `GEOS_LIB_DIR`, and `GEOS_VERSION` are set, they will
  be used
* otherwise, `pkg-config` (Linux / macOS) is queried to determine these values

You can build the included version of GEOS using the `static` feature, which
will also statically link libgeos to this crate.  In order to build GEOS, you
need to have `cmake` and a C++ compiler.


## Bindings

By default, prebuilt bindings are used if they match your version of GEOS.

If a prebuilt binding is not available, you can generate your own bindings using
the `bindgen` feature.


## Add more functions

This binding is written manually.

A little script is available to check what functions aren't available yet. You
can run it as follows:

```bash
> python3 check_missing/check_missing.py
```

It simply reads `geos` C header file and compare it with the `geos-sys`'s
`src/functions.rs` file. Normally, you should never have more functions in the
Rust code than the C code (deprecated functions aren't reexported in Rust).

If you want to support a newer GEOS version, please update the
`check_missing/geos_c.h` file and then run the `check_missing.py` script to see
what was added/removed.
