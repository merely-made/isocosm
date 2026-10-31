// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The core's inspection endpoint over stdio on the fixture world, taking no
//! arguments, so Graphshell's `g4_sessions` receipt can mount it (H2).

use cosmograph_host::{endpoint::SimEndpoint, fixtures::fixture};

fn main() -> Result<(), String> {
    let mut endpoint = SimEndpoint::new(fixture().world, None);
    graphshell_endpoint::stdio::serve_basic(&mut endpoint, std::io::stdin().lock(), std::io::stdout().lock())
        .map_err(|e| format!("{e:?}"))
}
