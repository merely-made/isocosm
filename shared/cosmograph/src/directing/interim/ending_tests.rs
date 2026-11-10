// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! How an epoch ends (cases 1199, 1212, 1217; lineage-rounds D7, D8).

use super::*;
use crate::Founding;
use crate::directing::found::Played;

const PACE: Pace = Pace { round: 4, scoring: 6 };
const EPOCH: Tick = 8;

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
    g.rules.epoch_ticks = EPOCH;
    g
}

fn found(seed: u64) -> Interim {
    let start = Start { within: 0, epochs: 1, ..Start::default() };
    let mut i = Interim::found(genesis(seed), start, Mode::Creative, PACE,
        Execution::Individuals).unwrap();
    i.ending.since = i.session.sim.state().tick;
    i
}

fn tick(i: &Interim) -> Tick { i.session.sim.state().tick }

/// One round: the happenings and the tick it ended on.
fn step(i: &mut Interim) -> (Vec<Happening>, Tick) {
    let out = i.round(&no_candidates).unwrap();
    (out, tick(i))
}

fn boundaries(out: &[Happening]) -> usize {
    out.iter().filter(|h| matches!(h, Happening::Boundary { .. })).count()
}

fn passed(out: &[Happening]) -> usize {
    out.iter().filter(|h| matches!(h, Happening::SpanPassed { .. })).count()
}

fn ended(out: &[Happening]) -> bool {
    out.iter().any(|h| matches!(h, Happening::LineageEnded { .. } | Happening::WorldEnded { .. }))
}

#[test]
fn the_default_ending_fires_where_the_timed_rule_predicts() {
    let mut i = found(101);
    assert_eq!(i.ending, Ending::default().tap_since(i.ending.since));
    for _ in 0..6 {
        let from = tick(&i);
        let (out, now) = step(&mut i);
        assert_eq!(boundaries(&out), usize::from(now / EPOCH > from / EPOCH));
        assert_eq!(passed(&out), 0);
        if ended(&out) { break; }
    }
}

trait Tap { fn tap_since(self, since: Tick) -> Self; }
impl Tap for Ending {
    fn tap_since(mut self, since: Tick) -> Self { self.since = since; self }
}

#[test]
fn waiting_without_a_call_never_ends_and_passes_the_span_once() {
    let mut i = found(101);
    i.ending.wait = true;
    let (mut fired, mut spans) = (0, 0);
    for _ in 0..5 {
        let (out, _) = step(&mut i);
        (fired, spans) = (fired + boundaries(&out), spans + passed(&out));
        if ended(&out) { break; }
    }
    assert!(tick(&i) - i.ending.since > 2 * EPOCH);
    assert_eq!((fired, spans), (0, 1));
}

#[test]
fn a_call_is_honoured_once_the_span_has_passed_and_not_before() {
    let mut i = found(101);
    i.ending.wait = true;
    let begun = i.ending.since;
    i.ending.call = true;
    let (out, now) = step(&mut i);
    assert!(now - begun < EPOCH);
    assert_eq!(boundaries(&out), 0, "the span has not passed");
    assert!(i.ending.call);
    let (out, now) = step(&mut i);
    assert_eq!(boundaries(&out), 1);
    assert_eq!(passed(&out), 1);
    assert_eq!((i.ending.since, i.ending.call, i.ending.wait), (now, false, true));
    for _ in 0..2 {
        let (out, _) = step(&mut i);
        assert_eq!(boundaries(&out), 0, "a second epoch needs a fresh call");
    }
    i.ending.call = true;
    let (out, now) = step(&mut i);
    assert_eq!(boundaries(&out), 1);
    assert_eq!(i.ending.since, now);
}

#[test]
fn a_forfeit_ends_the_epoch_before_the_span_under_both_rules() {
    for wait in [false, true] {
        let mut i = found(101);
        i.ending.wait = wait;
        i.ending.forfeit = true;
        let begun = tick(&i);
        let (out, now) = step(&mut i);
        assert!(now - begun < EPOCH);
        assert_eq!(boundaries(&out), 1, "wait {wait}");
        assert_eq!((i.ending.forfeit, i.ending.since, i.ending.wait), (false, now, wait));
    }
}

#[test]
fn an_extension_skips_one_timed_boundary_then_the_rule_resumes() {
    let mut i = found(101);
    i.ending.extend = true;
    let (mut crossings, mut fired) = (0, 0);
    for _ in 0..8 {
        let from = tick(&i);
        let (out, now) = step(&mut i);
        crossings += usize::from(now / EPOCH > from / EPOCH);
        fired += boundaries(&out);
        if ended(&out) { return; }
    }
    assert!(crossings >= 3);
    assert_eq!(fired, crossings - 1);
    assert!(!i.ending.extend);
}

#[test]
fn par_is_the_most_held_in_an_unnudged_copy_and_leaves_the_world_alone() {
    let (mut a, mut b) = (found(202), found(202));
    let lineage = "lineage:1";
    let hash = a.session.sim.state_hash();
    let par = boundary::par(&a.session, lineage, EPOCH).unwrap();
    assert_eq!(a.session.sim.state_hash(), hash);
    let mut most = boundary::standing(&b.session, lineage).0;
    for _ in 0..EPOCH {
        b.session.advance(1).unwrap();
        most = most.max(boundary::standing(&b.session, lineage).0);
    }
    assert_eq!(par, most);
    assert_eq!(a.set_par().unwrap(), Some(par));
    assert_eq!(a.ending.par, Some(par));
}

#[test]
fn a_met_condition_ends_the_epoch_and_an_unreachable_one_does_not() {
    let mut i = found(101);
    i.ending.wait = true;
    assert!(boundary::standing(&i.session, "lineage:1").0 > 0);
    i.ending.par = Some(u128::MAX);
    let (out, _) = step(&mut i);
    assert_eq!(boundaries(&out), 0);
    assert_eq!(i.ending.par, Some(u128::MAX));
    i.ending.par = Some(0);
    let (out, _) = step(&mut i);
    let at = out.iter().position(|h| matches!(h, Happening::ConditionMet { par: 0, .. })).unwrap();
    assert!(matches!(out[at + 1], Happening::Boundary { .. }));
    assert_eq!(i.ending.par, None);
}

#[test]
fn a_waiting_run_with_calls_replays_to_the_same_hash() {
    for seed in [101u64, 202, 303] {
        let mut i = found(seed);
        i.ending.wait = true;
        for n in 0..6 {
            i.ending.call = n % 2 == 1;
            let (out, _) = step(&mut i);
            if ended(&out) { break; }
        }
        let json = serde_json::to_vec(&i.session.save()).unwrap();
        let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
        assert_eq!(replayed.sim.state_hash(), i.session.sim.state_hash(), "seed {seed}");
    }
}
