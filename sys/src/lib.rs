//! [GEOS](https://trac.osgeo.org/geos/) C API bindings.
//!
//! It provides C-interface as is. If you want to use a more Rust-friendly crate,
//! prefer to use the [georust/geos](https://github.com/georust/geos) crate.
//!
//! You can also find it on [crates.io](https://crates.io/crates/geos).
//!
//! ## Static build
//!
//! If you want to link statically to libgeos, then use the `static` feature. It will build
//!`libgeos` so you need to have `cmake` and a C++ compiler.

extern crate libc;

#[cfg(feature = "static")]
extern crate link_cplusplus;

pub use functions::*;
pub use types::*;

mod functions;
mod types;
