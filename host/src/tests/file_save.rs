// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{fs::{self, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}};
use crate::file_save;

fn owner() -> PathBuf {
    let path = PathBuf::from(std::env::var_os("ISOCOSM_HOST_ATOMIC_SAVE_DIR")
        .expect("atomic-save gates require an explicit existing file owner"));
    assert!(path.is_dir(), "atomic-save gates never create an owner directory");
    path
}

fn fixture_file(bytes: &[u8]) -> tempfile::TempPath {
    let mut file = tempfile::Builder::new().prefix("q14-save-fixture-")
        .suffix(".save").tempfile_in(owner()).unwrap();
    file.write_all(bytes).unwrap();
    file.as_file().sync_all().unwrap();
    file.into_temp_path()
}

fn failure() -> io::Error { io::Error::other("declared precommit failure") }

#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn partial_staging_failure_preserves_previous_save_and_retires_only_own_stage() {
    let old = b"previous native and opaque mode bytes\0\xff";
    let destination = fixture_file(old);
    let sibling = destination.with_extension("pending");
    let mut sentinel = OpenOptions::new().write(true).create_new(true).open(&sibling).unwrap();
    sentinel.write_all(b"unrelated pending file").unwrap();
    drop(sentinel);
    let sentinel = tempfile::TempPath::try_from_path(sibling).unwrap();
    let mut stage_path = None;
    let result = file_save::with_failures(destination.as_ref(), b"replacement native bytes", |path, file, bytes| {
        assert_eq!(path.parent(), destination.parent(), "staging stays in the save directory");
        stage_path = Some(path.to_path_buf());
        file.write_all(&bytes[..bytes.len() / 2])?;
        Err(failure())
    }, |_| panic!("partial staging cannot reach publication"));
    assert!(result.is_err());
    assert_eq!(fs::read(&destination).unwrap(), old,
        "partial staging failure preserves every previous-save byte");
    assert!(!stage_path.unwrap().exists(), "failed own staging file is retired");
    assert_eq!(fs::read(&sentinel).unwrap(), b"unrelated pending file",
        "exclusive staging never truncates an unrelated pending file");
}

#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn complete_stage_is_exclusive_and_closed_before_a_precommit_refusal() {
    let old = b"old complete save";
    let destination = fixture_file(old);
    let new = b"new complete native and opaque mode bytes\0\xff";
    let mut stage_path = None;
    let result = file_save::with_failures(destination.as_ref(), new, |_, file, bytes| file.write_all(bytes), |path| {
        stage_path = Some(path.to_path_buf());
        assert_eq!(fs::read(path).unwrap(), new, "publication sees complete staged bytes");
        let collision = OpenOptions::new().write(true).create_new(true).open(path);
        assert_eq!(collision.unwrap_err().kind(), io::ErrorKind::AlreadyExists,
            "staging creation is exclusive");
        #[cfg(windows)] {
            use std::os::windows::fs::OpenOptionsExt;
            let closed = OpenOptions::new().read(true).share_mode(0).open(path);
            assert!(closed.is_ok(), "complete staging file is closed before publication");
        }
        assert_eq!(fs::read(&destination).unwrap(), old, "old save survives every precommit phase");
        Err(failure())
    });
    assert!(result.is_err());
    assert_eq!(fs::read(&destination).unwrap(), old);
    assert!(!stage_path.unwrap().exists());
}

#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn replacement_publishes_exact_bytes_and_can_replace_the_same_path_again() {
    let destination = fixture_file(b"previous save");
    let bytes = b"complete native history\0opaque mode\xff";
    file_save::replace(destination.as_ref(), bytes).unwrap();
    assert_eq!(fs::read(&destination).unwrap(), bytes);
    file_save::replace(destination.as_ref(), b"second complete save").unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"second complete save");
}

#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn missing_parent_refuses_without_creating_a_directory() {
    let occupied = fixture_file(b"occupied file is not a parent directory");
    let path = Path::new(&*occupied).join("world.save");
    assert!(file_save::replace(&path, b"replacement").is_err());
    assert_eq!(fs::read(&occupied).unwrap(), b"occupied file is not a parent directory");
    assert!(!path.exists());
}

#[cfg(windows)]
#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn refused_windows_replacement_keeps_previous_bytes_after_the_stage_is_closed() {
    use std::os::windows::fs::OpenOptionsExt;
    let old = b"previous Windows save";
    let destination = fixture_file(old);
    let lock = OpenOptions::new().read(true).share_mode(0).open(&destination).unwrap();
    let mut stage_path = None;
    let result = file_save::with_failures(destination.as_ref(), b"new save",
        |_, file, bytes| file.write_all(bytes), |path| {
            stage_path = Some(path.to_path_buf());
            Ok(())
        });
    assert!(result.is_err(), "an exclusively open destination refuses replacement");
    drop(lock);
    assert_eq!(fs::read(&destination).unwrap(), old, "failed replacement preserves previous bytes");
    assert!(!stage_path.unwrap().exists(), "failed publication retires only its staging path");
}

#[cfg(all(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn app_save_round_trip_retains_native_replay_and_two_independent_mode_windows() {
    use super::*;
    use crate::{products::Mode, save::WorldSave, ui::model::{App, Defaults, Preferences}};
    let f = fixture();
    let mut source = f.world.save().unwrap();
    let absent = vec![0, 255, 11, 0, 128];
    source.sections.insert("unbuilt-atomic-save-mode".into(), absent.clone());
    let destination = fixture_file(&source.encode().unwrap());
    let mut app = App::new(World::resume(source, Execution::Grouped).unwrap(),
        Some(destination.to_path_buf()));
    let first = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let second = app.open(f.participants[0], Seat::Player, Mode::Eponym, Preferences::default(), None).unwrap();
    app.turn(first).unwrap(); app.turn(second).unwrap(); app.turn(second).unwrap();
    let handles = [app.windows[&first].native.handle().unwrap(), app.windows[&second].native.handle().unwrap()];
    let expected = app.snapshot().unwrap();
    app.save().unwrap();
    let bytes = fs::read(&destination).unwrap();
    assert_eq!(bytes, expected.encode().unwrap(), "App publishes its complete snapshot bytes");
    let saved = WorldSave::decode(&bytes).unwrap();
    assert_eq!(saved, expected);
    assert_eq!(saved.sections["unbuilt-atomic-save-mode"], absent);
    let mut restored = App::new(World::resume(saved.clone(), Execution::Individuals).unwrap(), None);
    let a = restored.open_with_saved_preferences(f.participants[0], Seat::Player, Mode::Mesocosm, None, Some(handles[0])).unwrap();
    let b = restored.open_with_saved_preferences(f.participants[0], Seat::Player, Mode::Eponym, None, Some(handles[1])).unwrap();
    let angle = |key| serde_json::from_slice::<Defaults>(&restored.windows[&key].native.binding().view_defaults)
        .unwrap().quarter_turn;
    assert_eq!((angle(a), angle(b)), (1, 2));
    assert_eq!(restored.world.save().unwrap(), saved, "file save and restore add no native commands");
    assert_eq!(body(&f, f.subjects[0]), restored.world.read(|s|
        s.sim.state().population.get(f.subjects[0]).unwrap().clone()).unwrap());
}
