// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn certain() -> ProbeWorld {
    let source = BodyFounding {
        harm: Some(HarmFounding {
            hazard: [64, 64],
            cells: [1, 1],
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut genesis = fixture(false);
    let hazard = source.genesis.rules.processes["body:hazard"].clone();
    genesis.rules.processes.insert(hazard.id.clone(), hazard);
    ProbeWorld { genesis, ticks: 1 }
}

fn refuse(world: &ProbeWorld) -> String {
    match Crowd::new(world, 57, Variant::Histogram).unwrap().run() {
        Err(why) => why,
        Ok(_) => panic!("the crowd accepted an unsupported harm pass"),
    }
}

#[test]
fn a_committed_hazard_wounds_in_both_runners() {
    let mut world = certain();
    let p = world
        .genesis
        .rules
        .processes
        .get_mut("body:hazard")
        .unwrap();
    p.commitments = std::mem::take(&mut p.effects);
    let mut native = Session::new(world.genesis.clone(), Execution::Individuals).unwrap();
    native.advance(1).unwrap();
    assert!(
        native
            .sim
            .state()
            .population
            .get(1)
            .unwrap()
            .parts
            .values()
            .any(|p| !p.lost.is_empty())
    );
    let crowd = Crowd::new(&world, 57, Variant::Histogram)
        .unwrap()
        .run()
        .unwrap();
    assert!(
        crowd
            .bins
            .keys()
            .any(|e| e.parts.values().any(|p| !p.lost.is_empty()))
    );
}

#[test]
fn multiple_world_actors_at_one_site_are_refused() {
    let mut world = certain();
    let actor = world.genesis.population.get(0).unwrap().clone();
    world.genesis.population.insert(actor, 1).unwrap();
    assert!(refuse(&world).contains("one actor per site"));
}

#[test]
fn cross_site_harm_is_refused() {
    let mut world = certain();
    world
        .genesis
        .rules
        .processes
        .get_mut("body:hazard")
        .unwrap()
        .target
        .as_mut()
        .unwrap()
        .same_place = false;
    assert!(refuse(&world).contains("agentless local process"));
}

#[test]
fn an_identity_chosen_target_is_refused() {
    let mut world = certain();
    world
        .genesis
        .rules
        .processes
        .get_mut("body:hazard")
        .unwrap()
        .target
        .as_mut()
        .unwrap()
        .weighted = false;
    assert!(refuse(&world).contains("weighted target"));
}

#[test]
fn two_wounds_in_one_act_are_refused() {
    let mut world = certain();
    world
        .genesis
        .rules
        .processes
        .get_mut("body:hazard")
        .unwrap()
        .effects
        .push(Effect::Wound {
            who: Binding::Target,
            cells: 1.into(),
            slot: 2,
        });
    assert!(refuse(&world).contains("one target wound or rot per act"));
}

#[test]
fn risky_harm_is_refused_even_when_only_its_risk_wounds() {
    let mut world = certain();
    let p = world
        .genesis
        .rules
        .processes
        .get_mut("body:hazard")
        .unwrap();
    p.risk = Some(Risk {
        per_million: 1_000_000,
        effects: std::mem::take(&mut p.effects),
    });
    assert!(refuse(&world).contains("without risk or notes"));
}
