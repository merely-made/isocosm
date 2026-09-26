// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{
    super::{Crowd, ProbeFounding, ProbeWorld, Variant, exact::run_ordered, readings},
    *,
};

const FOOD: &str = "world:food";
const WATER: &str = "world:water";

fn two(seed: u64) -> ProbeWorld {
    ProbeFounding {
        seed,
        water: true,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

/// Two identical contesters alone at one site, with one ration of food and
/// one of water between them and no regrowth: each competition is one fight,
/// which ends when the loser of the first exchange spends all it has.
fn duel(seed: u64) -> ProbeWorld {
    let mut w = ProbeFounding {
        seed,
        water: true,
        lineages: 1,
        sites: [1, 1],
        members: [2, 2],
        cohort: 2,
        ticks: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let rules = &mut w.genesis.rules;
    for c in rules.competitions.values_mut() {
        (c.margin, c.cost, c.upset) = (10, 10, 0);
        for k in &mut c.kinds {
            if let Query::Below { amount, .. } = &mut k.hungry {
                *amount = 100;
            }
        }
    }
    let rations = [FOOD, WATER].map(|k| rules.competitions[k].ration);
    let site = w.genesis.sites.get_mut(&0).unwrap();
    site.accounts.insert("world:soil".into(), 0);
    site.accounts.insert(FOOD.into(), rations[0]);
    site.accounts.insert(WATER.into(), rations[1]);
    let pair = w.genesis.population.groups.get_mut(&1).unwrap();
    assert_eq!(pair.count, 2);
    let e = &mut pair.entity;
    e.traits.remove("leaning:scramble");
    e.traits.insert("leaning:contest".into());
    e.accounts.insert("matter:0-0".into(), 4);
    e.accounts.insert("matter:0-1".into(), 1);
    w
}

use crate::{probe::readings::Probe, rules::Query};

#[test]
fn settlement_sums_every_competition_and_caps_the_reserve_at_what_is_held() {
    let w = two(3);
    let comps = w.competitions();
    let (food, water) = (&comps[FOOD], &comps[WATER]);
    let kind = &food.kinds[0];
    let mut member = w.genesis.population.get(1).unwrap().clone();
    member.accounts.insert(kind.body.clone(), 3);
    let takes = BTreeMap::from([
        (
            FOOD.to_string(),
            Take {
                rounds: 2,
                spent: 2,
                gain: 0,
            },
        ),
        (
            WATER.to_string(),
            Take {
                rounds: 1,
                spent: 2,
                gain: water.ration,
            },
        ),
    ]);
    let mut expected = vec![food.round.clone(), food.round.clone(), water.round.clone()];
    // Four units spent, three held: three paid.
    expected.extend(std::iter::repeat_n(kind.spend.clone(), 3));
    expected.push(water.kinds[0].eat.clone());
    assert_eq!(settlement(comps, &member, &takes), expected);
    // Nothing taken, nothing settled.
    assert!(settlement(comps, &member, &BTreeMap::new()).is_empty());
}

#[test]
fn competitions_settle_alike_whatever_order_they_run_in() {
    for seed in 0..4 {
        let w = ProbeFounding {
            seed,
            water: true,
            members: [16, 32],
            ticks: 8,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let keys: Vec<&Key> = w.competitions().keys().collect();
        let back: Vec<&Key> = keys.iter().rev().copied().collect();
        let exact = |order: &[&Key]| run_ordered(&w, 7, true, order).unwrap().sim.state_hash();
        assert_eq!(exact(&keys), exact(&back), "world {seed}");
        let forward = Crowd::new(&w, 7, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let reversed = Crowd::new(&w, 7, Variant::Histogram)
            .unwrap()
            .reversed()
            .run()
            .unwrap();
        assert_eq!(forward.bins, reversed.bins, "world {seed}");
    }
}

#[test]
fn a_member_fighting_twice_pays_both_costs_but_never_more_than_it_holds() {
    // After upkeep each holds 3 body and no water. Each fight's loser spends
    // its 3; the winner takes the ration. One losing both pays 6 against the
    // 3 it held, so it pays 3; the other eats on its untouched 3.
    let (mut twice, mut split) = (0, 0);
    for seed in 0..40 {
        let w = duel(seed);
        let (food, water) = (
            w.competitions()[FOOD].ration,
            w.competitions()[WATER].ration,
        );
        let run = run_ordered(&w, seed, true, &w.competitions().keys().collect::<Vec<_>>());
        let members = readings::exact_members(run.as_ref().unwrap());
        let mut held: Vec<(u64, u64)> = members
            .iter()
            .flat_map(|(e, n)| {
                let at = |k: &str| e.accounts.get(k).copied().unwrap_or(0);
                std::iter::repeat_n((at("matter:0-0"), at("matter:0-1")), *n as usize)
            })
            .collect();
        held.sort();
        if held == [(0, 0), (3 + food, water)] {
            twice += 1;
        } else {
            assert_eq!(held, [(0, water), (food, 0)], "seed {seed}");
            split += 1;
        }
        // The crowd settles the same two ways.
        let crowd = Crowd::new(&w, seed, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let mut bins: Vec<(u64, u64, u64)> = readings::crowd_members(&crowd)
            .iter()
            .map(|(e, n)| {
                let at = |k: &str| e.accounts.get(k).copied().unwrap_or(0);
                (at("matter:0-0"), at("matter:0-1"), *n)
            })
            .collect();
        bins.sort();
        let options = [
            vec![(0, 0, 1), (3 + food, water, 1)],
            vec![(0, water, 1), (food, 0, 1)],
        ];
        assert!(options.contains(&bins), "seed {seed}: {bins:?}");
    }
    assert!(twice > 0 && split > 0, "{twice} twice, {split} split");
}

#[test]
fn without_shortage_two_competitions_settle_alike_in_both_runners() {
    for seed in 0..4 {
        let mut founding = ProbeFounding {
            seed,
            water: true,
            ticks: 2,
            ..Default::default()
        };
        founding.regrowth = [5000, 5000];
        let w = founding.generate().unwrap();
        // Nothing is random without shortage but the one member an
        // inspection draws, so every other reading must agree exactly.
        let derived: Vec<_> = readings::derive(&w)
            .into_iter()
            .filter(|r| !matches!(r.probe, Probe::Inspect { .. }))
            .collect();
        let run = run_ordered(&w, 1, true, &w.competitions().keys().collect::<Vec<_>>()).unwrap();
        let members = readings::exact_members(&run);
        let s = run.sim.state();
        let exact = readings::evaluate(&derived, &w, &members, &s.sites, s.tick, None).unwrap();
        let crowd = Crowd::new(&w, 2, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let members = readings::crowd_members(&crowd);
        let fungible = readings::evaluate(&derived, &w, &members, &crowd.sites, crowd.tick, None);
        assert_eq!(exact, fungible.unwrap(), "world {seed}");
        assert!(derived.iter().any(|r| r.key.starts_with(WATER)));
    }
}
