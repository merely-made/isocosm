// SPDX-License-Identifier: MPL-2.0

//! Rewrites chronicle files in place at the current version, through the same
//! migration a reader applies at load (case 1040).
//!
//! ```text
//! cargo run --example upgrade_chronicle -- <file.chronicle>...
//! ```

use wing_formats::{CHRONICLE_MAGIC, CHRONICLE_VERSION, frame, peek, read_chronicle};

fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("the chronicle is readable");
        let from = peek(CHRONICLE_MAGIC, &bytes).expect("the file is a chronicle");
        let chronicle = read_chronicle(&bytes).expect("the chronicle is a known version");
        let written = frame(CHRONICLE_MAGIC, CHRONICLE_VERSION, &chronicle).expect("encodable");
        std::fs::write(&path, &written).expect("the chronicle is writable");
        println!("{path}: v{from} -> v{CHRONICLE_VERSION}, {} deeds", chronicle.deeds.len());
    }
}
