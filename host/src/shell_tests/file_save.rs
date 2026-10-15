// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{fs, io::Write, path::PathBuf};
use super::*;

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

#[test]
#[ignore = "requires an explicit existing ISOCOSM_HOST_ATOMIC_SAVE_DIR owner"]
fn app_save_round_trip_retains_native_replay_and_two_independent_mode_windows() {
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
