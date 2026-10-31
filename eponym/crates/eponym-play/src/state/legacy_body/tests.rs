// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{AnatomyRecord, Name, World, WorldConfig, WorldError, WorldIntent, WorldSave};
use cosmograph::lineage::SpeciesId;
use isometer_core::{VolumeRef, snapshot};
use serde::Serialize;

const SUBJECT: SubjectId = SubjectId(824);

fn legacy_record() -> WorldSave {
    let seed = 7;
    let config = WorldConfig { side: 2, extent: 8 };
    let genesis = crate::founding::legacy_genesis_v2(seed, u32::from(config.side), config.extent as u64*2).unwrap();
    WorldSave { generator_version: 2, seed, config,
        base_hash: snapshot::hash_bytes(cosmograph::digest(&genesis).as_bytes()), intents: vec![] }
}

fn generated() -> GameState {
    let mut game = GameState::new(World::restore_record(legacy_record()).unwrap());
    let top = game.world().ground().surface(0,0).unwrap();
    let result = game.apply(crate::GameIntent::Generate { tick: game.next_tick(), subject: SUBJECT,
        body_seed: 9, at: [0,top+1,0] });
    assert!(result.is_ok(),"legacy Generate must use the recorded body grammar: {result:?}");
    game.apply(crate::GameIntent::Name { tick: game.next_tick(), subject: SUBJECT,
        name: Name::new("Old native body").unwrap() }).unwrap();
    game
}

#[test]
fn world_v2_keeps_its_old_foundation_and_version_without_recipe_migration() {
    let save = legacy_record();
    let world = World::restore_record(save.clone()).unwrap();
    assert_eq!(world.save_record(),save);
    assert!(!world.developed_bodies());
    assert!(world.sim().state().lineages[founding::SOPHONT].development.is_none());
    let current = World::generate(save.seed,save.config).unwrap();
    assert!(current.developed_bodies());
    assert_eq!(current.save_record().generator_version,4);
    assert_ne!(world.base_hash(),current.base_hash());
}

#[test]
fn mislabeled_or_tampered_legacy_foundation_is_refused_before_intents() {
    let mut save = legacy_record();
    let current = World::generate(save.seed,save.config).unwrap();
    save.base_hash = current.base_hash();
    assert!(matches!(World::restore_record(save),Err(WorldError::BaseDiverged {..})));
    let mut tampered = legacy_record();
    tampered.base_hash ^= 1;
    tampered.intents.push(WorldIntent::Carve { tick: Tick(0), by: SUBJECT, centre: [0;3], radius: -1 });
    assert!(matches!(World::restore_record(tampered),Err(WorldError::BaseDiverged {..})));
}

#[test]
fn legacy_generate_admit_and_replay_keep_the_former_recorded_effects() {
    let mut game = generated();
    let entity = game.entity(SUBJECT).unwrap();
    let e = game.world().sim().state().population.get(entity).unwrap();
    let draw = snapshot::hash_bytes(&snapshot::encode(&(7u64,9u64,SUBJECT)).unwrap());
    assert_eq!(e.accounts[founding::TISSUE],48_000 + draw%8_000);
    assert_eq!(game.bodies().get(SUBJECT).unwrap().profile, BodyProfile::draw(7,9,SUBJECT),
        "legacy body mass must retain the recorded profile draw");
    assert_eq!(e.accounts[RESERVE],100);
    assert_eq!(e.accounts[ENERGY],100);
    assert_eq!(e.body.as_ref().unwrap().parts[0].half_extent,[2,4,2]);
    assert!(e.parts.values().all(|p| p.matter.is_empty()));
    assert!(!game.world().session().entries.iter().any(|entry| matches!(entry.command,Command::Develop {..})));
    let document = LineageBody::new(SpeciesId(71),VolumeRef::from_tag(77),100,[1;3]);
    game.apply(crate::GameIntent::AdmitAnatomy { tick: game.next_tick(), subject: SUBJECT,
        revision: BodyRevisionId(0), document: Box::new(document.clone()) }).unwrap();
    let record = game.current_anatomy(SUBJECT).unwrap();
    assert_eq!(record.document,document);
    assert!(record.native_source.is_none());
    assert_eq!(game.world().sim().state().population.get(entity).unwrap().body.as_ref(),Some(document.doc()));
    let bytes = game.save().unwrap();
    let restored = GameState::restore(&bytes).unwrap();
    assert_eq!(restored.state_hash().unwrap(),game.state_hash().unwrap());
    assert_eq!(restored,game);
    assert_eq!(restored.save().unwrap(),bytes);
    assert_eq!(restored.world().save_record().generator_version,2);
}

#[derive(Serialize)]
struct FormerRecord<'a> {
    subject: SubjectId,
    revision: BodyRevisionId,
    document: &'a LineageBody,
}

#[test]
fn source_less_legacy_snapshot_retains_the_former_postcard_witness_bytes() {
    let document = LineageBody::new(SpeciesId(71),VolumeRef::from_tag(77),100,[1;3]);
    let record = AnatomyRecord { subject: SUBJECT, revision: BodyRevisionId(0),
        document, native_source: None };
    let former = FormerRecord { subject: record.subject, revision: record.revision,
        document: &record.document };
    assert_eq!(snapshot::encode(&record).unwrap(),snapshot::encode(&former).unwrap());
}
