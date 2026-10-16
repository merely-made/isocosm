// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use isocosm::Execution;
use isocosm_host::{
    save::{ImportSave, WorldSave},
    world::World,
};
use mesocosm_runtime::{GameSave, Runtime, runtime::MODE};

#[test]
fn legacy_game_import_loads_in_the_core_and_resumes_the_same_play() {
    let mut original = worlds::offered();
    assert_eq!(original.step(3), 3);
    original.queue(worlds::envelope(
        &original,
        worlds::nudge(original.critter().unwrap()),
    ));
    let legacy = original.save_game();
    let core = World::import(legacy.clone(), Execution::Grouped).unwrap();
    assert_eq!(core.cut().unwrap().state_hash, original.state_hash());
    let mut saved = core.save().unwrap();
    let unknown = vec![0, 255, 128, b'{', 0, b'}'];
    saved
        .sections
        .insert("absent-product".into(), unknown.clone());
    let saved = WorldSave::decode(&saved.encode().unwrap()).unwrap();
    let mut resumed = Runtime::resume_world(saved.clone(), worlds::RATE, None).unwrap();
    assert_eq!(resumed.state_hash(), original.state_hash());
    assert_eq!(resumed.queued_len(), original.queued_len());
    assert_eq!(resumed.save_world().unwrap(), saved);
    for _ in 0..3 {
        assert_eq!(resumed.step(1), original.step(1));
        assert_eq!(resumed.state_hash(), original.state_hash());
        assert_eq!(resumed.save_game().spatial, original.save_game().spatial);
    }
    assert_eq!(
        resumed.save_world().unwrap().sections["absent-product"],
        unknown
    );
}

#[test]
fn held_question_survives_core_save_and_resume() {
    let mut original = worlds::held();
    assert_eq!(original.step(1), 1);
    assert!(original.checkpoint().is_some());
    let saved = original.save_world().unwrap();
    let mut resumed = Runtime::resume_world(saved.clone(), worlds::RATE, None).unwrap();
    assert_eq!(resumed.save_world().unwrap(), saved);
    assert_eq!(resumed.step(1), 0);
    let parent = original.critter().unwrap();
    original.queue(worlds::envelope(&original, worlds::keep_parent(parent)));
    resumed.queue(worlds::envelope(&resumed, worlds::keep_parent(parent)));
    assert_eq!(resumed.step(1), original.step(1));
    assert_eq!(
        resumed.save_world().unwrap(),
        original.save_world().unwrap()
    );
}

#[test]
fn old_grammars_import_and_invalid_envelopes_refuse() {
    let original = worlds::offered();
    let legacy = original.save_game();
    let expected = legacy.clone().into_world_save().unwrap();
    assert_eq!(
        Runtime::decode_world_save(&serde_json::to_vec(&legacy).unwrap()).unwrap(),
        expected
    );
    assert_eq!(
        Runtime::decode_world_save(&expected.encode().unwrap()).unwrap(),
        expected
    );
    let native = original.save();
    let imported = Runtime::decode_world_save(&serde_json::to_vec(&native).unwrap()).unwrap();
    assert_eq!(imported.native, native);

    let mut broken = expected.clone();
    broken.format += 1;
    assert!(Runtime::decode_world_save(&broken.encode().unwrap()).is_err());
    assert!(Runtime::resume_world(broken, worlds::RATE, None).is_err());
    let mut broken = expected.clone();
    broken.sections.remove(MODE);
    assert!(Runtime::decode_world_save(&broken.encode().unwrap()).is_err());
    let mut broken = expected.clone();
    broken.sections.insert(MODE.into(), vec![255]);
    assert!(Runtime::resume_world(broken, worlds::RATE, None).is_err());
    let mut broken = expected;
    broken.native.state_hash ^= 1;
    assert!(Runtime::resume_world(broken, worlds::RATE, None).is_err());
    let mut broken = legacy;
    broken.version = 999;
    assert!(broken.into_world_save().is_err());
}

#[test]
fn idle_publish_keeps_the_world_revision() {
    let mut runtime = worlds::held();
    runtime.step(1);
    let revision = runtime.world().revision().unwrap();
    assert_eq!(runtime.step(1), 0);
    assert_eq!(runtime.world().revision().unwrap(), revision);
}

#[test]
fn another_sections_publication_does_not_lose_pending_input() {
    let mut runtime = worlds::offered();
    runtime
        .world()
        .put_section("absent-product", vec![255, 0])
        .unwrap();
    runtime.queue(worlds::envelope(
        &runtime,
        worlds::nudge(runtime.critter().unwrap()),
    ));
    let mut resumed =
        Runtime::resume_world(runtime.save_world().unwrap(), worlds::RATE, None).unwrap();
    assert_eq!(resumed.queued_len(), 1);
    assert_eq!(resumed.step(1), runtime.step(1));
    assert_eq!(resumed.save_world().unwrap(), runtime.save_world().unwrap());
}

#[test]
fn saving_after_an_endpoint_turn_keeps_its_newer_section() {
    let runtime = worlds::offered();
    let world = runtime.world();
    // The endpoint's section contains the whole saved loop. Changing it
    // models another door publishing between this driver's calls.
    let mut game = runtime.save_game().spatial;
    game.continuation.as_mut().unwrap().fault = Some("endpoint refusal".into());
    let bytes = serde_json::to_vec(&game).unwrap();
    world.put_section(MODE, bytes.clone()).unwrap();
    let saved = runtime.save_world().unwrap();
    assert_eq!(saved.sections[MODE], bytes);
    assert_eq!(GameSave::from_world(&saved).unwrap().spatial, game);
    let mut resumed = Runtime::resume_world(saved, worlds::RATE, None).unwrap();
    assert_eq!(resumed.fault(), Some("endpoint refusal"));
    assert_eq!(resumed.step(1), 0);
}
