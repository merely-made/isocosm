// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round for a participant who holds a lineage and plays no critter
//! (cases 1216, 1217, 1224, 1226).

use super::*;
use crate::{
    directing::{
        found::Played,
        holding_tests::{STRAIN, WORLD_LINE, genesis, hold, join, start, strain},
    },
    rules::{Hosting, year_ticks},
    spread::{self, Substrate, Surfaced},
};

const PACE: Pace = Pace { round: 2, scoring: 2 };

fn leaking() -> Option<Hosting> { Some(Hosting { period: 2, give: [1, 1], ..Hosting::default() }) }

/// A holder of `lineage` over `g`, playing nobody.
fn holder(g: Genesis, lineage: &str) -> (Interim, Id) {
    let mut s = start(g);
    let p = join(&mut s);
    hold(&mut s, p, lineage).unwrap();
    (Interim::resume(s, Mode::Creative, PACE).unwrap(), p)
}

fn surfaced(out: &[Happening]) -> Vec<&Surfaced> {
    out.iter().filter_map(|h| match h { Happening::Surfaced(e) => Some(e), _ => None }).collect()
}

fn ended(out: &[Happening]) -> usize {
    out.iter().filter(|h| matches!(h, Happening::LineageEnded { .. })).count()
}

#[test]
fn a_participant_who_plays_nothing_and_holds_nothing_has_no_round() {
    let mut s = start(genesis(&[], None, None));
    join(&mut s);
    let mut i = Interim::resume(s, Mode::Creative, PACE).unwrap();
    assert!(i.round(&no_candidates).is_err());
}

#[test]
fn a_holder_with_no_critter_runs_a_round_for_the_pace() {
    let (mut i, _) = holder(genesis(&[], None, None), STRAIN);
    assert_eq!(i.critter(), None);
    let from = i.session.sim.state().tick;
    let out = i.round(&no_candidates).unwrap();
    assert_eq!(i.session.sim.state().tick, from + PACE.round);
    assert!(out.is_empty(), "a still strain raises nothing: {out:?}");
}

#[test]
fn a_surface_that_reaches_a_new_host_raises_the_reach() {
    let mut g = genesis(&["gene:enter"], Some(Hosting { period: 2, ..Hosting::default() }), None);
    g.sites.get_mut(&0).unwrap().accounts.remove(STRAIN);
    g.sites.get_mut(&2).unwrap().accounts.insert(STRAIN.into(), 40);
    let (mut i, _) = holder(g, STRAIN);
    let out = i.round(&no_candidates).unwrap();
    let reached = |at| Surfaced::Reached { identity: strain(), at: Substrate::Host(at) };
    assert!(surfaced(&out).contains(&&reached(3)), "{out:?}");
    assert!(surfaced(&out).contains(&&reached(9)), "{out:?}");
    assert_eq!(ended(&out), 0);
}

#[test]
fn only_what_is_held_is_reported() {
    let mut g = genesis(&["gene:enter"], Some(Hosting { period: 2, ..Hosting::default() }), None);
    g.sites.get_mut(&2).unwrap().accounts.insert(STRAIN.into(), 40);
    let (mut i, p) = holder(g, WORLD_LINE);
    assert_eq!(i.session.sim.holds(p), vec![WORLD_LINE.to_string()]);
    let out = i.round(&no_candidates).unwrap();
    assert!(surfaced(&out).is_empty(), "the strain is not held: {out:?}");
}

#[test]
fn a_host_that_dies_in_the_round_raises_its_death() {
    let (mut i, p) = holder(genesis(&[], None, None), STRAIN);
    let mut session = i.session.clone();
    let mut borrowed = BorrowedInterim::new(&mut session, p, Mode::Creative, PACE, OnCollapse::Stay).unwrap();
    let mut killing = |s: &mut Session| {
        if s.sim.state().population.get(12).is_some_and(|e| e.alive) {
            s.command(Command::Kill { entity: 12 })?;
        }
        s.advance_tick_with_flows().map(|out| out.flows)
    };
    let (out, _) = borrowed.round(&no_candidates, &mut killing, &mut boundary::score).unwrap();
    assert!(surfaced(&out).contains(&&Surfaced::HostDied { identity: strain(), host: 12 }), "{out:?}");
    assert_eq!(ended(&out), 0, "the dead host still holds the matter");
    i.session = session;
}

#[test]
fn a_surface_emptied_everywhere_ends_once() {
    let mut g = genesis(&["gene:leak"], leaking(), None);
    g.sites.get_mut(&0).unwrap().accounts.remove(STRAIN);
    let (mut i, _) = holder(g, STRAIN);
    let out = i.round(&no_candidates).unwrap();
    assert!(surfaced(&out).contains(&&Surfaced::Left { identity: strain(), at: Substrate::Host(12) }), "{out:?}");
    assert_eq!(out.iter().filter(|h| **h == Happening::LineageEnded { lineage: STRAIN.into() }).count(), 1);
    let held = readings::held(&i.session.sim, STRAIN).unwrap();
    assert!(held.ended);
    assert_eq!(ended(&i.round(&no_candidates).unwrap()), 0, "it ended already");
}

#[test]
fn matter_in_transit_is_not_an_end() {
    let mut g = genesis(&["gene:leak"], leaking(), None);
    g.sites.get_mut(&0).unwrap().accounts.remove(STRAIN);
    g.population.lift(12).unwrap().accounts.remove(STRAIN);
    g.sites.get_mut(&0).unwrap().routes = vec![Route { to: 1, travel: 5, transmission: 500_000,
        border: None, authored: None }];
    g.spreads.transit.push(spread::Transfer { identity: strain(), from: 0, to: 1, departed: 0,
        arrives: 5, amount: 6 });
    let (mut i, _) = holder(g, STRAIN);
    let out = i.round(&no_candidates).unwrap();
    assert_eq!(ended(&out), 0, "{out:?}");
    assert!(!readings::held(&i.session.sim, STRAIN).unwrap().ended);
}

/// A year of forty ticks, so that it turns within a few rounds.
fn short_year(mut g: Genesis) -> Genesis {
    g.rules.tick_microseconds = Some(year_ticks(1) / 40);
    g
}

#[test]
fn a_world_holder_receives_what_the_world_raised_from_a_round_and_from_ahead() {
    let g = short_year(genesis(&[], None, Some(5)));
    let (mut i, _) = holder(g, WORLD_LINE);
    i.session.sim.state.tick = 19;
    let out = i.round(&no_candidates).unwrap();
    assert!(out.contains(&Happening::Raised(readings::Raised::Turned { rising: false })), "{out:?}");
    let before = i.session.sim.clone();
    let raised = i.ahead(2).unwrap();
    assert_eq!(raised, readings::raised(&before, &i.session.sim));
    let mut flip = short_year(genesis(&[], None, Some(5)));
    flip.rules.epoch_ticks = 2;
    let (mut j, _) = holder(flip, WORLD_LINE);
    j.session.sim.state.tick = 18;
    let raised = j.ahead(2).unwrap();
    assert!(raised.contains(&readings::Raised::Turned { rising: false }), "{raised:?}");
}

#[test]
fn a_holder_who_plays_a_critter_has_its_round_and_then_the_held_events() {
    let founded = |hold_it: bool| {
        let mut g = crate::Founding { seed: 101, sites: 5, population: 60, cohort_size: 4,
            lineages: 3, ecology: true, played: Some(Played { lineage: 1, region_sites: 2 }),
            ..Default::default() }.generate().unwrap();
        g.rules.epoch_ticks = 8;
        g.rules.tick_microseconds = Some(year_ticks(1) / 40);
        let mut i = Interim::found(g, Start { within: 0, epochs: 1, ..Start::default() }, Mode::Creative,
            Pace { round: 4, scoring: 6 }, Execution::Individuals).unwrap();
        let p = i.participant;
        if hold_it { hold(&mut i.session, p, "world:ground").unwrap(); }
        i.ending.since = i.session.sim.state().tick;
        i
    };
    let (mut plain, mut held) = (founded(false), founded(true));
    assert!(held.critter().is_some());
    let before = held.session.sim.clone();
    let (a, b) = (plain.round(&no_candidates).unwrap(), held.round(&no_candidates).unwrap());
    assert_eq!(held.session.sim.state().tick, plain.session.sim.state().tick);
    assert_eq!(b[..a.len()], a[..], "the critter's round is its own");
    let raised: Vec<_> = readings::raised(&before, &held.session.sim).into_iter().map(Happening::Raised).collect();
    assert_eq!(b[a.len()..], raised[..]);
}
