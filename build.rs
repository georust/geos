#[cfg(feature = "geo")]
extern crate skeptic;

#[cfg(feature = "geo")]
fn main() {
    // generates doc tests for `README.md`.
    skeptic::generate_doc_tests(&["README.md"]);
}

#[cfg(not(feature = "geo"))]
fn main() {}
