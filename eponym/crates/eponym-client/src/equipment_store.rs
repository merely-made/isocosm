// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit immutable saves for equipment state, not a game-wide save service.
//!
//! This was `body_sheet::equipment_store` until the body sheet was retired
//! (M5 of the isomere plan). Everything else under `body_sheet` was GUI
//! machinery the shared stack already owns; this is product state, so it moved
//! rather than died, unchanged apart from its second test's fixture. It keeps
//! its own discipline deliberately: a save is *published*, never replaced, and
//! a load reads the newest published one and refuses to fall back past a
//! broken save. The file is written under a `.pending` name and hard-linked
//! into place, so a reader can never observe a half-written save on any
//! supported platform.
//!
//! Nothing calls it today. Its only consumer was the retired `body_sheet`
//! binary's Save/Load; the `session` host writes one `session.save` through a
//! pending-then-rename path of its own in `bin/session/actions.rs`, which is a
//! different discipline — one mutable slot rather than an immutable series.
//! Reconciling the two is a ruling about what a Eponym save *is*, not a
//! retirement decision, so the two paths stand side by side and this one keeps
//! its receipts.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Publish a new snapshot. Existing saves are never replaced.
pub fn save_equipment(directory: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("save exceeds 16 MiB limit".into());
    }
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let name = format!(
        "equipment-{nanos:032}-{}-{:016}.bin",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let path = directory.join(name);
    let pending = path.with_extension("pending");
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        // A hard link publishes only a completed file and refuses overwrite on
        // every supported platform. A crash can leave a harmless pending link.
        fs::hard_link(&pending, &path)?;
        Ok(())
    })();
    if result.is_ok() {
        let _ = fs::remove_file(&pending);
    }
    result.map_err(|e| e.to_string())?;
    Ok(path)
}

/// Read the newest published save. Never silently fall back past a broken save.
pub fn load_equipment(directory: &Path) -> Result<(PathBuf, Vec<u8>), String> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with("equipment-") && name.ends_with(".bin") {
            candidates.push(entry.path());
        }
    }
    candidates.sort();
    let path = candidates
        .pop()
        .ok_or("no equipment saves in this directory")?;
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("save exceeds 16 MiB limit".into());
    }
    Ok((path, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eponym_world::GameState;
    use eponym_world::fixtures::session as session_fixture;

    fn scratch(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "paredros-equipment-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn saves_are_immutable_and_pending_files_are_not_loaded() {
        let directory = scratch("immutable");
        let first = save_equipment(&directory, b"first").unwrap();
        let second = save_equipment(&directory, b"second").unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(&first).unwrap(), b"first");
        let pending = directory.join("equipment-zzzz.pending");
        fs::write(&pending, b"incomplete").unwrap();
        assert_eq!(
            load_equipment(&directory).unwrap(),
            (second.clone(), b"second".to_vec())
        );
        for path in [first, second, pending] {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }

    /// The claim the retired `body_sheet` binary's two-process persistence
    /// smoke made, at the unit tier and over the session's own world: what a
    /// published save restores is the same accepted game, attachment included.
    #[test]
    fn disk_roundtrip_restores_attachment_into_a_fresh_game() {
        let directory = scratch("roundtrip");
        let fixture = session_fixture::timed_action_world();
        let game = fixture.action.session().game();
        let worn = game.attachments(fixture.target);
        assert_eq!(worn.len(), 1);
        assert!(worn[0].current);

        let path = save_equipment(&directory, &game.save().unwrap()).unwrap();
        let (published, bytes) = load_equipment(&directory).unwrap();
        assert_eq!(published, path);
        let restored = GameState::restore(&bytes).unwrap();
        assert_eq!(&restored, game);
        assert_eq!(restored.attachments(fixture.target), worn);

        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
