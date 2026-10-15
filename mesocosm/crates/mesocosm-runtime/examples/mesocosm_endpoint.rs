// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The played-turn endpoint over stdio on the core's fixture world, taking no
//! arguments, so Graphshell's `g4_sessions` receipt can mount it (H2).

use isocosm_host::fixtures::fixture;
use mesocosm_runtime::endpoint::MesocosmEndpoint;

fn main() -> Result<(), String> {
    let f = fixture();
    let mut endpoint = MesocosmEndpoint::open(&f.world, f.participants[0])?;
    graphshell_endpoint::stdio::serve_basic(&mut endpoint, std::io::stdin().lock(), std::io::stdout().lock())
        .map_err(|e| format!("{e:?}"))
}
