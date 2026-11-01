// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The lineage policy's controls (mesocosm-overlay M3.5).

use super::policy::{self, LineagePolicy, UNIT};
use super::tests::{founding, played};
use super::*;
use crate::{
    Execution, Session,
    history::Command,
    lineage::{boundary, revise},
    rules::{Causation, PolicyVariation},
};

/// `world`'s ecology with lineage policies varying as `v` says.
fn varying(seed: u64, v: PolicyVariation) -> Session {
    let mut g = founding(seed);
    g.rules.directing.as_mut().unwrap().policy = v;
    Session::new(g, Execution::Individuals).unwrap()
}

/// `world`'s ecology with every Choice process due each tick, so each
/// decision weighs them all against one another.
fn contested(seed: u64) -> Session {
    let mut g = founding(seed);
    for p in g.rules.processes.values_mut() {
        if p.causation == Causation::Choice && p.period.is_some() {
            p.period = Some(1);
        }
    }
    Session::new(g, Execution::Individuals).unwrap()
}

/// A larger ecology at the played pace and A1's rates, for the readout.
fn lasting(seed: u64, v: PolicyVariation) -> Session {
    let mut g = crate::Founding {
        seed,
        sites: 8,
        population: 160,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        ecology_rates: crate::ecology::Rates::A1,
        played: Some(Played {
            lineage: 1,
            region_sites: 4,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.directing.as_mut().unwrap().policy = v;
    Session::new(g, Execution::Individuals).unwrap()
}

/// The played critter's lineage.
fn line(s: &mut Session) -> Key {
    let (_, c) = played(s);
    s.sim.state().population.get(c).unwrap().lineage.clone()
}

/// The state with every policy and draw taken out, to compare with a run
/// that never had one.
fn without_policies(mut s: Session) -> u64 {
    for l in s.sim.state.lineages.values_mut() {
        l.policy = None;
    }
    for g in s.sim.state.population.groups.values_mut() {
        if g.entity.drawn.is_some() {
            g.entity_mut().drawn = None;
        }
    }
    s.sim.state_hash()
}

fn adopt(s: &mut Session, lineage: &Key, policy: LineagePolicy) {
    s.command(Command::Policy {
        lineage: lineage.clone(),
        policy,
    })
    .unwrap();
}

#[test]
fn a_founding_policy_chooses_as_the_hand_written_score_does() {
    for seed in [3, 7, 11] {
        let mut plain = varying(seed, PolicyVariation::default());
        let lineage = line(&mut plain);
        let mut minded = plain.clone();
        let found = policy::founding(&minded, &lineage);
        minded.command(found).unwrap();
        plain.advance(40).unwrap();
        minded.advance(40).unwrap();
        let tally = minded.sim.chosen.tally[&lineage];
        assert!(tally.decisions > 0, "seed {seed}: the policy chose");
        assert_eq!(tally.otherwise, 0, "seed {seed}: never otherwise");
        assert_eq!(
            without_policies(minded),
            plain.sim.state_hash(),
            "seed {seed}: the same world but for the policy"
        );
    }
}

#[test]
fn a_world_without_policies_hashes_as_before() {
    let mut s = varying(5, PolicyVariation::default());
    line(&mut s);
    let before = s.clone();
    s.advance(20).unwrap();
    let mut again = before;
    again.advance(20).unwrap();
    assert_eq!(s.sim.state_hash(), again.sim.state_hash());
    assert!(s.sim.state().lineages.values().all(|l| l.policy.is_none()));
    assert!(s.sim.chosen.tally.is_empty(), "no policy, no tally");
}

#[test]
fn a_varied_policy_replays_to_the_same_hash() {
    let v = PolicyVariation {
        founding: 12,
        expression: 12,
        candidates: 2,
    };
    let run = |seed| {
        let mut s = varying(seed, v);
        let lineage = line(&mut s);
        let found = policy::founding(&s, &lineage);
        s.command(found).unwrap();
        s.advance(40).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(run(9), run(9));
    assert_ne!(run(9), run(10));
}

#[test]
fn a_policy_that_misfits_the_world_is_refused() {
    let mut s = varying(4, PolicyVariation::default());
    let lineage = line(&mut s);
    let needs = crate::meaning::needs(&s.sim.genesis().rules).len();
    let short = LineagePolicy::founding(needs + 1);
    let refused = s.command(Command::Policy {
        lineage,
        policy: short,
    });
    assert!(refused.is_err());
}

#[test]
fn the_boundary_adopts_a_better_variant_and_not_a_worse_one() {
    let mut s = contested(3);
    let lineage = line(&mut s);
    let needs = crate::meaning::needs(&s.sim.genesis().rules).len();
    let good = LineagePolicy::founding(needs);
    // Planted: drives turned against every need, so nothing is served.
    let bad = LineagePolicy {
        drives: vec![-UNIT; needs],
        ..good.clone()
    };
    let offer = |p: &LineagePolicy, name: &str| boundary::Candidate {
        name: name.into(),
        commands: vec![Command::Policy {
            lineage: lineage.clone(),
            policy: p.clone(),
        }],
    };
    let turn = |s: &mut Session, offered: boundary::Candidate| {
        let only = |_: &Session, l: &str| {
            if l == lineage { vec![offered.clone()] } else { vec![] }
        };
        let turns = boundary::adapt(s, None, &only, 30).unwrap();
        turns.into_iter().find(|t| t.lineage == lineage).unwrap()
    };
    let mut worse = s.clone();
    adopt(&mut worse, &lineage, bad.clone());
    let t = turn(&mut worse, offer(&good, "good"));
    assert_eq!(t.committed.as_deref(), Some("good"), "{:?}", t.considered);
    adopt(&mut s, &lineage, good);
    let t = turn(&mut s, offer(&bad, "bad"));
    assert_eq!(t.committed, None, "{:?}", t.considered);
}

#[test]
fn the_boundary_offers_prospering_members_variants() {
    let v = PolicyVariation {
        founding: 0,
        expression: 16,
        candidates: 2,
    };
    let mut s = varying(6, v);
    let lineage = line(&mut s);
    let found = policy::founding(&s, &lineage);
    s.command(found).unwrap();
    let offered = revise::revisions(&s, &lineage);
    let variants: Vec<_> = offered
        .iter()
        .filter(|c| c.name.starts_with("policy:member-"))
        .collect();
    assert!(!variants.is_empty() && variants.len() <= 2, "{offered:?}");
    let own = s.sim.state().lineages[&lineage].policy.clone().unwrap();
    for c in variants {
        let Command::Policy { policy, .. } = &c.commands[0] else {
            panic!("a policy candidate commits a policy");
        };
        assert_ne!(*policy, own, "a member's variant differs from its line's");
    }
}

/// Epochs of a varied policy beside its hand-written twin: what the policy
/// chose otherwise, what the boundary committed, and how the line stood.
/// `cargo test policy_drift -- --ignored --nocapture`.
#[test]
#[ignore = "a readout, not a control"]
fn policy_drift() {
    let v = PolicyVariation {
        founding: 16,
        expression: 16,
        candidates: 3,
    };
    for seed in [9, 3] {
        let mut twin = lasting(seed, v);
        let lineage = line(&mut twin);
        let mut s = twin.clone();
        let found = policy::founding(&s, &lineage);
        s.command(found).unwrap();
        println!("seed {seed}, {lineage}");
        println!("epoch  chose  otherwise  members  held      twin members  twin held  committed");
        for epoch in 0..6 {
            s.advance(30).unwrap();
            twin.advance(30).unwrap();
            let reading = policy::readout(&mut s)
                .into_iter()
                .find(|r| r.lineage == lineage);
            let (t_held, t_members) = boundary::standing(&twin, &lineage);
            let turns = boundary::adapt(&mut s, None, &revise::revisions, 20).unwrap();
            let committed = turns
                .iter()
                .find(|t| t.lineage == lineage)
                .and_then(|t| t.committed.clone())
                .unwrap_or_else(|| "-".into());
            boundary::adapt(&mut twin, None, &revise::revisions, 20).unwrap();
            match reading {
                Some(r) => println!(
                    "{epoch:>5}  {:>5}  {:>9}  {:>7}  {:<8}  {:>12}  {:<9}  {committed}",
                    r.tally.decisions, r.tally.otherwise, r.members, r.held, t_members, t_held
                ),
                None => println!("{epoch:>5}  the line is gone"),
            }
        }
        if let Some(p) = &s.sim.state().lineages[&lineage].policy {
            println!("drives now {:?} (64 is the hand-written weight)", p.drives);
        }
    }
}

/// What a policy costs: one evaluation in isolation, and a world's ticks
/// beside its hand-written twin's. `cargo test policy_cost -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement"]
fn policy_cost() {
    use std::time::Instant;
    let needs = 4;
    let p = LineagePolicy::founding(needs).varied(1, 16);
    let t = policy::Terms {
        needs: vec![(0, 1200), (2, 300)],
        mood: -6,
        priority: 2,
        shape: [false, true, true, true],
    };
    let n = 200_000;
    let start = Instant::now();
    let mut sum = 0i64;
    for i in 0..n {
        sum = sum.wrapping_add(p.varied(i, 16).score(&t));
    }
    let varied = start.elapsed() / n as u32;
    let start = Instant::now();
    for _ in 0..n {
        sum = sum.wrapping_add(std::hint::black_box(&p).score(&t));
    }
    let scored = start.elapsed() / n as u32;
    println!("one option: {scored:?} to score, {varied:?} to vary and score ({sum})");
    let v = PolicyVariation {
        founding: 16,
        expression: 16,
        candidates: 3,
    };
    let mut twin = lasting(9, v);
    let lineage = line(&mut twin);
    let mut s = twin.clone();
    let found = policy::founding(&s, &lineage);
    s.command(found).unwrap();
    let start = Instant::now();
    twin.advance(90).unwrap();
    let plain = start.elapsed();
    let start = Instant::now();
    s.advance(90).unwrap();
    let minded = start.elapsed();
    let r = policy::readout(&mut s).into_iter().find(|r| r.lineage == lineage).unwrap();
    println!(
        "90 ticks: {plain:?} hand-written, {minded:?} with the policy, {} decisions, peak near {} members",
        r.tally.decisions,
        boundary::standing(&twin, &lineage).1
    );
}
