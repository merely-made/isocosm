// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{WorldConfig, WorldError, GameError, SessionError,
    identity::{BodyRevisionId, SubjectId, Tick},
    timed_action::{TimedActionSave, TimedActionSession, TimedActionError}};
use isometer_core::{snapshot, VolumeRef};
use isocosm::lineage::{LineageBody, SpeciesId};

const ARCHIVES: [&[u8];3] = [
    include_bytes!("../../../eponym-motion/tests/fixtures/timed-action-v1-game-v3.save"),
    include_bytes!("../../../eponym-motion/tests/fixtures/timed-action-v1-game-v4.save"),
    include_bytes!("../../../eponym-motion/tests/fixtures/timed-action-v1-game-v5.save"),
];

#[test]
fn genuine_motion_archives_keep_all_old_wire_bytes() {
    for (index, bytes) in ARCHIVES.into_iter().enumerate() {
        let saved:TimedActionSave = snapshot::decode(bytes).unwrap();
        assert_eq!(saved.session.game.version, index as u32 + 3);
        assert_eq!(saved.session.game.world.generator_version, 1);
        assert_eq!(snapshot::encode(&saved).unwrap(), bytes,
            "decoded old anatomy must retain geometry, mass, provenance and intent order");
        assert!(matches!(TimedActionSession::restore_record(saved),
            Err(TimedActionError::Session(SessionError::Game(GameError::World(
                WorldError::GeneratorDiverged { saved:1, .. }))))),
            "wire decoding cannot bypass the missing generator-1 authentication");
    }
}

#[test]
fn current_game_save_keeps_its_original_four_field_codec() {
    #[derive(Serialize)]
    struct Former<'a> { version:u32, world:&'a WorldSave, expected_hash:u64, intents:&'a Vec<GameIntent> }
    let saved = GameSave { version:crate::GAME_STATE_VERSION,
        world:WorldSave { generator_version:crate::GENERATOR_VERSION, seed:17,
            config:WorldConfig::default(), base_hash:83, intents:vec![] },
        expected_hash:97, intents:vec![GameIntent::AdmitAnatomy {
            tick:Tick(0), subject:SubjectId(1), revision:BodyRevisionId(0),
            document:Box::new(LineageBody::new(SpeciesId(1), VolumeRef::from_tag(1), 23, [2;3])),
        }] };
    let former = Former { version:saved.version, world:&saved.world,
        expected_hash:saved.expected_hash, intents:&saved.intents };
    let encoded = snapshot::encode(&saved).unwrap();
    assert_eq!(encoded, snapshot::encode(&former).unwrap());
    assert_eq!(snapshot::decode::<GameSave>(&encoded).unwrap(), saved);
}

#[test]
fn old_anatomy_id_corruption_and_unknown_generator_are_refused() {
    let saved:TimedActionSave = snapshot::decode(ARCHIVES[0]).unwrap();
    let mut json = serde_json::to_value(&saved.session.game).unwrap();
    let document = json["intents"].as_array_mut().unwrap().iter_mut()
        .find_map(|i| i.get_mut("AdmitAnatomy")).unwrap();
    document["document"]["parts"][0]["id"] = 1.into();
    assert!(serde_json::from_value::<GameSave>(json).is_err(),
        "wire IDs cannot change the index-to-mass correspondence");
    let mut unknown = saved;
    unknown.session.game.world.generator_version = u32::MAX;
    assert!(matches!(TimedActionSession::restore_record(unknown),
        Err(TimedActionError::Session(SessionError::Game(GameError::World(
            WorldError::GeneratorDiverged { saved:u32::MAX, .. }))))));
}
