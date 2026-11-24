// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

/// Open the native founding bench beside the current game.
pub(super) fn open(seed: u64) -> Result<(), String> {
    std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .args(["--founding-editor", "--seed", &seed.to_string()])
        .spawn().map(|_| ()).map_err(|e| format!("Cannot open the founding editor: {e}"))
}
