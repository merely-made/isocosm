// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The VTT endpoint at the DM's seat over stdio on the table world, taking no
//! arguments, so Graphshell's `g4_sessions` receipt can mount it (H2).

#[path = "../tests/support/table.rs"]
mod table;

use cosmograph_overlay::Seat;
use isocosm_native::VttEndpoint;

fn main() -> Result<(), String> {
    let t = table::table();
    let mut endpoint = VttEndpoint::open(&t.world, t.dm, Seat::Dm)?;
    graphshell_endpoint::stdio::serve_basic(&mut endpoint, std::io::stdin().lock(), std::io::stdout().lock())
        .map_err(|e| format!("{e:?}"))
}
