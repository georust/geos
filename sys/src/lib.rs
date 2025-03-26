//! Low level [GEOS](https://libgeos.org/) C API bindings for GEOS >= 3.6.0.
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
//!
//! This documentation is generated based on GEOS 3.11.  Please see the
//! [GEOS Changelog](https://github.com/libgeos/geos/blob/main/NEWS.md) for
//! a listing of which entries were added for each GEOS version.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]

extern crate libc;

#[cfg(feature = "static")]
extern crate link_cplusplus;

#[cfg(not(any(feature = "v3_7_0", feature = "dox")))]
include!("../prebuilt-bindings/geos_3.6.rs");

#[cfg(all(feature = "v3_7_0", not(any(feature = "v3_8_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.7.rs");

#[cfg(all(feature = "v3_8_0", not(any(feature = "v3_9_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.8.rs");

#[cfg(all(feature = "v3_9_0", not(any(feature = "v3_10_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.9.rs");

#[cfg(all(feature = "v3_10_0", not(any(feature = "v3_11_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.10.rs");

#[cfg(all(feature = "v3_11_0", not(any(feature = "v3_10_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.11.rs");

#[cfg(all(feature = "v3_12_0", not(any(feature = "v3_11_0", feature = "dox"))))]
include!("../prebuilt-bindings/geos_3.12.rs");

#[cfg(any(feature = "v3_13_0", feature = "dox"))]
include!("../prebuilt-bindings/geos_3.13.rs");
