#![cfg(not(test))]
extern crate geos;
use geos::version;

fn main() {
    println!("geos_c version: {}", version());
}
