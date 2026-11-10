use super::*;
use crate::{
    directing::readings,
    rules::{Directing, Effect, Query},
    schema::Learned,
};
use std::collections::BTreeMap;

pub(super) fn critter(s: &Session) -> Id {
    super::super::tests::critter(s)
}

/// A learning that never fades within a short run, bounded at three steps.
pub(super) fn slow() -> Learning {
    Learning { bound: 12, span: 1_000_000, ..Learning::default() }
}

fn learned(weight: i32, settled: Tick) -> Option<Learned> {
    Some(Learned { settled, weights: BTreeMap::from([(HUNGER, weight)]) })
}

fn give(s: &mut Session, c: Id, k: Option<Learned>) {
    s.sim.state.population.lift(c).unwrap().learned = k;
}

#[test]
fn the_rule_is_validated_and_needs_the_policy_gate() {
    let mut d = Directing { learning: Some(Learning::default()), ..Directing::default() };
    assert!(d.validate().is_err(), "no policy gate");
    d.policy = GATE;
    assert!(d.validate().is_ok());
    for bad in [
        Learning { step: 0, ..Learning::default() },
        Learning { bound: 2, ..Learning::default() },
        Learning { youth: [1, 0], ..Learning::default() },
        Learning { span: 0, ..Learning::default() },
        Learning { fade: [3, 2], ..Learning::default() },
        Learning { pass: [1, 0], ..Learning::default() },
    ] {
        d.learning = Some(bad);
        assert!(d.validate().is_err(), "{bad:?}");
    }
    assert_eq!(Learning::default().pass, [1, 3], "case 1058: a third");
}

#[test]
fn unset_leaves_no_mention_and_no_difference() {
    let mut s = fixture(3, None, 20);
    s.advance(40).unwrap();
    let json = serde_json::to_string(&s.save()).unwrap();
    assert!(!json.contains("learn"), "learning leaked into a save with the rule unset");
    assert!(s.sim.state().population.groups.values().all(|g| g.entity.learned.is_none()));
    // Set but with no policy founded, nothing is learned and nothing is
    // different: the same state hash as the world that never had the rule.
    // The hash reads the rules too, so what is compared is the world.
    let world = |learning| {
        let mut s = build(3, Setup { found: false, ..Setup::new(learning, 20) }, |_| {});
        s.advance(40).unwrap();
        (s.sim.state().population.groups.clone(), s.sim.state().relations.clone())
    };
    assert!(world(None) == world(Some(Learning::default())));
}

#[test]
fn a_served_act_gains_weight_a_failed_one_loses_it_and_both_clamp() {
    let (_, mut s) = fed_seed(|seed| fixture(seed, Some(slow()), 20));
    let c = critter(&s);
    for want in [4, 8, 12, 12] {
        s.advance(3).unwrap();
        assert_eq!(stored(&s, c), Some(want));
    }
    // The same meal with a price above its worth: accepted, and it leaves
    // the critter poorer.
    let costly = |g: &mut crate::simulation::Genesis| {
        g.rules.processes.get_mut("ecology:feed-1").unwrap().effects.push(Effect::Transform {
            who: crate::rules::Binding::Actor,
            take: BTreeMap::from([("matter:1-0".into(), 3)]),
            give: BTreeMap::from([("world:soil".into(), 3)]),
            conversion: None,
        });
        g.rules.processes.get_mut("ecology:feed-1").unwrap().effects.push(Effect::Transfer {
            from: crate::rules::Binding::Actor,
            to: crate::rules::Binding::Place,
            account: "world:soil".into(),
            amount: crate::rules::Amount::Fixed(3),
        });
    };
    let (_, mut s) = fed_seed(|seed| build(seed, Setup::new(Some(slow()), 20), costly));
    let c = critter(&s);
    for want in [-4, -8, -12, -12] {
        s.advance(3).unwrap();
        assert_eq!(stored(&s, c), Some(want));
    }
}

#[test]
fn a_young_member_moves_youth_times_as_far_as_an_adult() {
    let learn = Learning { bound: 32, span: 1_000_000, ..Learning::default() };
    let after = |matures: u64| {
        let gate = move |g: &mut crate::simulation::Genesis| {
            g.rules.processes.get_mut("ecology:birth-1").unwrap().requires.push(Query::Age { at_least: matures });
        };
        let (_, mut s) = fed_seed(|seed| build(seed, Setup::new(Some(learn), 20), gate));
        let c = critter(&s);
        assert_eq!(super::super::learn::maturity(&s.sim.genesis().rules, s.sim.state().population.get(c).unwrap()), Some(matures));
        s.advance(6).unwrap();
        stored(&s, c).unwrap()
    };
    let (young, adult) = (after(1_000_000), after(1));
    assert_eq!(adult, 8);
    assert_eq!(young, adult * 2, "youth [2, 1]");
}

#[test]
fn a_delta_fades_by_whole_spans_and_reading_it_changes_nothing() {
    let learn = Learning { span: 100, ..Learning::default() };
    let (_, mut s) = fed_seed(|seed| fixture(seed, Some(learn), 20));
    let c = critter(&s);
    give(&mut s, c, learned(32, 0));
    s.sim.state.tick = 0;
    let (hash, kept) = (s.sim.state_hash(), s.sim.state().population.get(c).unwrap().learned.clone());
    let read = |s: &mut Session, t: Tick| {
        s.sim.state.tick = t;
        readings::learned(&s.sim, c).unwrap().weights
    };
    let mut want = 32;
    for k in 0..8u64 {
        assert_eq!(read(&mut s, k * 100 + 99), vec![(HUNGER as usize, want)], "{k} spans and a part");
        // Each span takes a quarter, the share's integer remainder kept off.
        want = want * 3 / 4;
        assert_eq!(read(&mut s, (k + 1) * 100), if want == 0 { vec![] } else { vec![(HUNGER as usize, want)] });
    }
    // Read late, then early again: the same, since nothing was written.
    assert_eq!(read(&mut s, 300), vec![(HUNGER as usize, 13)]);
    assert_eq!(read(&mut s, 100), vec![(HUNGER as usize, 24)]);
    s.sim.state.tick = 0;
    assert_eq!(s.sim.state_hash(), hash);
    assert_eq!(s.sim.state().population.get(c).unwrap().learned, kept);
}

#[test]
fn two_members_alike_but_for_their_deltas_choose_differently() {
    let (_, mut s) = fed_seed(|seed| fixture(seed, Some(slow()), 1000));
    let c = critter(&s);
    let d = c + 1;
    let (a, b) = (s.sim.state().population.get(c).unwrap(), s.sim.state().population.get(d).unwrap());
    assert!(b.alive && b.lineage == a.lineage && b.place == a.place);
    let act = |s: &Session, id| s.sim.consider(id, false).process.unwrap();
    assert_eq!((act(&s, c), act(&s, d)), ("ecology:feed-1".into(), "ecology:feed-1".into()));
    // Turned against feeding, one takes the act that competes with it.
    give(&mut s, c, learned(-12, 0));
    assert_eq!(act(&s, c), "ecology:upkeep-1");
    assert_eq!(act(&s, d), "ecology:feed-1");
    // The rule unset, a delta is not read.
    let (_, mut off) = fed_seed(|seed| fixture(seed, None, 1000));
    let o = critter(&off);
    give(&mut off, o, learned(-12, 0));
    assert_eq!(act(&off, o), "ecology:feed-1");
}

#[test]
fn a_child_takes_a_third_of_its_parents_delta_toward_nought() {
    let l = Learning::default();
    let parent = Learned { settled: 0, weights: BTreeMap::from([(0, -8), (1, 2), (2, 9)]) };
    let child = l.handed(&parent, 5).unwrap();
    assert_eq!(child.weights, BTreeMap::from([(0, -2), (2, 3)]), "-8/3 and 9/3, 2/3 gone");
    assert_eq!(child.settled, 5);
    assert_eq!(l.handed(&Learned { settled: 0, weights: BTreeMap::from([(0, 2)]) }, 1), None);

    let births = |g: &mut crate::simulation::Genesis| {
        for q in &mut g.rules.processes.get_mut("ecology:birth-1").unwrap().requires {
            if let Query::Account { at_least, .. } = q {
                *at_least = 5;
            }
        }
    };
    let (_, mut s) = fed_seed(|seed| build(seed, Setup::new(Some(slow()), 20), births));
    let known = s.sim.state().population.next_id;
    let mut checked = 0;
    for _ in 0..60 {
        s.advance(1).unwrap();
        let st = s.sim.state();
        for (&id, g) in st.population.groups.range(known..) {
            let e = &g.entity;
            let Some(parent) = st.relations.iter().find(|r| r.subject == id && r.kind == "sim:parent") else { continue };
            let Some(p) = st.population.get(parent.object).and_then(|p| p.learned.clone()) else { continue };
            if e.born != st.tick || e.lineage != "lineage:1" {
                continue;
            }
            assert_eq!(e.learned, l.handed(&p, e.born), "child {id}");
            assert!(e.learned.as_ref().is_none_or(|k| k.settled == e.born));
            checked += 1;
        }
    }
    assert!(checked > 0, "no child was born of a parent that had learned");
}
