// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::*;
use eponym_play::identity::{BodyRevisionId, SubjectId};
use eponym_play::founding::{ENERGY, RESERVE, SOPHONT, TISSUE};
use isocosm::{Command, anatomy};
use isocosm::lineage::SpeciesId;
use isometer_core::PartId;

const SUBJECT: SubjectId = SubjectId(818);

fn game() -> GameState {
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let top = world.ground().surface(0,0).unwrap();
    let at = [0, top+1, 0];
    let mut game = GameState::new(world);
    game.apply(GameIntent::Generate { tick: game.next_tick(), subject: SUBJECT, body_seed: 9, at }).unwrap();
    game.apply(GameIntent::Name { tick: game.next_tick(), subject: SUBJECT, name: Name::new("Native body").unwrap() }).unwrap();
    game
}

fn admit(game: &mut GameState) {
    let document = game.native_anatomy(SUBJECT, SpeciesId(71)).unwrap();
    game.apply(GameIntent::AdmitAnatomy { tick: game.next_tick(), subject: SUBJECT,
        revision: BodyRevisionId(0), document: Box::new(document) }).unwrap();
}

#[test]
fn selected_native_recipe_measures_six_parts_860_tissue_105_reserve_capacity() {
    let game = game();
    let sim = game.world().sim();
    let e = sim.state().population.get(game.entity(SUBJECT).unwrap()).unwrap();
    let rules = &sim.genesis().rules;
    assert_eq!(e.living().count(), 6);
    assert_eq!(anatomy::held(e, rules, TISSUE), 860);
    assert_eq!(anatomy::held(e, rules, RESERVE), 100);
    assert_eq!(anatomy::room(e, rules, RESERVE) + anatomy::held(e, rules, RESERVE), 105);
    assert_eq!(e.accounts[ENERGY], 100);
    assert!(!e.accounts.contains_key(TISSUE));
    assert!(!e.accounts.contains_key(RESERVE));
    assert_eq!(e.parts[&PartId(0)].cells["function:store"], 5);
    for id in [1,2,3,4] {
        let p = &e.parts[&PartId(id)];
        for f in ["function:contract", "function:support", "function:grip"] { assert_eq!(p.cells[f],1); }
    }
    assert_eq!(e.parts[&PartId(5)].cells["function:sense"],1);
    assert_eq!(game.bodies().get(SUBJECT).unwrap().profile.mass_mg,860);
    assert!(game.world().session().entries.iter().any(|e| matches!(e.command, Command::Develop {..})));
}

#[test]
fn admitted_snapshot_matches_native_geometry_mass_and_source_without_a_body_write() {
    let mut game = game();
    let id = game.entity(SUBJECT).unwrap();
    let before = game.world().sim().state().population.get(id).unwrap().clone();
    let entries = game.world().session().entries.len();
    let tick = game.world().sim().state().tick;
    let native_hash = game.world().sim().state_hash();
    admit(&mut game);
    let e = game.world().sim().state().population.get(id).unwrap();
    let record = game.current_anatomy(SUBJECT).unwrap();
    assert_eq!(record.document.doc(), e.body.as_ref().unwrap());
    assert_eq!(record.document.total_mass_mg(),860);
    let source = record.native_source.as_ref().expect("native admission must retain its source");
    assert_eq!(source.entity,id);
    assert_eq!(source.lineage,SOPHONT);
    assert_eq!(source.body_revision,e.body_revision);
    assert_eq!(source.tick,tick);
    assert_eq!(source.state_hash,native_hash);
    assert_eq!(record.revision,BodyRevisionId(0));
    assert_eq!(e,&before);
    assert_eq!(game.world().session().entries.len(),entries, "snapshot admission may not log Embody");
}

#[test]
fn a_different_authored_geometry_cannot_replace_the_native_body() {
    let mut game = game();
    let hash = game.state_hash().unwrap();
    let saved = game.save().unwrap();
    let wrong = eponym_play::fixtures::three_lives::wetland_body();
    assert_eq!(game.apply(GameIntent::AdmitAnatomy { tick: game.next_tick(), subject: SUBJECT,
        revision: BodyRevisionId(0), document: Box::new(wrong) }), Err(GameError::Anatomy(AnatomyError::NativeMismatch)));
    assert_eq!(game.state_hash().unwrap(),hash);
    assert_eq!(game.save().unwrap(),saved);
    assert!(game.anatomies().get(SUBJECT).is_none());
}

#[test]
fn first_hunger_and_fatigue_spend_from_the_native_owners_and_replay() {
    let mut game = game();
    admit(&mut game);
    game.apply(GameIntent::Observe { tick: game.next_tick(), subject: SUBJECT, target: game.movement().position(SUBJECT).unwrap() }).unwrap();
    let sim = game.world().sim();
    let e = sim.state().population.get(game.entity(SUBJECT).unwrap()).unwrap();
    assert_eq!(anatomy::held(e,&sim.genesis().rules,RESERVE),99);
    assert_eq!(e.accounts[ENERGY],99);
    let restored = GameState::restore(&game.save().unwrap()).unwrap();
    assert_eq!(restored,game);
    assert_eq!(restored.current_anatomy(SUBJECT).unwrap(),game.current_anatomy(SUBJECT).unwrap());
}
