// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Time flow (case 1216): the world runs ahead, unwatched, mid-play.

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

fn replays(i: &Interim) {
    let json = serde_json::to_vec(&i.session.save()).unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), i.session.sim.state_hash());
        assert_eq!(loaded.entries, i.session.entries);
    }
}

#[test]
fn ahead_runs_whole_epochs_with_every_line_adapting_and_names_what_it_raised() {
    for seed in [101u64, 202] {
        let mut i = found(seed);
        let (start, before) = (tick(&i), i.session.sim.clone());
        // The unattended path on a copy: the boundaries it would hold.
        let mut copy = i.session.clone();
        let (handover, held) = deep::run_deep_time_adapting(&mut copy,
            DeepTimeSpan { epochs: 3 }, PACE.scoring).unwrap();
        assert_eq!(held.len(), 3, "a boundary at each epoch");
        let ticks: Vec<_> = held.iter().map(|h| match h {
            Happening::Boundary { tick, .. } => *tick,
            _ => panic!("only boundaries"),
        }).collect();
        assert_eq!(ticks, [start + EPOCH, start + 2 * EPOCH, start + 3 * EPOCH]);
        let raised = i.ahead(3).unwrap();
        assert_eq!(tick(&i), start + 3 * EPOCH, "seed {seed}");
        assert_eq!(handover.to_tick, tick(&i));
        assert_eq!(i.session.sim.state_hash(), copy.sim.state_hash(), "the deep-time path itself");
        assert_eq!(raised, readings::raised(&before, &i.session.sim));
        assert_eq!(i.ending.since, tick(&i), "the epoch begins again here");
        assert!(i.session.sim.state().population.get(i.participant).is_some_and(super::super::is_participant));
        replays(&i);
        i.round(&no_candidates).unwrap();
        replays(&i);
    }
}

#[test]
fn ahead_from_mid_epoch_runs_to_the_boundary_it_names() {
    let mut i = found(101);
    i.round(&no_candidates).unwrap();
    assert_ne!(tick(&i) % EPOCH, 0);
    let from = tick(&i);
    i.ahead(1).unwrap();
    assert_eq!(tick(&i), (from / EPOCH + 1) * EPOCH);
    replays(&i);
}

#[test]
fn ahead_of_nothing_changes_nothing_and_a_refusal_leaves_the_loop_untouched() {
    let mut i = found(101);
    let hash = i.session.sim.state_hash();
    assert!(i.ahead(0).unwrap().is_empty());
    assert_eq!(i.session.sim.state_hash(), hash);
    i.pace.scoring = 0;
    let (entries, ending) = (i.session.entries.clone(), i.ending);
    assert!(i.ahead(1).is_err());
    assert_eq!(i.session.sim.state_hash(), hash);
    assert_eq!(i.session.entries, entries);
    assert_eq!(i.ending, ending);
}

#[test]
fn a_critter_dead_before_ahead_is_succeeded_at_the_next_round() {
    let mut i = found(101);
    let first = i.critter().unwrap();
    i.session.sim.state.population.lift(first).unwrap().alive = false;
    i.ahead(1).unwrap();
    assert_eq!(i.critter(), Some(first), "ahead takes up nobody for the dead");
    let out = i.round(&no_candidates).unwrap();
    let next = out.iter().find_map(|h| match h {
        Happening::Died { critter, next } if *critter == first => Some(*next),
        _ => None,
    });
    let next = next.expect("the round answers the death").expect("the lineage has an heir");
    assert_eq!(i.critter(), Some(next));
    assert!(i.session.sim.state().population.get(next).unwrap().alive);
}
