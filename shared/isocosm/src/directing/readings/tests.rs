// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::directing::{Aim, Toward, found::Played, tests::played};
use crate::{Execution, Founding, Session, history::Command};
use std::collections::BTreeMap;

fn world(seed: u64) -> Session {
    regional(seed, 2)
}

fn regional(seed: u64, region_sites: u32) -> Session {
    let founding = Founding {
        seed,
        sites: 6,
        population: 72,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        played: Some(Played {
            lineage: 1,
            region_sites,
        }),
        ..Default::default()
    };
    Session::new(founding.generate().unwrap(), Execution::Individuals).unwrap()
}

#[test]
fn regions_merge_into_one_as_biomass_falls() {
    for seed in 1..6 {
        let s = world(seed);
        let full = regions::site_biomass(&s.sim);
        let mut last = usize::MAX;
        for divisor in [1u128, 2, 4, 16, 1_000_000] {
            let scaled: BTreeMap<_, _> = full
                .iter()
                .map(|(site, levels)| {
                    let levels = levels
                        .iter()
                        .map(|(k, m)| (k.clone(), m / divisor))
                        .collect();
                    (*site, levels)
                })
                .collect();
            let n = regions::grow(&s.sim, &scaled).len();
            assert!(
                n <= last,
                "seed {seed}: regions grew from {last} to {n} as biomass fell"
            );
            last = n;
        }
        assert_eq!(
            last, 1,
            "seed {seed}: a connected world near no biomass is one region"
        );
        assert!(
            regions(&s.sim).len() > 1,
            "seed {seed}: the founded world has several regions"
        );
    }
}

#[test]
fn a_region_with_a_level_gone_has_collapsed() {
    // A region as wide as the world holds every level founded.
    let s = regional(3, 6);
    assert!(
        regions(&s.sim).iter().all(|r| !r.collapsed),
        "{:?}",
        regions(&s.sim)
    );
    // Planted fault: every producer gone.
    let gone: BTreeMap<_, _> = regions::site_biomass(&s.sim)
        .into_iter()
        .map(|(site, mut levels)| {
            levels.insert("life:producer".into(), 0);
            (site, levels)
        })
        .collect();
    assert!(regions::grow(&s.sim, &gone).iter().all(|r| r.collapsed));
}

#[test]
fn orders_grow_from_answered_nudges() {
    let mut s = world(5);
    let (p, c) = played(&mut s);
    let home = s.sim.state().population.get(c).unwrap().place;
    for _ in 0..6 {
        if !s.sim.state().population.get(c).unwrap().alive {
            break;
        }
        let nudge = Command::Nudge {
            participant: p,
            critter: c,
            aim: Aim::Attend,
            toward: Toward::Site(home),
        };
        s.command(nudge).unwrap();
        s.advance(6).unwrap();
    }
    let o = orders(&s.sim, p, c);
    let answered = s
        .sim
        .state()
        .nudges
        .iter()
        .filter(|n| n.answer.is_some())
        .count();
    assert!(answered > 0, "staying put answers a nudge to attend here");
    assert!(o.range.contains_key(&home) || o.avoid.contains(&home));
    assert_eq!(o.home.is_some(), o.range.get(&home).is_some_and(|v| *v > 0));
}

#[test]
fn readings_read_the_same_from_a_replay() {
    let mut s = world(9);
    let (p, c) = played(&mut s);
    for round in 0..5u64 {
        if s.sim.state().population.get(c).is_some_and(|e| e.alive) {
            let toward = Toward::Site(round % 6);
            let nudge = Command::Nudge {
                participant: p,
                critter: c,
                aim: Aim::Act,
                toward,
            };
            s.command(nudge).unwrap();
        }
        s.advance(7).unwrap();
    }
    let json = serde_json::to_vec(&s.save()).unwrap();
    let r = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(suggestions(&s.sim, c), suggestions(&r.sim, c));
    assert_eq!(orders(&s.sim, p, c), orders(&r.sim, p, c));
    assert_eq!(regions(&s.sim), regions(&r.sim));
    for mode in [Mode::Survival, Mode::Creative] {
        assert_eq!(view(&s.sim, c, mode), view(&r.sim, c, mode));
    }
}

#[test]
fn survival_shows_no_more_than_creative() {
    let mut s = world(4);
    let (_, c) = played(&mut s);
    s.advance(20).unwrap();
    let truth = view(&s.sim, c, Mode::Creative);
    let known = view(&s.sim, c, Mode::Survival);
    let e = s.sim.state().population.get(c).unwrap();
    assert!(known.sites.contains(&e.place));
    assert!(known.sites.is_subset(&truth.sites) && known.events.is_subset(&truth.events));
    assert!(known.things.keys().all(|f| truth.shows(*f)));
    assert!(known.sites.len() < truth.sites.len() || e.visits.len() + 1 >= truth.sites.len());
    // Planted fault: a suggestion aimed at a stranger elsewhere is filtered.
    let away = truth.things.keys().copied().find(|f| !known.shows(*f));
    if let Some(stranger) = away {
        let planted = Suggestion {
            process: "test:any".into(),
            target: Some(stranger),
            score: 0,
        };
        assert!(known.filter(vec![planted]).is_empty());
    }
}
