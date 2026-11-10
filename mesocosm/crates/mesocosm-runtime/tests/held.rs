// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The handle on a lineage with no critter body through the saved loop
//! (cases 1216, 1217, 1221, 1223, 1224, 1226): the seeded microbe, holding,
//! steering, a holder with no living body, the turn, and replay.

#[path = "round_endings/door.rs"]
#[allow(dead_code)]
mod door;

use cosmograph::{Founding, directing::{Played, interim::Happening}, simulation::Genesis,
    spread::{Identity, Substrate}};
use door::Door;
use mesocosm_runtime::{Refusal, continuation::{HeldLine, LoopAnswer, LoopPhase, Rounds}, microbe};
use std::collections::BTreeSet;

fn genesis(climate_turn: Option<i32>, hosting: bool) -> Genesis {
    let mut g = Founding { seed: 5, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }.generate().unwrap();
    if hosting { microbe::declare(&mut g).unwrap(); }
    if let Some(bound) = climate_turn {
        g.rules.climate = Some(cosmograph::rules::Climate { turn: Some(bound), ..Default::default() });
    }
    g
}

/// A door over a hosting world whose microbe is seeded, and the seed's site.
fn seeded() -> (Door, u64) {
    let mut d = Door::over(genesis(None, true));
    let site = *d.session.sim.state().sites.iter()
        .find(|(_, s)| s.accounts.contains_key(microbe::ACCOUNT)).unwrap().0;
    for command in microbe::seeding(site) { d.session.command(command).unwrap(); }
    (d, site)
}

fn micro() -> Identity { Identity::Surface(microbe::KEY.into()) }

fn stock(d: &Door, at: Substrate) -> u64 {
    cosmograph::spread::held(d.session.sim.state(), at, &micro()).unwrap()
}

fn surface_total(d: &Door) -> (u64, u64) {
    let held = cosmograph::directing::readings::held(&d.session.sim, microbe::KEY).unwrap();
    (held.sites.total, held.hosts.total)
}

#[test]
fn the_seeded_microbe_exists_conserves_matter_and_takes_dead_matter_and_a_body() {
    let plain = Door::over(genesis(None, false));
    let declared = Door::over(genesis(None, true));
    assert_eq!(plain.session.sim.matter(), declared.session.sim.matter(), "declaring moves matter, adds none");
    let (mut d, site) = seeded();
    assert_eq!(d.session.sim.matter(), plain.session.sim.matter(), "seeding leaves the world's matter as it was");
    assert_eq!(stock(&d, Substrate::Site(site)), microbe::SEED_AMOUNT);
    assert!(d.session.sim.is_surface(microbe::KEY));
    d.configure(Rounds::Capped, false);
    let mut entered = false;
    for _ in 0..60 {
        d.step();
        d.settle();
        let (_, hosts) = surface_total(&d);
        entered |= hosts > 0;
    }
    let (sites, hosts) = surface_total(&d);
    println!("after 60 rounds: sites {sites}, hosts {hosts}");
    assert!(sites + hosts > microbe::SEED_AMOUNT, "it took dead matter");
    assert!(entered, "it entered a body");
}

fn total(d: &Door) -> u64 {
    let held = cosmograph::directing::readings::held(&d.session.sim, microbe::KEY).unwrap();
    held.sites.total + held.hosts.total + held.places.total + held.transit
}

/// Rounds, settling births and losses and taking each epoch review's status quo.
fn run(d: &mut Door, rounds: usize) -> usize {
    let mut surfaced = 0;
    for _ in 0..rounds {
        d.step();
        surfaced += d.saved().happenings.iter().filter(|h| matches!(h, Happening::Surfaced(_))).count();
        if d.saved().review.is_some() { d.review().unwrap(); } else { d.settle(); }
    }
    surfaced
}

fn rot_off() -> LoopAnswer {
    LoopAnswer::Steer { lineage: microbe::KEY.into(), on: BTreeSet::new(), off: BTreeSet::from(["gene:rot".to_string()]) }
}

#[test]
fn holding_while_playing_shows_in_the_reading_and_steering_rot_off_stops_the_growth() {
    let (mut d, _) = seeded();
    d.configure(Rounds::Capped, false);
    assert!(d.reading().held.is_empty() && d.reading().holdable.contains(&microbe::KEY.to_string()));
    assert!(matches!(d.answer(rot_off()), Err(Refusal::Sim(_))), "a non-holder's steer is refused");
    d.answer(LoopAnswer::Hold { lineage: microbe::KEY.into() }).unwrap();
    let reading = d.reading();
    assert!(!reading.holdable.contains(&microbe::KEY.to_string()), "held, so no longer holdable");
    let [HeldLine::Surface(line)] = reading.held.as_slice() else { panic!("{:?}", reading.held) };
    assert_eq!((line.lineage.as_str(), line.sites.total, line.ended), (microbe::KEY, microbe::SEED_AMOUNT, false));
    assert!(line.traits.contains("gene:rot"));
    let surfaced = run(&mut d, 30);
    assert!(surfaced > 0, "a critter's round carries what befell the held microbe");
    assert!(total(&d) > microbe::SEED_AMOUNT, "rotting, it grew");
    d.answer(rot_off()).unwrap();
    let held = total(&d);
    assert!(!d.reading().held.iter().any(|l| matches!(l, HeldLine::Surface(s) if s.traits.contains("gene:rot"))));
    run(&mut d, 30);
    assert_eq!(total(&d), held, "with rot off it takes nothing more");
    d.answer(LoopAnswer::Unhold { lineage: microbe::KEY.into() }).unwrap();
    assert!(d.reading().held.is_empty());
}

#[test]
fn a_session_with_holds_replays_to_its_hash_and_a_plain_loop_saves_no_held() {
    let (mut d, _) = seeded();
    d.configure(Rounds::Capped, false);
    let wire = serde_json::to_string(&d.reading()).unwrap();
    assert!(!wire.contains("\"held\""), "nothing held, nothing keyed");
    assert!(!serde_json::to_string(&d.game).unwrap().contains("held"), "the saved loop never carries them");
    d.answer(LoopAnswer::Hold { lineage: microbe::KEY.into() }).unwrap();
    run(&mut d, 10);
    assert!(serde_json::to_string(&d.reading()).unwrap().contains("\"held\""));
    let replayed = cosmograph::Session::load(d.session.save(), cosmograph::Execution::Grouped).unwrap();
    assert_eq!(replayed.sim.state_hash(), d.session.sim.state_hash());
    let plain = Door::over(genesis(None, false));
    assert!(!serde_json::to_string(&plain.game).unwrap().contains("held"));
}

#[test]
fn a_holder_of_the_world_class_lineage_turns_the_cycles_within_the_bound() {
    let mut d = Door::over(genesis(Some(5), false));
    d.configure(Rounds::Capped, false);
    let world = d.reading().holdable.into_iter().find(|key| key == "world:ground").expect("the world-class lineage is holdable");
    let turn = |shift| LoopAnswer::Turn { lineage: "world:ground".into(), shift };
    assert!(matches!(d.answer(turn(2)), Err(Refusal::Sim(_))), "not held yet");
    d.answer(LoopAnswer::Hold { lineage: world }).unwrap();
    d.answer(turn(2)).unwrap();
    assert!(matches!(d.reading().held.as_slice(),
        [HeldLine::World { shift: 2, bound: Some(5), .. }]), "{:?}", d.reading().held);
    assert!(matches!(d.answer(turn(6)), Err(Refusal::Sim(_))), "beyond the bound");
    assert!(matches!(d.answer(turn(-6)), Err(Refusal::Sim(_))));
    d.answer(turn(-5)).unwrap();
    assert!(matches!(d.reading().held.as_slice(), [HeldLine::World { shift: -5, .. }]));
    assert_ne!(d.phase(), LoopPhase::Held);
}
