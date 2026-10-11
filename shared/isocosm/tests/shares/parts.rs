// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Whole-prey quotas stay intact; coincident bites share their chosen part.
use super::*;
use isocosm::{Session, anatomy, flows::Holder, geometry::{Body, Sketch}};

fn fixture(stock: [u64; 2], wants: [u64; 2], same_part: bool) -> Genesis {
    let ledgers = [[(WANT, wants[0])], [(WANT, wants[1])]];
    let members = [
        ("lineage:0", &ledgers[0][..]),
        ("lineage:0", &ledgers[1][..]),
        ("lineage:1", &[][..]),
    ];
    let mut p = hunt(1);
    if let Effect::Eat { amount, .. } = &mut p.effects[0] {
        *amount = Amount::Computed(Expr::Read(Reading::Account {
            who: Binding::Actor,
            key: WANT.into(),
        }));
    }
    let mut g = world(0, &members, vec![p]);
    g.rules.shapes = default_shapes();
    let prey = g.population.lift(3).unwrap();
    prey.embody(Body::sketch(stock.into_iter().enumerate().map(|(i, held)| (
        i as u32,
        Sketch {
            half_extent: [2, 2, 2],
            parent: (i > 0).then_some(0),
            part: Part { matter: BTreeMap::from([("test:a".into(), held)]), ..Default::default() },
            ..Default::default()
        },
    ))));
    let of = ["test:a".to_string()];
    // A bounded fixture search, then exact native draws in the real test.
    // Neither the production draw nor the actors' planned action IDs change.
    let seed = (0..128).find(|seed| {
        let a = anatomy::bitten(prey, &of, isocosm::draw(*seed, "bite", &[0]));
        let b = anatomy::bitten(prey, &of, isocosm::draw(*seed, "bite", &[1]));
        a == Some(PartId(0)) && b == Some(PartId(if same_part { 0 } else { 1 }))
    }).expect("the bounded fixture includes both draw outcomes");
    g.seed = seed;
    g.validate().expect("the native anatomical fixture is admitted");
    g
}

fn books(s: &Session) -> BTreeMap<(Holder, Key), i128> {
    let mut b = BTreeMap::new();
    let is_matter = |k: &Key| matches!(s.sim.genesis().rules.accounts.get(k), Some(AccountKind::Matter { .. }));
    for (&first, group) in &s.sim.state().population.groups {
        for actor in first..first + group.count {
            for (key, value) in &group.entity.accounts {
                if is_matter(key) && *value > 0 {
                    b.insert((Holder::Entity(actor), key.clone()), i128::from(*value));
                }
            }
            for (&part, p) in &group.entity.parts {
                for (key, value) in &p.matter {
                    if is_matter(key) && *value > 0 {
                        b.insert((Holder::Part(actor, part), key.clone()), i128::from(*value));
                    }
                }
            }
        }
    }
    for (&site, p) in &s.sim.state().sites {
        for (key, value) in &p.accounts {
            if is_matter(key) && *value > 0 {
                b.insert((Holder::Site(site), key.clone()), i128::from(*value));
            }
        }
    }
    b
}

fn check(g: Genesis, want: [u64; 2], remaining: u64) {
    let mut hashes = vec![];
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.sim.watch("test:hunt");
        let matter = s.sim.matter();
        let before = books(&s);
        let run = s.advance_tick_with_flows().unwrap();
        let after = books(&s);
        assert_eq!((run.result.accepted, run.result.blocked), (2, 0),
            "a retained zero grant is accepted rather than missing from the plan");
        if want == [0, 0] {
            assert!(run.flows.is_empty(), "accepted zero grants have no matter flows");
            assert_eq!(s.sim.state().population.get(3), g.population.get(3),
                "zero grants leave the donor's anatomy unchanged");
        }
        let mut claimed = BTreeMap::<(Holder, Key), i128>::new();
        for flow in &run.flows {
            for ((holder, key), sign) in [(&flow.from, -1), (&flow.to, 1)] {
                let mut add = |holder, count: u64| {
                    *claimed.entry((holder, key.clone())).or_default() +=
                        sign * i128::from(flow.amount) * i128::from(count);
                };
                match *holder {
                    Holder::Entity(first) => (first..first + flow.count).for_each(|id| add(Holder::Entity(id), 1)),
                    Holder::Part(first, p) => (first..first + flow.count).for_each(|id| add(Holder::Part(id, p), 1)),
                    _ => add(*holder, flow.count),
                }
            }
        }
        let keys: BTreeSet<_> = before.keys().chain(after.keys()).chain(claimed.keys()).collect();
        for key in keys {
            let held = |b: &BTreeMap<(Holder, Key), i128>| b.get(key).copied().unwrap_or(0);
            assert_eq!(held(&after) - held(&before), held(&claimed), "flow holder/account {key:?}");
        }
        let watched = s.sim.take_watched();
        assert_eq!(watched.len(), 2, "both bites land without an identity-order refusal");
        assert_eq!(watched.iter().map(|w| (w.actor, w.target)).collect::<Vec<_>>(), vec![(1, Some(3)), (2, Some(3))]);
        for (actor, wanted) in (1..=2).zip(want) {
            assert_eq!(s.sim.state().population.get(actor).unwrap().accounts.get(GUT).copied().unwrap_or(0), wanted,
                "each granted bite is constrained before either actor writes");
        }
        assert_eq!(anatomy::held(s.sim.state().population.get(3).unwrap(), &g.rules, "test:a"), remaining,
            "ungranted matter stays in the chosen prey");
        assert_eq!(s.sim.matter(), matter);
        let saved = s.save();
        hashes.push(saved.state_hash);
        for replay in [Execution::Individuals, Execution::Grouped] {
            assert_eq!(Session::load(saved.clone(), replay).unwrap().save().state_hash, saved.state_hash);
        }
    }
    assert_eq!(hashes[0], hashes[1], "the two modes land the same anatomical shares");
}

#[test]
fn coincident_one_unit_parts_floor_both_bites_without_first_actor_winning() {
    check(fixture([1, 1], [3, 3], true), [0, 0], 2);
}

#[test]
fn separate_parts_preserve_the_whole_prey_grants() {
    check(fixture([1, 1], [3, 3], false), [1, 1], 0);
}

#[test]
fn a_part_cap_never_replaces_the_whole_prey_fraction() {
    check(fixture([1, 1], [3, 1], false), [1, 0], 1);
}

#[test]
fn an_oversubscribed_bitten_part_gives_each_granted_bite_the_same_fraction() {
    check(fixture([2, 5], [4, 4], true), [1, 1], 5);
}
