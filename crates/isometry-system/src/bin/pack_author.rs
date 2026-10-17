//! Private terminal consumer for authored generator-pack manifests.
#![deny(clippy::all)]
#[path = "../pack_author/mod.rs"]
mod author;
fn main() {
    if let Err(error) = author::run() {
        eprintln!("pack-author: {error}");
        std::process::exit(1);
    }
}
