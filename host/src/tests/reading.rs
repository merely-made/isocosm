// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{mode_sections, products::Mode, ui::model::{App, Defaults, Preferences}};
use isocosm_overlay::{AttentionChange, PlaceHandle};

fn app() -> (App, u64, u64, Fixture) {
    let f = fixture();
    let mut app = App::new(f.world.share(), None);
    let first = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let second = app.open(f.participants[0], Seat::Player, Mode::Eponym, Preferences::default(), None).unwrap();
    (app, first, second, f)
}

#[test]
fn mode_sections_restore_each_native_window_defaults_without_new_history() {
    let (mut app, first, second, f) = app();
    app.turn(first).unwrap();
    app.turn(second).unwrap(); app.turn(second).unwrap();
    let handles = [app.windows[&first].native.handle().unwrap(), app.windows[&second].native.handle().unwrap()];
    let native = app.world.save().unwrap().native;
    let saved = app.snapshot().unwrap();
    assert_eq!(saved.native, native, "mode view memory adds no native replay entries");
    let world = World::resume(saved.clone(), Execution::Grouped).unwrap();
    let mut restored = App::new(world, None);
    let a = restored.open_with_saved_preferences(f.participants[0], Seat::Player, Mode::Mesocosm, None, Some(handles[0])).unwrap();
    let b = restored.open_with_saved_preferences(f.participants[0], Seat::Player, Mode::Eponym, None, Some(handles[1])).unwrap();
    let read = |key| serde_json::from_slice::<Defaults>(&restored.windows[&key].native.binding().view_defaults).unwrap().quarter_turn;
    assert_eq!((read(a), read(b)), (1, 2), "saved cameras remain independent for actual native tokens");
    assert_eq!(restored.world.save().unwrap(), saved);
}

#[test]
fn inactive_mode_memory_survives_save_and_remains_non_authoritative_until_switch() {
    let (mut app, first, _, f) = app();
    let site = *app.windows[&first].native.projection().unwrap().visible.sites.iter().next().unwrap();
    app.examine(first, Some(site)).unwrap(); app.turn(first).unwrap();
    app.switch(first, Mode::Vtt).unwrap();
    let handle = app.windows[&first].native.handle().unwrap();
    let saved = app.snapshot().unwrap();
    let mut restored = App::new(World::resume(saved.clone(), Execution::Grouped).unwrap(), None);
    let key = restored.open_with_saved_preferences(f.participants[0], Seat::Player, Mode::Vtt, None, Some(handle)).unwrap();
    assert_eq!(restored.world.save().unwrap(), saved);
    restored.switch(key, Mode::Mesocosm).unwrap();
    let local = &restored.windows[&key];
    assert_eq!(local.native.binding().attention.examined, Some(PlaceHandle(site)));
    assert_eq!(serde_json::from_slice::<Defaults>(&local.native.binding().view_defaults).unwrap().quarter_turn, 1);
    assert!(local.native.projection().unwrap().attention.examined.contains(&PlaceHandle(site)),
        "restored inactive mode watch enters attention only through native switch admission");
}

#[test]
fn saved_mode_rows_cannot_transfer_native_participant_or_seat_authority() {
    let (app, first, _, f) = app();
    let handle = app.windows[&first].native.handle().unwrap();
    let saved = app.snapshot().unwrap();
    let mut restored = App::new(World::resume(saved.clone(), Execution::Grouped).unwrap(), None);
    assert!(restored.open_with_saved_preferences(f.participants[1], Seat::Player, Mode::Mesocosm, None, Some(handle)).is_err());
    assert_eq!(restored.world.save().unwrap(), saved, "foreign saved token refuses before native publication");
}

#[test]
fn mode_section_snapshots_preserve_opaque_game_and_absent_mode_bytes() {
    let (app, _, _, _) = app();
    let mut saved = app.snapshot().unwrap();
    let opaque = vec![0, 255, 19, 0, 128];
    saved.sections.insert("unbuilt-future-mode".into(), opaque.clone());
    let mut section: serde_json::Value = serde_json::from_slice(&saved.sections["mesocosm"]).unwrap();
    section["game"] = serde_json::json!(opaque);
    saved.sections.insert("mesocosm".into(), serde_json::to_vec(&section).unwrap());
    let snapshot = mode_sections::capture(saved.clone(), &app.windows).unwrap();
    assert_eq!(snapshot.sections["unbuilt-future-mode"], saved.sections["unbuilt-future-mode"],
        "view updates carry absent modes unread and byte exact");
    let section: serde_json::Value = serde_json::from_slice(&snapshot.sections["mesocosm"]).unwrap();
    assert_eq!(section["game"], serde_json::json!(opaque), "mode view updates carry opaque game bytes unchanged");
    assert_eq!(snapshot.native, saved.native);
    let mut future = saved.clone();
    future.sections.insert("mesocosm".into(), b"{\"version\":2,\"windows\":{},\"game\":[]}".to_vec());
    assert!(mode_sections::capture(future.clone(), &app.windows).is_err());
    assert_eq!(app.world.save().unwrap().native, saved.native);
}

#[test]
fn ordinary_stream_envelopes_follow_native_attention_and_the_current_seat() {
    // Reuse the declared body fixture, adding the same inert noted process
    // as the native stream fixture. Replay its actual Join/Take commands.
    let mut f = fixture();
    let saved = f.world.save().unwrap().native;
    let mut genesis = saved.genesis.clone();
    genesis.rules.field.strength = 1_000_000;
    genesis.rules.field.decay_per_tick = 1;
    let key = "process:host-stream-fixture";
    genesis.rules.processes.insert(key.into(), isocosm::rules::Process { id: key.into(),
        causation: isocosm::rules::Causation::Choice, requires: vec![], commitments: vec![],
        effects: vec![], risk: None, target: None, period: None, priority: 0,
        need_account: None, need_below: 0, glyphs: BTreeSet::new(), invariants: BTreeSet::new(), note: true });
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    for entry in saved.entries { session.command(entry.command).unwrap(); }
    let outcome = session.command(Command::Act { actor: f.subjects[0], target: None,
        process: key.into(), cause: None }).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(receipt.accepted());
    f.world = World::new(session);
    let mut app = App::new(f.world.share(), None);
    let first = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let site = *app.windows[&first].native.projection().unwrap().visible.sites.iter().next().unwrap();
    let request = app.windows[&first].native.window_attention_request(AttentionChange::Examine(PlaceHandle(site))).unwrap();
    app.windows.get_mut(&first).unwrap().native.apply_attention(request).unwrap();
    app.step().unwrap();
    let cut = app.windows[&first].native.projection().unwrap();
    let records = app.world.read(|s| crate::stream::records(s, &cut)).unwrap().unwrap();
    let expected = app.world.read(|s| s.sim.attended_events(cut.access.participant.0,
        isocosm::directing::readings::Mode::Survival)).unwrap().unwrap();
    assert!(expected.iter().any(|e| e.id == receipt.id), "real native noted Act supplies a nonempty stream");
    assert_eq!(records.len(), expected.len());
    for (record, event) in records.iter().zip(&expected) {
        assert_eq!(serde_json::from_slice::<isocosm::schema::Event>(&record.payload).unwrap(), *event);
        assert!(cut.visible.events.contains(&event.id));
    }
    let mut stale = cut.clone(); stale.access.subject = None;
    assert!(app.world.read(|s| crate::stream::records(s, &stale)).unwrap().is_err());
}
