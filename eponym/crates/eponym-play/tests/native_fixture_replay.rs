// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::*;
use eponym_play::identity::{BodyRevisionId, SubjectId, Tick};
use eponym_play::founding::{ENERGY, FULL, RESERVE, SOPHONT, TISSUE};
use eponym_play::timed_action::{Direction, TimedActionRules, TimedActionSession};
use isocosm::{Command, Execution, anatomy};
use isocosm::lineage::SpeciesId;
use isometer_core::{PartId, snapshot};
use wing_functions::{Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules};

const ACTOR: SubjectId = SubjectId(31);
const TARGET: SubjectId = SubjectId(32);

fn fixture() -> World {
    World::generate_native_fixture(7, WorldConfig { side: 2, extent: 8 },
        NativeFixtureProfile::PreciseCombatV1).unwrap()
}
fn start(world: &World) -> [i32; 3] { [0, world.ground().surface(0, 0).unwrap() + 1, 0] }
fn generate(game: &mut GameState, subject: SubjectId, recipe: NativeFixtureRecipe, at: [i32; 3]) {
    game.apply(GameIntent::GenerateNativeRecipe {
        tick: game.next_tick(), subject, body_seed: 1, at, recipe,
    }).unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(), subject, name: Name::new("Body").unwrap(),
    }).unwrap();
    let document = game.native_anatomy(subject, SpeciesId(1)).unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(), subject, revision: BodyRevisionId(0), document: Box::new(document),
    }).unwrap();
}
fn game() -> GameState {
    let world = fixture(); let at = start(&world); let mut game = GameState::new(world);
    generate(&mut game, ACTOR, NativeFixtureRecipe::ProbeActorV1, at);
    generate(&mut game, TARGET, NativeFixtureRecipe::ProbeTargetV1, at);
    game
}
fn entity(game: &GameState, subject: SubjectId) -> &isocosm::schema::Entity {
    game.world().sim().state().population.get(game.entity(subject).unwrap()).unwrap()
}

#[test]
fn native_fixture_profile_regrows_the_declared_native_pack_and_preserves_base_hash() {
    let world = fixture(); let save = world.save_record();
    assert_eq!(save.generator_version, 5);
    assert_eq!(world.sim().state().movement, isocosm::schema::Movement::Journey);
    let restored = World::restore(&world.save().unwrap())
        .expect("fixture profile must regenerate its authenticated founding");
    assert_eq!(restored.save_record(), save);
    assert_eq!(restored.state_hash().unwrap(), world.state_hash().unwrap());
    assert_eq!(isocosm::digest(restored.sim().genesis()), isocosm::digest(world.sim().genesis()));
    let ordinary = World::generate(7, world.config()).unwrap();
    assert_eq!(ordinary.generator_version(), 4);
    assert_ne!(ordinary.base_hash(), world.base_hash());
}

#[test]
fn mislabeled_or_modified_fixture_founding_is_refused_before_game_intents() {
    let world = fixture(); let mut wrong = world.save_record(); wrong.generator_version = 4;
    assert!(matches!(World::restore_record(wrong), Err(WorldError::BaseDiverged { .. })));
    let mut corrupt = world.save_record(); corrupt.base_hash ^= 1;
    assert!(matches!(World::restore_record(corrupt), Err(WorldError::BaseDiverged { .. })));
    let mut unknown = world.save_record(); unknown.generator_version = 6;
    assert!(matches!(World::restore_record(unknown), Err(WorldError::GeneratorDiverged { saved: 6, .. })));
}

#[test]
fn recipe_choice_is_logged_before_develop_and_survives_game_replay() {
    let world = fixture(); let at = start(&world); let mut game = GameState::new(world);
    generate(&mut game, ACTOR, NativeFixtureRecipe::ProbeActorV1, at);
    let actor = entity(&game, ACTOR).clone();
    generate(&mut game, TARGET, NativeFixtureRecipe::ProbeTargetV1, at);
    assert_eq!(entity(&game, ACTOR).body, actor.body, "later selection may not replace an existing body");
    assert_eq!(entity(&game, ACTOR).parts, actor.parts);
    assert_eq!(actor.body.as_ref().unwrap().part(PartId(0)).unwrap().half_extent, [1; 3],
        "actor choice must retain its native probe root");
    assert_eq!(entity(&game, TARGET).body.as_ref().unwrap().part(PartId(0)).unwrap().half_extent, [1, 3, 1],
        "target choice must retain its native probe root");
    let entries = &game.world().session().entries;
    let revisions: Vec<_> = entries.iter().enumerate().filter_map(|(i, e)|
        matches!(e.command, Command::Revise { .. }).then_some(i)).collect();
    assert_eq!(revisions.len(), 2);
    for i in revisions {
        assert!(matches!(entries[i + 1].command, Command::Arrive(_)));
        assert!(matches!(entries[i + 2].command, Command::Develop { .. }));
    }
    let save = game.save_record().unwrap();
    assert_eq!(save.version, 8);
    assert!(matches!(save.intents[0], GameIntent::GenerateNativeRecipe { recipe: NativeFixtureRecipe::ProbeActorV1, .. }));
    assert!(matches!(save.intents[3], GameIntent::GenerateNativeRecipe { recipe: NativeFixtureRecipe::ProbeTargetV1, .. }));
    let restored = GameState::restore(&game.save().unwrap()).unwrap();
    assert_eq!(restored, game);
    assert_eq!(restored.save().unwrap(), game.save().unwrap());
    assert_eq!(restored.current_anatomy(ACTOR).unwrap(), game.current_anatomy(ACTOR).unwrap());
    let ordinary = SubjectId(33);
    game.apply(GameIntent::Generate { tick: game.next_tick(), subject: ordinary, body_seed: 1, at }).unwrap();
    assert_eq!(entity(&game, ordinary).living().count(), 6, "ordinary Generate keeps the compact recipe");
}

#[test]
fn unknown_recipe_or_recipe_in_an_old_world_is_refused_atomically() {
    assert!(snapshot::decode::<NativeFixtureRecipe>(&[2]).is_err());
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let at = start(&world); let mut game = GameState::new(world); let before = game.save().unwrap();
    assert_eq!(game.apply(GameIntent::GenerateNativeRecipe { tick: game.next_tick(),
        subject: ACTOR, body_seed: 1, at, recipe: NativeFixtureRecipe::ProbeActorV1 }),
        Err(GameError::World(WorldError::NotFixture)));
    assert_eq!(game.save().unwrap(), before);
    assert!(game.entity(ACTOR).is_none());
    let mut native = fixture().session().clone(); let before = native.save();
    let mut d = native.sim.state().lineages[SOPHONT].development.clone().unwrap();
    d.recipe.tagmata[0].segment = "kind:unknown-fixture".into();
    assert!(native.command(Command::Revise { lineage: SOPHONT.into(), development: d }).is_err());
    assert_eq!(native.save(), before);
}

#[test]
fn old_game_versions_cannot_carry_the_new_recipe_intent() {
    let game = game(); let mut save = game.save_record().unwrap(); save.version = 7;
    assert_eq!(GameState::restore_record(save), Err(GameError::LegacyNativeRecipeIntent),
        "version7 cannot carry the fixture recipe vocabulary");
    let session = Session::begin(game, ACTOR).unwrap(); let mut save = session.save_record().unwrap();
    save.game.version = 7;
    assert_eq!(Session::restore_record(save), Err(SessionError::Game(GameError::LegacyNativeRecipeIntent)),
        "session version7 cannot carry the fixture recipe vocabulary");
}

#[test]
fn published_canon_v7_keeps_its_existing_vocabulary_guard_after_v8() {
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let at = start(&world); let mut game = GameState::new(world);
    game.apply(GameIntent::Generate { tick: game.next_tick(), subject: ACTOR, body_seed: 1, at }).unwrap();
    game.apply(GameIntent::Name { tick: game.next_tick(), subject: ACTOR,
        name: Name::new("Canon witness").unwrap() }).unwrap();
    let mut session = Session::begin(game, ACTOR).unwrap();
    session.apply_game(GameIntent::ReviseCanon { tick: session.game().next_tick(), revision: 2,
        seed: 99, cause: CanonRevisionCause::Authored { label: "kept v7 canon".into() } }).unwrap();
    let mut record = session.save_record().unwrap(); record.game.version = 7;
    assert_eq!(Session::restore_record(record).unwrap(), session);
    let mut game = GameState::new(World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap());
    game.apply(GameIntent::ReviseCanon { tick: game.next_tick(), revision: 2, seed: 99,
        cause: CanonRevisionCause::Authored { label: "kept v7 canon".into() } }).unwrap();
    let mut save = game.save_record().unwrap(); save.version = 7;
    assert_eq!(GameState::restore_record(save).unwrap(), game);
}

#[test]
fn native_selected_stocks_fit_and_are_conserved_without_loose_fallback() {
    let game = game(); let sim = game.world().sim(); let rules = &sim.genesis().rules;
    for subject in [ACTOR, TARGET] {
        let e = entity(&game, subject); let books = anatomy::books(e);
        assert_eq!(anatomy::held(e, rules, RESERVE), FULL);
        assert_eq!(e.accounts[ENERGY], FULL);
        assert!(!e.accounts.contains_key(RESERVE), "fixture reserve must live in declared native stores");
        assert!(!e.accounts.contains_key(TISSUE));
        let capacity: u64 = e.parts.iter().map(|(id, part)| {
            let geo = e.body.as_ref().unwrap().part(*id).unwrap();
            anatomy::bound(geo.half_extent, part, rules, TISSUE)
        }).sum();
        assert_eq!(anatomy::held(e, rules, TISSUE), capacity);
        let arrival = game.world().session().entries.iter().find_map(|entry| match &entry.command {
            Command::Arrive(a) if a.accounts.get(TISSUE) == Some(&capacity) && a.lineage == SOPHONT => Some(a),
            _ => None,
        }).unwrap();
        for key in [TISSUE, RESERVE, ENERGY] { assert_eq!(books.get(key), arrival.accounts.get(key)); }
    }
    let native = game.world().session();
    let json = serde_json::to_vec(&native.save()).unwrap();
    let restored = isocosm::Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(restored.sim.state_hash(), native.sim.state_hash());
    assert_eq!(restored.entries, native.entries);
}

#[test]
fn overflow_development_refuses_without_losing_supplied_stock_or_history() {
    let game = game(); let mut native = game.world().session().clone();
    let seed = 1; let body = native.sim.development_body(SOPHONT, seed).unwrap();
    let bound = |key: &str| body.parts.iter().map(|(id, p)|
        anatomy::bound(body.doc.part(*id).unwrap().half_extent, p, &native.sim.genesis().rules, key)).sum::<u64>();
    let tissue = bound(TISSUE); let reserve = bound(RESERVE).checked_add(1).unwrap();
    let at = game.world().site();
    let outcome = native.command(Command::Arrive(isocosm::arrival::Arrival {
        lineage: SOPHONT.into(), site: at, method: isocosm::schema::Method::Normative,
        accounts: [(TISSUE.into(), tissue), (RESERVE.into(), reserve), (ENERGY.into(), FULL)].into(),
        traits: Default::default(), skills: Default::default(), disposition: [0; 5], character: None,
    })).unwrap();
    let entity = outcome.strip_prefix("entity:").unwrap().parse().unwrap();
    let before = native.save(); let hash = native.sim.state_hash();
    assert!(native.command(Command::Develop { entity, seed }).is_err());
    assert_eq!(native.save(), before);
    assert_eq!(native.sim.state_hash(), hash);
    let e = native.sim.state().population.get(entity).unwrap();
    assert!(e.body.is_none()); assert_eq!(e.accounts[RESERVE], reserve);
}

fn timed(game: GameState) -> TimedActionSession {
    let session = Session::begin(game, ACTOR).unwrap();
    let part = PartRef { subject: ACTOR.0, part: 0 };
    let network = FunctionalNetwork::new(vec![
        Node { id: NodeId(1), kind: NodeKind::Source { part, capacity: 12, charge: 12 } },
        Node { id: NodeId(2), kind: NodeKind::Effect { part } },
    ], vec![Edge { from: NodeId(1), to: NodeId(2), capacity: 12 }]).unwrap();
    let rules = TimedActionRules { max_contributors: 2, max_elapsed_ticks: 10,
        charge_per_tick: 1, max_charge_per_limb: 12, evaluation: WorldRules {
            allowed_operators: [Operator::Strengthen].into(),
            allowed_costs: [1, 2, 3, 5, 8, 12].into(),
            max_range: 0, max_hops: 4,
        } };
    let mut timed = TimedActionSession::begin(session, network, rules).unwrap();
    timed.prepare(Direction::Right).unwrap();
    let now = timed.action().unwrap().last_tick;
    timed.charge(Tick(now.0 + 1)).unwrap();
    timed
}

#[test]
fn selected_native_snapshot_and_source_survive_session_and_timed_action_replay() {
    let game = game(); let session = Session::begin(game.clone(), ACTOR).unwrap();
    let restored = Session::restore(&session.save().unwrap()).unwrap();
    assert_eq!(restored, session);
    assert_eq!(restored.game().current_anatomy(ACTOR).unwrap(), session.game().current_anatomy(ACTOR).unwrap());
    let mut straight = timed(game); let mut resumed = TimedActionSession::restore(&straight.save().unwrap()).unwrap();
    assert_eq!(resumed, straight);
    assert!(resumed.action().is_some());
    let tick = Tick(straight.action().unwrap().last_tick.0 + 1);
    assert_eq!(straight.charge(tick).unwrap(), resumed.charge(tick).unwrap());
    assert_eq!(straight.save().unwrap(), resumed.save().unwrap());
}

#[test]
fn game_session_and_timed_action_expected_hashes_remain_authoritative() {
    let game = game(); let mut save = game.save_record().unwrap(); save.expected_hash ^= 1;
    assert!(matches!(GameState::restore_record(save), Err(GameError::StateDiverged { .. })));
    let session = Session::begin(game.clone(), ACTOR).unwrap(); let mut save = session.save_record().unwrap();
    save.expected_hash ^= 1;
    assert!(matches!(Session::restore_record(save), Err(SessionError::StateDiverged { .. })));
    let mut save = timed(game).save_record().unwrap(); save.session.game.expected_hash ^= 1;
    assert!(matches!(TimedActionSession::restore_record(save),
        Err(eponym_play::timed_action::TimedActionError::Session(SessionError::Game(GameError::StateDiverged { .. })))));
}
