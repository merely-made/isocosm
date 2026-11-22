// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use cosmograph::Execution;
use cosmograph_host::{
    save::{ImportSave, WorldSave},
    world::World,
};
use mesocosm_runtime::{GameSave, Runtime, runtime::MODE};
use mesocosm_runtime::continuation::{self, LoopAction, LoopAnswer, LoopInput, LoopPhase};
use cosmograph_overlay::{ParticipantHandle, Tick};

fn answer(runtime: &mut Runtime, answer: LoopAnswer) -> Result<String, mesocosm_runtime::Refusal> {
    let reading = runtime.loop_reading().unwrap();
    let input = LoopInput {
        kind: continuation::KIND.into(),
        version: continuation::VERSION,
        participant: ParticipantHandle(runtime.interim().participant),
        tick: Tick(runtime.tick()),
        expected: reading.cut,
        action: LoopAction::Answer(answer),
    };
    runtime.submit_loop_input(&input)
}

#[test]
fn leaving_a_living_critter_keeps_it_watchable_after_reopen_and_allows_reentry() {
    verify_handoff(worlds::offered());
}

#[test]
fn the_generated_spatial_clock_and_placements_survive_handoff_and_reopen() {
    let mut founded = mesocosm_runtime::Founded::generated(16_003_851_315_348_448_882, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = cosmograph::ecology::Rates::A1;
    let bodies = domain.bodies.as_mut().unwrap();
    bodies.hazard = [0; 2];
    bodies.wound = [0; 2];
    founded.genesis = domain.generate().unwrap();
    for group in founded.genesis.population.groups.values_mut() {
        group.count = 1;
    }
    for site in founded.genesis.sites.values_mut() {
        site.conditions.extend([
            ("terrain:elevation".into(), 64),
            ("terrain:relief".into(), 0),
            ("terrain:water".into(), 0),
        ]);
    }
    cosmograph::spatial::embodiment::declare(&mut founded.genesis).unwrap();
    let genesis = &founded.genesis;
    let templates = genesis.rules.spatial_templates.as_ref().unwrap();
    let (actor, neighbour) = templates.stands.values().find_map(|stands| {
        let mut actors = stands.iter().filter_map(|stand| stand.actor);
        Some((actors.next()?, actors.next()?))
    }).expect("two placed playable bodies at a site");
    for id in [actor, neighbour] {
        founded.genesis.population.groups.get_mut(&id).unwrap().entity_mut().method =
            cosmograph::schema::Method::Deliberative;
    }
    let runtime = Runtime::bench_placed(founded, worlds::RATE).unwrap();
    let participant = runtime.interim().participant;
    runtime.world().command(cosmograph::Command::Take { participant, critter: actor }).unwrap();
    // Refresh the fixture's saved continuation over the selected native binding.
    let saved = runtime.save_game().into_world_save().unwrap();
    let runtime = Runtime::resume_world(saved, worlds::RATE, None).unwrap();
    assert!(runtime.spatial().members.contains_key(&runtime.critter().unwrap()));
    verify_handoff(runtime);
}

fn verify_handoff(mut original: Runtime) {
    let critter = original.critter().unwrap();
    let participant = original.interim().participant;
    let bond = original.sim().bond(participant, critter);
    let tick = original.tick();
    answer(&mut original, LoopAnswer::Watch).unwrap();
    assert_eq!(original.tick(), tick);
    assert_eq!(original.critter(), None);
    assert_eq!(original.watched_critter(), Some(critter));
    assert_eq!(original.sim().bond(participant, critter), bond);
    assert!(original.sim().state().population.get(critter).unwrap().alive);
    assert!(original.view().shows(critter));

    let bytes = original.save_world().unwrap().encode().unwrap();
    let saved = Runtime::decode_world_save(&bytes).unwrap();
    let mut reopened = Runtime::resume_world(saved, worlds::RATE, None).unwrap();
    assert_eq!(reopened.watched_critter(), Some(critter));
    assert_eq!(reopened.loop_reading().unwrap().phase, LoopPhase::Watching);
    assert_eq!(reopened.step(3), 3);
    assert_eq!(original.step(3), 3);
    assert_eq!(reopened.tick(), tick + 3);
    assert_eq!(reopened.save_world().unwrap(), original.save_world().unwrap());
    assert_eq!(reopened.sim().plays(participant), None);
    assert!(reopened.view().shows(critter));

    let choices = reopened.loop_reading().unwrap().lineages;
    assert!(choices.iter().any(|id| id.0 == critter));
    let next = choices.into_iter().find(|id| id.0 != critter).expect("another living critter");
    answer(&mut reopened, LoopAnswer::OtherLineage(next)).unwrap();
    assert_eq!(reopened.critter(), Some(next.0));
    assert_eq!(reopened.watched_critter(), None);
    assert_eq!(reopened.loop_reading().unwrap().phase, LoopPhase::Running);
    assert_eq!(reopened.sim().bond(participant, critter), bond);
}

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

#[test]
fn enter_edited_founding_joins_as_watcher_at_the_selected_tick() {
    use cosmograph::{Command, Execution, Founding, Session};
    use mesocosm_runtime::continuation::LoopPhase;
    let founding = Founding { seed: 9, population: 42, sites: 1, ecology: true,
        played: Some(cosmograph::directing::Played { lineage: 1, region_sites: 1 }),
        ..Default::default() };
    let mut preview = Session::new(founding.generate().unwrap(), Execution::Grouped).unwrap();
    preview.advance(17).unwrap();
    cosmograph::bench::inspect_first_actor(&mut preview).unwrap();
    let input = serde_json::to_vec(&preview.save()).unwrap();
    let unjoined = Runtime::decode_world_save(&input).unwrap();
    assert!(Runtime::resume_world(unjoined, 6, None).is_err());
    let game = Runtime::enter_founding(&input, 6, None).unwrap();
    preview.command(Command::Join).unwrap();
    assert_eq!(game.session().save(), preview.save());
    assert_eq!(game.loop_reading().unwrap().phase, LoopPhase::Watching);
    let saved = game.save_world().unwrap();
    let mut reopened = Runtime::resume_world(saved, 6, None).unwrap();
    assert_eq!(reopened.current_loop_phase(), Some(LoopPhase::Watching));
    assert_eq!(reopened.session().save(), preview.save());
    reopened.step(1);
    assert_eq!(reopened.tick(), 18);
    assert_eq!(reopened.loop_reading().unwrap().phase, LoopPhase::Watching);
    answer(&mut reopened, LoopAnswer::NewWorld).unwrap();
    answer(&mut reopened, LoopAnswer::Watch).unwrap();
    let first = reopened.loop_reading().unwrap().lineages[0];
    answer(&mut reopened, LoopAnswer::OtherLineage(first)).unwrap();
    assert_eq!(reopened.critter(), Some(first.0));
    assert!(reopened.spatial().continuation.as_ref().unwrap().founding_site.is_none());
    assert!(Runtime::enter_founding(&serde_json::to_vec(&preview.save()).unwrap(), 6, None).is_err());
}
