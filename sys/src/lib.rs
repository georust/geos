//! Low level [GEOS](https://libgeos.org/) C API bindings for GEOS >= 3.7.0.
//!
//! It provides C-interface as is. If you want to use a more Rust-friendly crate,
//! use the [georust/geos](https://github.com/georust/geos) crate.

//! You can also find it on [crates.io](https://crates.io/crates/geos).
//!
//! By default, the build will use system-installed GEOS if available.
//!
//! You can build the included version of GEOS using the `static` feature, which
//! will also statically link libgeos to this crate.  In order to build GEOS, you
//! need to have `cmake` and a C++ compiler.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]

extern crate libc;

#[cfg(feature = "static")]
extern crate link_cplusplus;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
