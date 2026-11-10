// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Starting a round at a picked site (case 1211, lineage-rounds D5).

use super::*;
use crate::Founding;
use crate::directing::found::Played;
use readings::{StartAt, start_at, starts};

const LINEAGE: &str = "lineage:1";
const PACE: Pace = Pace { round: 4, scoring: 6 };

fn genesis(seed: u64) -> Genesis {
    let mut g = Founding {
        seed,
        sites: 5,
        population: 60,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.epoch_ticks = 8;
    g
}

fn found(seed: u64, pick: Option<Id>) -> Result<Interim> {
    let start = Start { pick, ..Start::default() };
    Interim::found(genesis(seed), start, Mode::Creative, PACE, Execution::Individuals)
}

/// Sites 0-1-2-3-4 in a line, with the lineage's living members planted at
/// `places`, one member each (the rest of the lineage dead), each playable or not.
fn line(places: &[(Id, Method)]) -> Session {
    let mut g = genesis(7);
    let template = g.sites[&0].routes[0].clone();
    for (&id, site) in g.sites.iter_mut() {
        site.routes = [id.wrapping_sub(1), id + 1]
            .into_iter()
            .filter(|to| *to < 5)
            .map(|to| Route { to, ..template.clone() })
            .collect();
    }
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let ours = ids(&s);
    assert!(ours.len() >= places.len(), "the seed has {} members", ours.len());
    for (n, first) in ours.into_iter().enumerate() {
        let e = s.sim.state.population.lift(first).unwrap();
        e.journey = None;
        match places.get(n) {
            Some(&(site, method)) => (e.alive, e.place, e.method) = (true, site, method),
            None => e.alive = false,
        }
    }
    s
}

/// The lineage's members, one id each, lowest first.
fn ids(s: &Session) -> Vec<Id> {
    s.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.lineage == LINEAGE).flat_map(|(f, g)| *f..*f + g.count).collect()
}

#[test]
fn starts_list_the_range_then_the_fringe_with_the_nearest_body() {
    use Method::{Deliberative as D, Reactive as R};
    // Range {1, 4}: site 1 holds an unplayable member before two playable ones.
    let s = line(&[(1, R), (1, D), (1, D), (4, D)]);
    let m = ids(&s);
    let all = starts(&s.sim, LINEAGE);
    let range = |site, critter| StartAt { site, critter, journey: false };
    let fringe = |site, critter| StartAt { site, critter, journey: true };
    // The rules hold moves from site 1 to both neighbours and none from 4 to
    // 3, so site 3 is beside the range and no fringe: its member cannot walk
    // there and starts where it is.
    assert_eq!(all, vec![range(1, m[1]), range(4, m[3]), fringe(0, m[1]), fringe(2, m[1])]);
    let why = start_at(&s.sim, LINEAGE, 3).unwrap_err();
    assert!(why.contains("can walk to site 3"), "{why}");
}

#[test]
fn a_range_site_without_a_playable_body_is_omitted_and_refused() {
    let s = line(&[(1, Method::Reactive), (4, Method::Deliberative)]);
    let all = starts(&s.sim, LINEAGE);
    assert!(all.iter().all(|a| a.site != 1));
    let why = start_at(&s.sim, LINEAGE, 1).unwrap_err();
    assert!(why.contains(LINEAGE) && why.contains("none is playable"), "{why}");
}

#[test]
fn a_site_two_steps_beyond_the_range_is_refused_by_name() {
    let s = line(&[(1, Method::Deliberative)]);
    assert!(start_at(&s.sim, LINEAGE, 2).unwrap().journey);
    let why = start_at(&s.sim, LINEAGE, 3).unwrap_err();
    assert!(why.contains(LINEAGE) && why.contains("site 3") && why.contains("outside"), "{why}");
}

#[test]
fn a_lineage_with_no_living_member_has_no_start() {
    let s = line(&[]);
    assert!(starts(&s.sim, LINEAGE).is_empty());
    let why = start_at(&s.sim, LINEAGE, 1).unwrap_err();
    assert!(why.contains(LINEAGE) && why.contains("no living member"), "{why}");
}

#[test]
fn a_picked_range_site_starts_in_its_lowest_playable_member() {
    for seed in [101u64, 202, 303] {
        let probe = found(seed, None).unwrap();
        let pick = starts(&probe.session.sim, LINEAGE).into_iter().find(|a| !a.journey)
            .expect("a range start").site;
        let i = found(seed, Some(pick)).unwrap();
        let sim = &i.session.sim;
        let taken = i.critter().unwrap();
        assert_eq!(sim.state().population.get(taken).unwrap().resident_site(), Some(pick));
        let lowest = sim.state().population.groups.iter().find(|(_, g)| {
            let e = &g.entity;
            e.alive && e.lineage == LINEAGE && e.method == Method::Deliberative
                && e.resident_site() == Some(pick)
        });
        assert_eq!(Some(taken), lowest.map(|(f, _)| *f), "seed {seed}");
    }
}

#[test]
fn a_picked_fringe_site_takes_the_nearest_body_and_nudges_it_there() {
    let fringe = (1u64..40).find_map(|seed| {
        let probe = found(seed, None).ok()?;
        let at = starts(&probe.session.sim, LINEAGE).into_iter().find(|a| a.journey)?;
        Some((seed, at))
    });
    let (seed, at) = fringe.expect("a seed with a fringe start");
    let i = found(seed, Some(at.site)).unwrap();
    assert_eq!(i.critter(), Some(at.critter));
    let nudges = &i.session.sim.state().nudges;
    let last = nudges.last().expect("a fringe start nudges its body");
    assert_eq!((last.critter, last.toward), (at.critter, crate::directing::Toward::Site(at.site)));
    let beyond = found(seed, Some(999)).unwrap_err();
    assert!(beyond.contains("outside"), "{beyond}");
}

#[test]
fn no_pick_takes_the_first_life_as_before() {
    for seed in [101u64, 202, 303] {
        let i = found(seed, None).unwrap();
        // The old rule: the lowest living deliberative member at a habitable
        // site, else the lowest anywhere.
        let sim = &i.session.sim;
        let homes = readings::habitable(sim, LINEAGE);
        let ours: Vec<_> = sim.state().population.groups.iter().filter(|(_, g)| {
            g.entity.alive && g.entity.lineage == LINEAGE && g.entity.method == Method::Deliberative
        }).collect();
        let home = ours.iter().find(|(_, g)| g.entity.resident_site().is_some_and(|s| homes.contains(&s)));
        let old = home.or(ours.first()).map(|(f, _)| **f);
        assert_eq!(i.critter(), old, "seed {seed}");
    }
}

#[test]
fn starts_read_the_same_from_a_replayed_run() {
    let mut i = found(7, None).unwrap();
    for _ in 0..3 { i.round(&no_candidates).unwrap(); }
    let json = serde_json::to_vec(&i.session.save()).unwrap();
    let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(replayed.sim.state_hash(), i.session.sim.state_hash());
    let live = starts(&i.session.sim, LINEAGE);
    assert!(!live.is_empty());
    assert_eq!(live, starts(&replayed.sim, LINEAGE));
    for at in &live {
        assert_eq!(start_at(&i.session.sim, LINEAGE, at.site), Ok(*at));
    }
}
