// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isocosm::schema::Movement;

#[test]
fn world_save_wire_pins_its_movement_semantics() {
    for version in [2,3,4] {
        // This is a current explicit-mode control, not a manufactured old save.
        let movement = if version==2 { Movement::ImmediateV3 } else { Movement::Journey };
        let w = World::generate_version(7, WorldConfig { side:2, extent:8 }, version).unwrap();
        let record = w.save_record();
        assert_eq!(record.generator_version, version);
        assert_eq!(w.developed_bodies(), version==4);
        let bytes = w.save().unwrap();
        let loaded = World::restore(&bytes).unwrap();
        assert_eq!(loaded.session.sim.state().movement,movement);
        assert_eq!(loaded.base_hash,w.base_hash);
        assert_eq!(loaded.session.sim.state_hash(),w.session.sim.state_hash());
        assert_eq!(loaded.save().unwrap(),bytes);
        let mut wrong = record; wrong.generator_version = 5;
        assert!(matches!(World::restore_record(wrong),Err(WorldError::GeneratorDiverged {..})));
    }
}

#[test]
#[ignore = "requires immutable genuine Eponym WorldSave2 bytes and SHA256 from root"]
fn authentic_world_save2_keeps_native_movement_and_binary_bytes() {
    let path = std::env::var("EPONYM_JOURNEY_V2_ARCHIVE").unwrap();
    // Outer custody authenticates bytes/SHA before this native decode gate.
    let bytes = std::fs::read(path).unwrap();
    let old: WorldSave = snapshot::decode(&bytes).unwrap();
    assert_eq!(old.generator_version,2);
    let loaded = World::restore(&bytes).unwrap();
    assert_eq!(loaded.session.sim.state().movement,Movement::ImmediateV3);
    assert_eq!(loaded.base_hash,old.base_hash);
    assert_eq!(loaded.save().unwrap(),bytes);
}
