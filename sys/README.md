# geos-sys

[GEOS](https://trac.osgeo.org/geos/) C API bindings.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
prefer to use the [georust/geos](https://github.com/georust/geos) crate.

You can also find it on [crates.io](https://crates.io/crates/geos).

## Static build

If you want to link statically to libgeos, then use the `static` feature. It will build `libgeos` so you need to have `cmake` and a C++ compiler.

## Add more functions

A little script is available to check what functions aren't available yet. You can run it as follows:

```bash
> python3 check_missing/check_missing.py
```

It simply reads `geos` C header file and compare it with the `geos-sys`'s `src/functions.rs` file. Normally, you should never have more functions in the Rust code than the C code (deprecated functions aren't reexported in Rust).
