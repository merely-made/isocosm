// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{products::{Input, Mode}, ui::model::{App, Preferences}};
use isocosm_overlay::{EntityHandle, PlaceHandle, Pointable, Intent, Tick};

fn app() -> (App, u64, u64, Fixture) {
    let fixture = fixture();
    let mut app = App::new(fixture.world.share(), None);
    let first = app.open(fixture.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let second = app.open(fixture.participants[0], Seat::Player, Mode::Eponym, Preferences::default(), None).unwrap();
    (app, first, second, fixture)
}

#[test]
fn actual_product_mode_model_preserves_native_body_and_the_other_window() {
    let (mut app, first, second, f) = app();
    let body_before = body(&f, f.subjects[0]);
    let other = app.windows[&second].native.binding().clone();
    let handle = app.windows[&first].native.handle().unwrap();
    let cut = app.world.cut().unwrap();
    app.switch(first, Mode::Vtt).unwrap();
    assert_eq!(app.windows[&first].native.binding().mode, "vtt");
    assert_eq!(app.windows[&first].native.handle().unwrap(), handle);
    assert_eq!(app.windows[&second].native.binding(), &other, "Mode UI changes only its addressed window");
    assert_eq!(body(&f, f.subjects[0]), body_before);
    assert_eq!(app.windows[&first].native.projection().unwrap().access.subject.unwrap().0, f.subjects[0]);
    assert_eq!(app.world.cut().unwrap().entries, cut.entries + 1, "both product windows observe the same native Mode admission");
    assert_eq!(app.windows[&first].native.cut().unwrap(), app.windows[&second].native.cut().unwrap());
}

#[test]
fn actual_mode_defaults_are_local_and_camera_turns_create_no_native_watch() {
    let (mut app, first, second, _) = app();
    let before = app.world.save().unwrap();
    let other = app.windows[&second].native.binding().clone();
    app.turn(first).unwrap();
    assert_eq!(app.world.save().unwrap(), before, "camera defaults do not create a native attention entry");
    assert_eq!(app.windows[&second].native.binding(), &other);
    let rotated = app.windows[&first].native.binding().view_defaults.clone();
    app.switch(first, Mode::Vtt).unwrap();
    app.switch(first, Mode::Mesocosm).unwrap();
    assert_eq!(app.windows[&first].native.binding().view_defaults, rotated);
}

#[test]
fn actual_product_readings_share_the_seat_cut_and_new_world_changes() {
    let (mut app, first, second, f) = app();
    let hidden = f.world.read(|s| {
        let cut = app.windows[&first].native.projection().unwrap();
        s.sim.state().population.groups.keys().find(|id| !cut.visible.shows(**id)
            && !isocosm::directing::is_participant(&s.sim.state().population.get(**id).unwrap())).copied()
    }).unwrap().expect("declared three-site fixture has a hidden native group");
    let hidden_pin = Pointable::Entity(EntityHandle(hidden));
    let request = app.windows[&first].native.window_attention_request(isocosm_overlay::AttentionChange::Pin(hidden_pin)).unwrap();
    app.windows.get_mut(&first).unwrap().native.apply_attention(request).unwrap();
    app.refresh();
    let cut = app.windows[&first].native.projection().unwrap();
    assert!(!cut.visible.shows(hidden));
    assert!(!cut.attention.pinned.contains(&hidden_pin));
    let expected = app.world.read(|s| mesocosm_runtime::borrowed::Reading::new(s, &cut.visible).things()).unwrap();
    assert_eq!(app.windows[&first].rows.things.len(), expected.len(), "product rows must use the actual seat cut");
    let hash_before = app.world.cut().unwrap().state_hash;
    app.step(first).unwrap();
    assert_ne!(app.world.cut().unwrap().state_hash, hash_before);
    assert_eq!(app.windows[&first].native.cut().unwrap(), app.windows[&second].native.cut().unwrap());
    for key in [first, second] {
        let local = &app.windows[&key];
        let cut = local.native.projection().unwrap();
        let mode = Mode::parse(&local.native.binding().mode).unwrap();
        let expected = app.world.read(|s| crate::products::read(s, &cut, mode)).unwrap().unwrap();
        assert_eq!(local.rows, expected);
    }
}

#[test]
fn actual_mode_switch_close_and_restore_preserve_each_native_contribution() {
    let (mut app, first, second, f) = app();
    let first_site = *app.windows[&first].native.projection().unwrap().visible.sites.iter().next().unwrap();
    app.examine(first, Some(first_site)).unwrap();
    let site = *app.windows[&second].native.projection().unwrap().visible.sites.iter().next().unwrap();
    app.examine(second, Some(site)).unwrap();
    let handle = app.windows[&second].native.handle().unwrap();
    let saved = app.world.save().unwrap();
    let resume = World::resume(saved.clone(), Execution::Grouped).unwrap();
    let mut restored = App::new(resume, None);
    let exact = restored.open(f.participants[0], Seat::Player, Mode::Eponym, Preferences::default(), Some(handle)).unwrap();
    assert_eq!(restored.world.save().unwrap(), saved, "named product restore creates no new window history");
    assert_eq!(restored.windows[&exact].native.handle().unwrap(), handle);
    app.close(first).unwrap();
    assert!(app.windows.contains_key(&second));
    assert_eq!(app.world.player_attention(f.participants[0]).unwrap().examined,
        BTreeSet::from([PlaceHandle(site)]), "UI close withdraws only that native window contribution");
}

#[test]
fn actual_product_submission_refuses_another_mode_and_a_non_dm_table_edit_atomically() {
    use isocosm_overlay::{WorldPoint, vtt::{Assertion, Cell, NewCharacter}};
    let (mut app, first, _, _) = app();
    let assertions = vec![Assertion::Character(NewCharacter { key: "ui-mira".into(), name: "Mira".into(), faction: None,
        at: Cell { site: PlaceHandle(0), at: WorldPoint([0, 0, 0]) }, owner: None })];
    let input = Input { mode: Mode::Vtt, bytes: serde_json::to_vec(&assertions).unwrap(), round: 1, region: None };
    app.windows.get_mut(&first).unwrap().input = Some(input);
    let before = app.world.save().unwrap();
    assert!(app.send(first).is_err(), "a chosen intent cannot act through another product mode");
    assert_eq!(app.world.save().unwrap(), before);
    app.switch(first, Mode::Vtt).unwrap();
    let before = app.world.save().unwrap();
    assert!(app.send(first).is_err(), "ordinary VTT assertions require the DM seat");
    assert_eq!(app.world.save().unwrap(), before);
}

#[test]
fn actual_borrowed_mesocosm_nudge_uses_named_participant_and_retains_body() {
    use isocosm_overlay::mesocosm::{MesocosmIntent, Nudge, NudgeTarget, NudgeMeaning};
    let (mut app, first, _, f) = app();
    let site = f.world.read(|s| s.sim.state().population.get(f.subjects[0]).unwrap().place).unwrap();
    let envelope = isocosm_overlay::IntentEnvelope { participant: f.participants[0], tick: Tick(app.world.cut().unwrap().tick),
        intent: Intent::Game(MesocosmIntent::Nudge(Nudge { entity: EntityHandle(f.subjects[0]),
            target: NudgeTarget::Place(PlaceHandle(site)), meaning: NudgeMeaning::Attend })) };
    app.windows.get_mut(&first).unwrap().input = Some(Input { mode: Mode::Mesocosm,
        bytes: serde_json::to_vec(&envelope).unwrap(), round: 1, region: None });
    let body_before = body(&f, f.subjects[0]);
    let entries = app.world.cut().unwrap().entries;
    app.send(first).unwrap();
    assert_eq!(app.world.cut().unwrap().entries, entries + 1, "borrowed Mesocosm logs the real native nudge");
    assert_eq!(body(&f, f.subjects[0]), body_before);
    let before = app.world.save().unwrap();
    let mut stale = envelope; stale.participant = f.participants[1];
    app.windows.get_mut(&first).unwrap().input.as_mut().unwrap().bytes = serde_json::to_vec(&stale).unwrap();
    assert!(app.send(first).is_err(), "borrowed Mesocosm refuses a foreign envelope participant");
    assert_eq!(app.world.save().unwrap(), before);
}

#[test]
#[ignore = "serial native gate: writes the explicitly named headed fixture and actual Join identities"]
fn write_declared_real_native_mode_host_fixture() {
    let f = fixture();
    let path = std::path::PathBuf::from(std::env::var_os("ISOCOSM_HOST_GATE_SAVE")
        .expect("the gate names its owned output path"));
    let mut saved = f.world.save().unwrap();
    saved.sections.insert("absent-mode".into(), vec![0, 255, 128, b'{', b'}', 0]);
    let bytes = saved.encode().unwrap();
    let identity = serde_json::to_vec(&serde_json::json!({
        "participant": f.participants[0].0, "subject": f.subjects[0],
        "other_participant": f.participants[1].0, "other_subject": f.subjects[1],
        "native_hash": saved.native.state_hash, "native_entries": saved.native.entries.len(),
        "save_bytes_native_digest": isocosm::digest(&bytes), "declared_seed": 11
    })).unwrap();
    std::fs::write(&path, bytes).unwrap();
    std::fs::write(path.with_extension("identity.json"), identity).unwrap();
}
