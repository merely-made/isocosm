// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "../../mesocosm-runtime/tests/runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use cosmograph::Execution;
use cosmograph_host::{save::WorldSave, world::World};
use mesocosm_genet::played;
use mesocosm_runtime::Runtime;

#[test]
fn the_heads_writer_loads_in_the_core_and_watch_carries_on() {
    let mut runtime = worlds::held();
    runtime.step(1);
    let mut saved = runtime.save_world().unwrap();
    let unknown = vec![255, 0, b'{', 127];
    saved
        .sections
        .insert("absent-product".into(), unknown.clone());
    let mut runtime = Runtime::resume_world(saved, worlds::RATE, None).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("head.json");
    played::write_world_trace(&path, &runtime).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let core = World::resume(WorldSave::decode(&bytes).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(core.cut().unwrap().state_hash, runtime.state_hash());
    assert_eq!(core.save().unwrap().sections["absent-product"], unknown);
    assert_eq!(
        played::replay(&path).unwrap(),
        (runtime.state_hash(), runtime.state_hash())
    );
    let mut watched = played::resume(&path, worlds::RATE).unwrap();
    assert_eq!(watched.save_world().unwrap(), runtime.save_world().unwrap());
    for game in [&mut watched, &mut runtime] {
        let parent = game.critter().unwrap();
        game.queue(worlds::envelope(game, worlds::keep_parent(parent)));
        game.step(2);
    }
    assert_eq!(watched.save_world().unwrap(), runtime.save_world().unwrap());
    // Replacing an existing save uses the same atomic writer as the window.
    played::write_world_trace(&path, &watched).unwrap();
    assert_eq!(
        played::resume(&path, worlds::RATE)
            .unwrap()
            .save_world()
            .unwrap(),
        watched.save_world().unwrap()
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}
