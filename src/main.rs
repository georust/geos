#![cfg(not(test))]
extern crate geos;
use geos::{version, init, finish};

fn main() {
    init();
    println!("geos_c version: {}", version());
    finish();
}
