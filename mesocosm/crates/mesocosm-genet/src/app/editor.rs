// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{env, path::PathBuf, process::Command};

/// Open the local founding workshop. The current world stays in its window.
pub(super) fn open() -> Result<(), String> {
    let game = env::current_exe().map_err(|why| why.to_string())?;
    let editor =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/lineage-editor/serve.py");
    let dir = env::var_os("MESOCOSM_EDITOR_BIN_DIR")
        .map(PathBuf::from)
        .or_else(|| game.parent().map(PathBuf::from))
        .ok_or("missing binary directory")?;
    if !editor.is_file() {
        return Err("The founding editor is missing from this checkout".into());
    }
    let started = Command::new(if cfg!(windows) { "python" } else { "python3" })
        .arg(editor)
        .arg("--open")
        .arg("--from-menu")
        .arg("--bin-dir")
        .arg(dir)
        .arg("--game")
        .arg(game)
        .output()
        .map_err(|why| format!("Cannot open the lineage editor: {why}"))?;
    if started.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&started.stderr).trim().to_owned())
    }
}
