//! The weight per kind of act and the pending credit (cases 1051, 1058;
//! M3.6): what is stored, how it is read, faded and passed on, and when a
//! pending act is credited or marked down.

use super::mechanics::{critter, slow};
use super::*;
use crate::schema::{Learned, Pending};
use std::collections::BTreeMap;

const MOVE: &str = "move:test";
const FEED: &str = "ecology:feed-1";

fn acts(pairs: &[(&str, i32)]) -> BTreeMap<Key, i32> {
    pairs.iter().map(|&(k, w)| (k.into(), w)).collect()
}

fn pending(act: &str, since: Tick) -> Option<Pending> {
    Some(Pending { act: act.into(), since })
}

fn held(s: &Session, c: Id) -> Learned {
    s.sim.state().population.get(c).unwrap().learned.clone().unwrap_or_default()
}

/// A world at tick `at` in which the critter has just ended `act`, which
/// `served` it or `bettered` it as given.
fn ended(s: &mut Session, c: Id, at: Tick, act: &str, served: bool, bettered: bool) {
    s.sim.state.tick = at;
    s.sim.learn_from(c, 1, act, served, bettered);
}

fn hiring(horizon: Tick) -> Learning {
    Learning { horizon, ..slow() }
}

fn start(learning: Learning) -> (Session, Id) {
    let (_, s) = fed_seed(|seed| fixture(seed, Some(learning), 1000));
    let c = critter(&s);
    (s, c)
}

#[test]
fn nothing_held_is_none_and_the_new_fields_save_as_nothing() {
    let nothing = Learned::default();
    assert!(nothing.is_empty());
    let old = Learned { settled: 3, weights: BTreeMap::from([(1, 4)]), ..Default::default() };
    let json = serde_json::to_string(&old).unwrap();
    assert_eq!(json, r#"{"settled":3,"weights":{"1":4}}"#, "no acts and no pending: as it saved before");
    assert_eq!(serde_json::from_str::<Learned>(&json).unwrap(), old);
    let new = Learned { acts: acts(&[(MOVE, 4)]), pending: pending(MOVE, 2), ..old };
    assert_eq!(serde_json::from_str::<Learned>(&serde_json::to_string(&new).unwrap()).unwrap(), new);
    assert!(!new.is_empty() && !Learned { pending: pending(MOVE, 1), ..nothing }.is_empty());
    // And a member whose acts and weights fade away carries none.
    let (mut s, c) = start(Learning { span: 10, ..slow() });
    ended(&mut s, c, 1, MOVE, true, false);
    assert!(held(&s, c).pending.is_some());
    s.sim.state.population.lift(c).unwrap().learned = Some(Learned { settled: 0, acts: acts(&[(MOVE, 1)]), ..Default::default() });
    ended(&mut s, c, 100, FEED, false, false);
    assert_eq!(held(&s, c).acts, BTreeMap::new(), "faded to nought");
}

#[test]
fn an_acts_weight_adds_to_its_score_as_a_need_of_its_pressure_does() {
    let (mut s, c) = start(slow());
    let score = |s: &Session, act: &str| s.sim.consider(c, false).options[act].0;
    let before = score(&s, "ecology:upkeep-1");
    let k = Learned { settled: 0, acts: acts(&[("ecology:upkeep-1", 12)]), ..Default::default() };
    s.sim.state.population.lift(c).unwrap().learned = Some(k);
    // The strongest need weighs ten, so a weight of 64 is a thousand.
    assert_eq!(score(&s, "ecology:upkeep-1") - before, 12 * 1000 / 64);
    assert_eq!(score(&s, FEED), s.sim.consider(c, false).options[FEED].0, "the others are as they were");
    // Without the rule, it is not read.
    let (_, mut off) = fed_seed(|seed| fixture(seed, None, 1000));
    let o = critter(&off);
    let plain = off.sim.consider(o, false).options["ecology:upkeep-1"].0;
    off.sim.state.population.lift(o).unwrap().learned = Some(Learned { settled: 0, acts: acts(&[("ecology:upkeep-1", 12)]), ..Default::default() });
    assert_eq!(off.sim.consider(o, false).options["ecology:upkeep-1"].0, plain);
}

#[test]
fn a_better_act_steps_its_own_weight_and_a_neutral_one_none() {
    let (mut s, c) = start(slow());
    ended(&mut s, c, 1, FEED, true, true);
    assert_eq!(held(&s, c).acts, acts(&[(FEED, 4)]));
    assert_eq!(held(&s, c).pending, None);
    ended(&mut s, c, 2, FEED, true, true);
    assert_eq!(held(&s, c).acts, acts(&[(FEED, 8)]));
    // Served but no better, as a move is: nothing in its weight yet.
    ended(&mut s, c, 3, MOVE, true, false);
    assert_eq!(held(&s, c).acts, acts(&[(FEED, 8)]));
    assert_eq!(held(&s, c).pending, pending(MOVE, 3));
}

#[test]
fn a_served_act_inside_the_horizon_credits_the_act_pending() {
    let (mut s, c) = start(hiring(12));
    ended(&mut s, c, 10, MOVE, true, false);
    // Acts between that better nothing leave it be, even a costly one.
    ended(&mut s, c, 12, "ecology:upkeep-1", true, false);
    ended(&mut s, c, 13, "ecology:upkeep-1", false, false);
    assert_eq!(held(&s, c).pending, pending(MOVE, 10), "the first act that did not serve stays");
    // The last tick of the horizon is inside it.
    ended(&mut s, c, 22, FEED, true, true);
    let k = held(&s, c);
    assert_eq!(k.acts, acts(&[(FEED, 4), (MOVE, 4)]), "the meal and the move are both credited");
    assert_eq!(k.pending, None);
    // Wellbeing alone is not credit: nothing better happened to it.
    ended(&mut s, c, 30, MOVE, true, false);
    ended(&mut s, c, 31, "ecology:upkeep-1", true, false);
    assert_eq!(held(&s, c).acts, k.acts);
}

#[test]
fn a_pending_act_settles_as_failed_once_the_horizon_has_passed() {
    let (mut s, c) = start(hiring(12));
    ended(&mut s, c, 10, MOVE, true, false);
    let (hash, kept) = (s.sim.state_hash(), held(&s, c));
    let read = |s: &mut Session, t: Tick| {
        s.sim.state.tick = t;
        crate::directing::readings::learned(&s.sim, c).unwrap()
    };
    let inside = read(&mut s, 22);
    assert_eq!((inside.acts.len(), inside.pending.clone()), (0, Some((MOVE.into(), 10))));
    let after = read(&mut s, 23);
    assert_eq!(after.acts, vec![(MOVE.to_string(), -4)], "marked down by the step at the next read");
    assert_eq!(after.pending, None);
    // Reading wrote nothing, and read early again it is as it was.
    assert_eq!(held(&s, c), kept);
    assert_eq!(read(&mut s, 22).pending, inside.pending);
    s.sim.state.tick = 10;
    assert_eq!(s.sim.state_hash(), hash);
    // The next learning step settles it, and a better act after the horizon
    // credits only itself.
    ended(&mut s, c, 30, FEED, true, true);
    let k = held(&s, c);
    assert_eq!((k.acts, k.pending), (acts(&[(FEED, 4), (MOVE, -4)]), None));
    // A late failure steps the weight down toward its bound.
    ended(&mut s, c, 40, MOVE, true, false);
    ended(&mut s, c, 60, FEED, true, true);
    assert_eq!(held(&s, c).acts[MOVE], -8);
}

#[test]
fn a_young_member_is_marked_down_by_youths_step() {
    let learn = Learning { bound: 32, span: 1_000_000, ..Learning::default() };
    let gate = |g: &mut crate::simulation::Genesis| {
        g.rules.processes.get_mut("ecology:birth-1").unwrap().requires.push(crate::rules::Query::Age { at_least: 1_000_000 });
    };
    let (_, mut s) = fed_seed(|seed| build(seed, Setup::new(Some(learn), 1000), gate));
    let c = critter(&s);
    ended(&mut s, c, 1, MOVE, true, false);
    ended(&mut s, c, 40, FEED, false, false);
    assert_eq!(held(&s, c).acts, acts(&[(MOVE, -8)]), "twice a step, in youth");
}

#[test]
fn a_horizon_of_nought_keeps_no_pending_and_credits_only_acts_that_serve_at_once() {
    let (mut s, c) = start(hiring(0));
    ended(&mut s, c, 1, MOVE, true, false);
    ended(&mut s, c, 2, "ecology:upkeep-1", false, false);
    assert_eq!(held(&s, c), Learned::default(), "no pending, no weight");
    ended(&mut s, c, 3, FEED, true, true);
    assert_eq!(held(&s, c).acts, acts(&[(FEED, 4)]));
    assert_eq!(held(&s, c).pending, None);
    // Over a run, no member ever holds a pending act, and no act weighs
    // below nought, since only acts that better a member move one.
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut run = build(3, Setup { mode, ..Setup::new(Some(hiring(0)), 20) }, |_| {});
        run.advance(60).unwrap();
        let st = run.sim.state();
        let ks: Vec<_> = st.population.groups.values().filter_map(|g| g.entity.learned.as_ref()).collect();
        assert!(!ks.is_empty(), "{mode:?}: someone learned");
        assert!(ks.iter().all(|k| k.pending.is_none() && k.acts.values().all(|&w| w > 0)), "{mode:?}");
    }
}

#[test]
fn fading_and_birth_carry_the_act_weights_and_leave_a_pending_act_behind() {
    let l = Learning { span: 100, ..Learning::default() };
    let k = Learned { settled: 0, weights: BTreeMap::from([(1, 8)]), acts: acts(&[(MOVE, 32), (FEED, 1)]), pending: pending(MOVE, 5) };
    let faded = l.faded(&k, 199);
    assert_eq!(faded.settled, 100);
    assert_eq!(faded.acts, acts(&[(MOVE, 24)]), "a quarter lost, and a weight of one gone");
    assert_eq!(faded.weights, BTreeMap::from([(1, 6)]));
    assert_eq!(faded.pending, k.pending, "waiting does not fade");
    let child = l.handed(&k, 0).unwrap();
    assert_eq!(child.acts, acts(&[(MOVE, 10)]), "a third of 32, a third of 1 gone");
    assert_eq!(child.pending, None, "a child does not wait on its parent's act");
    assert_eq!(child.weights, BTreeMap::from([(1, 2)]));
    let only = Learned { settled: 0, pending: pending(MOVE, 1), ..Default::default() };
    assert_eq!(l.handed(&only, 0), None);
    let acts_only = Learned { settled: 0, acts: acts(&[(MOVE, 9)]), ..Default::default() };
    assert_eq!(l.handed(&acts_only, 0).map(|k| k.acts), Some(acts(&[(MOVE, 3)])));
}

#[test]
fn a_run_with_pending_acts_saves_and_replays_to_the_same_hash_in_both_modes() {
    let travelling = |g: &mut crate::simulation::Genesis| {
        for p in g.rules.processes.iter_mut().filter(|(id, _)| id.starts_with("move:")).map(|(_, p)| p) {
            p.priority = 500;
            p.period = Some(3);
        }
    };
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = build(4, Setup { mode, ..Setup::new(Some(Learning { span: 50, ..Learning::default() }), 20) }, travelling);
        s.advance(40).unwrap();
        let waiting = |s: &Session| s.sim.state().population.groups.values().filter(|g| g.entity.learned.as_ref().is_some_and(|k| k.pending.is_some())).count();
        assert!(waiting(&s) > 0, "{mode:?}: someone had an act pending");
        let json = serde_json::to_vec(&s.save()).unwrap();
        for mut loaded in [Session::load_json(&json, mode).unwrap(), Session::load(s.save(), mode).unwrap()] {
            assert_eq!(loaded.sim.state_hash(), s.sim.state_hash(), "{mode:?}: reloaded");
            let mut again = s.clone();
            again.advance(40).unwrap();
            loaded.advance(40).unwrap();
            assert_eq!(loaded.sim.state_hash(), again.sim.state_hash(), "{mode:?}: re-advanced");
        }
    }
}
