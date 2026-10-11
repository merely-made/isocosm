// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A part cap precedes whole-part eligibility, preserving the native crossing.
use super::*;
use isocosm::{flows::Flow, simulation::Work};

fn coincident(asks: [u64; 2], stock: [u64; 2]) -> (Genesis, [Id; 2], Id) {
    let (mut g, grazers, prey) = pass_world(0, asks);
    let trait_key = "test:part-grazer";
    g.rules.traits.insert(trait_key.into());
    g.rules.processes.get_mut("test:graze").unwrap().requires.push(Query::Trait {
        who: Binding::Actor, key: trait_key.into(),
    });
    for id in grazers {
        g.population.lift(id).unwrap().traits.insert(trait_key.into());
    }
    for (id, held) in stock.into_iter().enumerate() {
        g.population.lift(prey).unwrap().parts.get_mut(&PartId(id as u32)).unwrap()
            .matter = BTreeMap::from([("tissue:0".into(), held)]);
    }
    let of = ["tissue:0".to_string()];
    let e = g.population.get(prey).unwrap();
    g.seed = (0..128).find(|seed| {
        [0, 1].into_iter().all(|act| {
            anatomy::bitten(e, &of, isocosm::draw(*seed, "bite", &[act])) == Some(PartId(1))
        })
    }).expect("a bounded fixture seed draws the same leaf twice");
    (g, grazers, prey)
}

fn both(g: Genesis, grazers: [Id; 2], prey: Id, inspect: impl Fn(&Session, &[Flow])) {
    let mut hashes = vec![];
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.sim.watch("test:graze");
        let before = books(&s);
        let matter = s.sim.matter();
        let run = s.advance_tick_with_flows().unwrap();
        let Work { accepted, blocked, .. } = run.result;
        assert_eq!((accepted, blocked), (2, 0), "both scheduled receipts are accepted");
        let watched = s.sim.take_watched();
        assert_eq!(watched.iter().map(|w| (w.actor, w.target)).collect::<Vec<_>>(),
            grazers.map(|id| (id, Some(prey))).to_vec(), "accepted watched receipts bind the prey");
        let after = books(&s);
        let mut claimed = Books::new();
        for f in &run.flows {
            assert_eq!(f.count, 1, "the fixture keeps each physical member separate");
            for (h, key, sign) in [(f.from.0, &f.from.1, -1), (f.to.0, &f.to.1, 1)] {
                *claimed.entry((h, key.clone())).or_default() += sign * i128::from(f.amount);
            }
        }
        let keys: BTreeSet<_> = before.keys().chain(after.keys()).chain(claimed.keys()).collect();
        for k in keys {
            let held = |b: &Books| b.get(k).copied().unwrap_or(0);
            assert_eq!(held(&after) - held(&before), held(&claimed), "whole-part flow {k:?}");
        }
        assert_eq!(s.sim.matter(), matter);
        inspect(&s, &run.flows);
        let saved = s.save();
        hashes.push(saved.state_hash);
        for replay in [Execution::Individuals, Execution::Grouped] {
            assert_eq!(Session::load(saved.clone(), replay).unwrap().save().state_hash, saved.state_hash);
        }
    }
    assert_eq!(hashes[0], hashes[1], "both execution modes preserve the same incorporation boundary");
}

fn foreign_parts(s: &Session, id: Id) -> Vec<&Part> {
    s.sim.state().population.get(id).unwrap().parts.values()
        .filter(|p| p.matter.get("tissue:0").is_some_and(|v| *v > 0)).collect()
}

#[test]
fn coincident_whole_bites_are_capped_to_partial_meals_before_incorporation() {
    let (g, grazers, prey) = coincident([10, 3], [3, 10]);
    both(g, grazers, prey, |s, flows| {
        for (id, expected) in grazers.into_iter().zip([7, 2]) {
            assert_eq!(anatomy::held(s.sim.state().population.get(id).unwrap(),
                &s.sim.genesis().rules, "tissue:0"), expected, "each bite receives its part fraction");
            assert!(foreign_parts(s, id).is_empty(), "a partial bite cannot incorporate the leaf");
        }
        let donor = s.sim.state().population.get(prey).unwrap();
        assert_eq!(donor.parts.len(), 2);
        assert_eq!(donor.parts[&PartId(1)].matter["tissue:0"], 1);
        assert!(donor.alive);
        assert_eq!(flows.iter().map(|f| f.amount).sum::<u64>(), 9);
    });
}

#[test]
fn a_lone_full_part_quota_still_incorporates_with_a_coincident_zero_bite() {
    let (g, grazers, prey) = coincident([10, 0], [3, 10]);
    both(g, grazers, prey, |s, flows| {
        let moved = foreign_parts(s, grazers[0]);
        assert_eq!(moved.len(), 1, "the unchanged native incorporation path lands the leaf");
        assert_eq!(moved[0].matter["tissue:0"], 10);
        assert_eq!(moved[0].cells["function:fix"], 3, "a native frond retains its cells");
        assert!(foreign_parts(s, grazers[1]).is_empty());
        let donor = s.sim.state().population.get(prey).unwrap();
        assert_eq!(donor.parts.len(), 1);
        assert!(!donor.parts.contains_key(&PartId(1)) && donor.alive);
        assert!(s.sim.state().lineages["lineage:1"].development.as_ref().unwrap()
            .lexicon.contains("kind:frond"));
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].from, (Holder::Part(prey, PartId(1)), "tissue:0".into()));
        assert_eq!(flows[0].to.0.body(), Some(grazers[0]));
        assert!(matches!(flows[0].to.0, Holder::Part(..)));
        assert_eq!(flows[0].amount, 10);
    });
}

#[test]
fn zero_part_grants_are_accepted_without_flow_or_incorporation() {
    let (g, grazers, prey) = coincident([1, 1], [1, 1]);
    let original = g.population.get(prey).unwrap().clone();
    both(g, grazers, prey, |s, flows| {
        assert!(flows.is_empty(), "accepted zero-grant receipts move no matter");
        assert_eq!(s.sim.state().population.get(prey).unwrap(), &original,
            "a zero grant leaves the donor's parts and geometry unchanged");
        assert!(grazers.into_iter().all(|id| foreign_parts(s, id).is_empty()));
    });
}

#[test]
fn a_full_part_cap_preserves_the_existing_refused_crossing_as_a_meal() {
    let (mut g, grazers, prey) = coincident([10, 0], [3, 10]);
    g.lineages.get_mut("lineage:0").unwrap().development.as_mut().unwrap().domain = 1;
    both(g, grazers, prey, |s, flows| {
        assert!(foreign_parts(s, grazers[0]).is_empty(), "the cap cannot bypass affinity refusal");
        assert_eq!(anatomy::held(s.sim.state().population.get(grazers[0]).unwrap(),
            &s.sim.genesis().rules, "tissue:0"), 10);
        let donor = s.sim.state().population.get(prey).unwrap();
        assert_eq!(donor.parts.len(), 2, "a refused crossing consumes tissue without taking the part");
        assert_eq!(donor.parts[&PartId(1)].matter.get("tissue:0").copied().unwrap_or(0), 0);
        assert!(!s.sim.state().lineages["lineage:1"].development.as_ref().unwrap()
            .lexicon.contains("kind:frond"));
        assert_eq!(flows.iter().map(|f| f.amount).sum::<u64>(), 10);
    });
}
