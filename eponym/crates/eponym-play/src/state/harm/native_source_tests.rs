// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{GameIntent, Name, World, WorldConfig};
use isocosm::lineage::SpeciesId;

#[test]
fn two_native_severings_keep_one_game_revision_and_the_actual_native_snapshot_source() {
    let subject = SubjectId(819);
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let top = world.ground().surface(0,0).unwrap();
    let mut game = GameState::new(world);
    game.apply(GameIntent::Generate { tick: game.next_tick(), subject, body_seed: 9,
        at: [0,top+1,0] }).unwrap();
    game.apply(GameIntent::Name { tick: game.next_tick(), subject, name: Name::new("Two leaves").unwrap() }).unwrap();
    let doc = game.native_anatomy(subject, SpeciesId(71)).unwrap();
    game.apply(GameIntent::AdmitAnatomy { tick: game.next_tick(), subject,
        revision: BodyRevisionId(0), document: Box::new(doc) }).unwrap();
    let before = game.current_anatomy(subject).unwrap().native_source.as_ref().unwrap().body_revision;
    let handoff = EponymHandoff::Hurt(Harm { vigour_drained: 0,
        wounds: [1,4].map(|part| Wound { part: PartHandle(part), severity: WoundSeverity::Severed }).to_vec() });
    let (revision, died, _) = game.receive_harm(game.next_tick(), subject, handoff).unwrap();
    assert!(!died);
    assert_eq!(revision,BodyRevisionId(1),"one game revision for the admitted severing batch");
    let id = game.entity(subject).unwrap();
    let e = game.world.sim().state().population.get(id).unwrap();
    let record = game.anatomies.get(subject).unwrap();
    assert_eq!(e.body_revision,before+2,"native commands each moved the native body revision");
    let source = record.native_source.as_ref().unwrap();
    assert_eq!(source.body_revision,e.body_revision,"snapshot source must name the native result");
    assert_eq!(source.state_hash,game.world.sim().state_hash());
    assert_eq!(record.document.doc(),e.body.as_ref().unwrap());
    assert_eq!(record.document,game.native_anatomy(subject,SpeciesId(71)).unwrap());
    for part in [1,4] { assert!(record.document.part(PartId(part)).unwrap().severed); }
}
