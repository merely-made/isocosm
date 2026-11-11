// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The consumers' limits: what they lay, on whom, and what they do.

use super::{limits::{CEILING, FULL}, *};
use crate::{Execution, Session, bodied::Bodies, directing::Played};
use std::collections::BTreeSet;

fn founding(limits: Option<Limits>) -> Founding {
    Founding {
        seed: 9,
        sites: 8,
        population: 200,
        cohort_size: 4,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::A1,
        web: Web::Diets,
        bodies: Some(Bodies::default()),
        played: Some(Played { lineage: 1, region_sites: 4 }),
        limits,
        ..Default::default()
    }
}

fn consumer(i: u32) -> bool {
    Role::of(Web::Diets, i).consumer()
}

fn big() -> Limits {
    Limits { full: 1_000_000, ceiling: [1_000_000, 1] }
}

#[test]
fn unset_limits_found_and_hash_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("limits"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    assert_eq!(crate::digest(&plain.generate().unwrap()), crate::digest(&bare.generate().unwrap()));
}

#[test]
fn limits_round_trip() {
    let f = founding(Some(Limits { full: 11, ceiling: [5, 2] }));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"limits\""));
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    assert_eq!(Limits::default(), Limits { full: FULL, ceiling: CEILING });
}

#[test]
fn the_ceiling_is_a_ratio_of_the_founding_share() {
    let l = |n, d| Limits { full: 1, ceiling: [n, d] };
    assert_eq!(l(3, 1).most(6), 18);
    assert_eq!(l(2, 1).most(6), 12);
    assert_eq!(l(3, 2).most(5), 8, "rounded up");
    assert_eq!(l(1, 1000).most(1), 1);
}

/// The requires a process holds that another does not.
fn added(with: &Process, without: &Process) -> Vec<Query> {
    assert_eq!(with.requires[..without.requires.len()], without.requires[..], "only appended");
    with.requires[without.requires.len()..].to_vec()
}

#[test]
fn the_limits_gate_the_feed_and_birth_of_consumers_of_the_living_and_nobody_else() {
    let (off, on) = (founding(None).generate().unwrap(), founding(Some(Limits::default())).generate().unwrap());
    assert_eq!(off.rules.processes.keys().collect::<Vec<_>>(), on.rules.processes.keys().collect::<Vec<_>>());
    for (id, p) in &on.rules.processes {
        let plain = &off.rules.processes[id];
        let extra = added(p, plain);
        let i = id.rsplit('-').next().and_then(|n| n.parse::<u32>().ok());
        let gated = i.is_some_and(consumer);
        let mut same = p.clone();
        same.requires.truncate(plain.requires.len());
        assert_eq!(&same, plain, "{id} changes only in its requires");
        match (id.rsplit_once('-').map(|(k, _)| k), gated) {
            (Some("ecology:feed"), true) => assert_eq!(extra,
                [Query::Below { who: Binding::Actor, key: format!("matter:{}-0", i.unwrap()), amount: FULL }]),
            (Some("ecology:birth"), true) => {
                let Query::MembersBelow { among, amount } = &extra[0] else { panic!("{id}: {extra:?}") };
                assert_eq!(extra.len(), 1);
                assert_eq!(among, &BTreeSet::from([format!("lineage:{}", i.unwrap())]));
                assert!(*amount >= 3, "a share of at least one, three times");
            },
            _ => assert!(extra.is_empty(), "{id} gains {extra:?}"),
        }
    }
    // Producers and the decomposer have neither gate; the three consumers both.
    assert!(!on.rules.processes.contains_key("ecology:feed-0"));
    assert_eq!(on.rules.processes["ecology:feed-2"].requires, off.rules.processes["ecology:feed-2"].requires);
    assert_ne!(on.rules.processes["ecology:feed-1"].requires, off.rules.processes["ecology:feed-1"].requires);
}

#[test]
fn the_ceiling_follows_each_lineages_founded_share() {
    let g = founding(Some(Limits::default())).generate().unwrap();
    for i in (0..5).filter(|&i| consumer(i)) {
        let n: u64 = g.population.groups.values()
            .filter(|c| c.entity.alive && c.entity.lineage == format!("lineage:{i}")).map(|c| c.count).sum();
        let want = Limits::default().most(n.div_ceil(g.sites.len() as u64).max(1));
        let birth = &g.rules.processes[&format!("ecology:birth-{i}")];
        assert!(birth.requires.iter().any(|q| matches!(q, Query::MembersBelow { amount, .. } if *amount == want)), "lineage {i}");
    }
}

/// Tissue held by the living of each consumer lineage after `ticks`, and the
/// consumers born since the founding.
fn after(limits: Limits, ticks: u64) -> (u64, usize) {
    let g = founding(Some(limits)).generate().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    s.advance(ticks).unwrap();
    let mut most = 0;
    let mut born = 0;
    for c in s.sim.state().population.groups.values() {
        let Some(i) = c.entity.lineage.strip_prefix("lineage:").and_then(|n| n.parse::<u32>().ok()) else { continue };
        if !consumer(i) || !c.entity.alive {
            continue;
        }
        most = most.max(crate::anatomy::books(&c.entity).get(&format!("matter:{i}-0")).copied().unwrap_or(0));
        born += usize::from(c.entity.born > 0);
    }
    (most, born)
}

#[test]
fn a_full_consumer_does_not_feed_and_a_hungry_one_does() {
    // A founded body holds 10. Below a fullness of 1 every consumer is full
    // and none eats, so none holds more than it was founded with; with a
    // fullness far above, some eat past it.
    let (full, _) = after(Limits { full: 1, ..big() }, 40);
    assert!(full <= 10, "no consumer ate: {full}");
    let (hungry, _) = after(big(), 40);
    assert!(hungry > 10, "some consumer ate: {hungry}");
    // And no body is taken past the gate by more than one meal.
    let (held, _) = after(Limits { full: 12, ..big() }, 40);
    assert!(held < 12 + Rates::A1.meal(true), "held {held}");
}

#[test]
fn births_stop_at_the_ceiling_and_resume_below_it() {
    // A ceiling of one where none stand: no consumer is born. Raised, they are.
    let (_, none) = after(Limits { ceiling: [1, 1_000_000], ..big() }, 80);
    assert_eq!(none, 0, "no consumer is born where none may stand");
    let (_, some) = after(big(), 80);
    assert!(some > 0, "consumers are born under a ceiling they never reach");
}

#[test]
fn the_limits_compose_with_the_mass_law() {
    let mut f = founding(Some(Limits::default()));
    f.mass_law = Some(MassLaw::default());
    let mut plain = founding(None);
    plain.mass_law = Some(MassLaw::default());
    let (on, off) = (f.generate().unwrap(), plain.generate().unwrap());
    for i in (0..5).filter(|&i| consumer(i)) {
        for kind in ["feed", "birth"] {
            let id = format!("ecology:{kind}-{i}");
            let (p, q) = (&on.rules.processes[&id], &off.rules.processes[&id]);
            assert_eq!(added(p, q).len(), 1, "{id}");
            // The scaled meal, period and age stand as the mass law left them.
            assert_eq!((&p.effects, p.period), (&q.effects, q.period), "{id}");
        }
    }
    assert!(on.founding.as_ref().unwrap().mass_law.as_ref().unwrap().reference.is_some());
}

#[test]
fn the_limits_compose_with_flat_dormancy() {
    let mut f = founding(Some(Limits::default()));
    f.dormancy = Some(Dormancy::flat());
    let mut plain = founding(None);
    plain.dormancy = Some(Dormancy::flat());
    let (on, off) = (f.generate().unwrap(), plain.generate().unwrap());
    assert!(on.rules.processes.keys().any(|k| k.starts_with("ecology:wake-")), "seeds wake");
    assert_eq!(on.rules.processes.keys().collect::<Vec<_>>(), off.rules.processes.keys().collect::<Vec<_>>());
    for i in (0..5).filter(|&i| consumer(i)) {
        for kind in ["feed", "birth"] {
            let id = format!("ecology:{kind}-{i}");
            assert_eq!(added(&on.rules.processes[&id], &off.rules.processes[&id]).len(), 1, "{id}");
        }
    }
    // The producers' dormant processes are the same, and gain nothing.
    for (id, p) in &on.rules.processes {
        if id.contains("dormant") || id.contains("wake") {
            assert_eq!(p, &off.rules.processes[id], "{id}");
        }
    }
}

#[test]
fn the_limits_name_what_they_cannot_found() {
    let says = |f: Founding, what: &str| {
        let e = f.generate().err().unwrap_or_else(|| panic!("founded: {what}"));
        assert!(e.to_string().contains("limits"), "{what}: {e}");
    };
    says(Founding { bodies: None, ..founding(Some(Limits::default())) }, "no bodies");
    says(Founding { ecology: false, ..founding(Some(Limits::default())) }, "no ecology");
    says(Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Limits::default())) }, "world time");
    says(founding(Some(Limits { full: 0, ..Limits::default() })), "no fullness");
    says(founding(Some(Limits { ceiling: [0, 1], ..Limits::default() })), "no ceiling");
    says(founding(Some(Limits { ceiling: [3, 0], ..Limits::default() })), "no divisor");
}
