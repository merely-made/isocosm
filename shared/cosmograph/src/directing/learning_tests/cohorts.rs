//! Cohorts, rows and replay under learning (cases 1052, 1058).

use super::mechanics::{critter, slow};
use super::*;
use crate::schema::Learned;
use std::collections::{BTreeMap, BTreeSet};

fn whole_row(s: &mut Session, first: Id, weight: i32) {
    let k = Learned { settled: 0, weights: BTreeMap::from([(HUNGER, weight)]) };
    s.sim.state.population.groups.get_mut(&first).unwrap().entity_mut().learned = Some(k);
}

fn weight(s: &Session, id: Id) -> Option<i32> {
    stored(s, id)
}

/// A world with a cohort of three alike members, `c + 1` to `c + 3`, each
/// holding `weight` on hunger and sharing a row.
fn cohort(weight: i32) -> (Session, Id) {
    let (_, mut s) = fed_seed(|seed| fixture(seed, Some(slow()), 20));
    let c = critter(&s);
    let row = s.sim.state().population.groups.get(&(c + 1)).expect("the played member's mates are a row");
    assert_eq!(row.count, 3);
    whole_row(&mut s, c + 1, weight);
    (s, c + 1)
}

#[test]
fn a_member_whose_delta_would_not_change_is_not_lifted() {
    let (mut s, first) = cohort(12);
    let before = s.sim.state().population.groups.clone();
    // Already at the bound of 12: a served act moves nothing.
    s.sim.learn_step(first + 1, &[HUNGER as usize], 1, true);
    assert!(s.sim.state().population.groups == before, "no lift, no store");
    // A failed one moves it, so that member alone takes a row.
    s.sim.learn_step(first + 1, &[HUNGER as usize], 1, false);
    let rows = &s.sim.state().population.groups;
    assert_eq!(rows.len(), before.len() + 2, "a row of three split about its middle");
    assert_eq!([weight(&s, first), weight(&s, first + 1), weight(&s, first + 2)], [Some(12), Some(8), Some(12)]);
}

#[test]
fn a_cohort_acting_as_one_moves_all_its_members_in_one_row() {
    let (mut s, first) = cohort(12);
    let rows = s.sim.state().population.groups.len();
    s.sim.learn_step(first, &[HUNGER as usize], 3, false);
    assert_eq!(s.sim.state().population.groups.len(), rows);
    assert_eq!([weight(&s, first), weight(&s, first + 1), weight(&s, first + 2)], [Some(8); 3]);
}

#[test]
fn members_with_equal_histories_stay_in_one_row() {
    let (mut s, first) = cohort(12);
    let rows = s.sim.state().population.groups.len();
    for member in [first, first + 1] {
        s.sim.learn_step(member, &[HUNGER as usize], 1, false);
    }
    assert!(s.sim.state().population.groups.len() > rows, "each took a row as it learned");
    // As a Grouped advance ends, alike neighbours fold into one row again.
    s.sim.state.population.restrict(&BTreeSet::new());
    assert_eq!(s.sim.state().population.groups[&first].count, 2, "the two that learned alike");
    assert_eq!(s.sim.state().population.groups.len(), rows + 1, "the third, unlearned, is its own");
}

/// The rows of a short run, with the rule on and off.
fn rows(learning: Option<Learning>, seed: u64, mode: Execution) -> usize {
    let mut s = build(seed, Setup { mode, ..Setup::new(learning, 20) }, |_| {});
    s.advance(60).unwrap();
    let learners = s.sim.state().population.groups.values().any(|g| g.entity.learned.is_some());
    assert_eq!(learners, learning.is_some(), "members learned only where the rule is set");
    s.sim.state().population.groups.len()
}

/// In this world each member that acts is already lifted into a row of its
/// own by its act, so learning adds none: 77, 119 and 87 rows after sixty
/// ticks from 18 under Individuals, and 66, 118 and 86 under Grouped, with
/// the rule on or off.
#[test]
fn learning_adds_no_rows_where_members_already_act_alone() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for seed in [1, 3, 5] {
            assert_eq!(rows(Some(Learning::default()), seed, mode), rows(None, seed, mode), "{mode:?} seed {seed}");
        }
    }
}

#[test]
fn a_learning_run_saves_and_replays_to_the_same_hash_in_both_modes() {
    let learn = Learning { span: 50, ..Learning::default() };
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = build(5, Setup { mode, ..Setup::new(Some(learn), 20) }, |_| {});
        s.advance(40).unwrap();
        let learned = |s: &Session| -> BTreeMap<Id, Learned> {
            let st = s.sim.state();
            st.population.groups.iter().filter_map(|(&id, g)| Some((id, g.entity.learned.clone()?))).collect()
        };
        assert!(!learned(&s).is_empty(), "{mode:?}: someone learned");
        let json = serde_json::to_vec(&s.save()).unwrap();
        for mut loaded in [Session::load_json(&json, mode).unwrap(), Session::load(s.save(), mode).unwrap()] {
            assert_eq!(loaded.sim.state_hash(), s.sim.state_hash(), "{mode:?}: reloaded");
            assert_eq!(learned(&loaded), learned(&s));
            let mut again = s.clone();
            again.advance(40).unwrap();
            loaded.advance(40).unwrap();
            assert_eq!(loaded.sim.state_hash(), again.sim.state_hash(), "{mode:?}: re-advanced");
        }
    }
}

#[test]
fn an_advance_refused_part_way_puts_what_was_learned_back() {
    let tight = |g: &mut crate::simulation::Genesis| g.rules.limits.events_per_advance = 150;
    let (_, mut s) = fed_seed(|seed| build(seed, Setup::new(Some(slow()), 20), tight));
    s.advance(3).unwrap();
    let (hash, learned) = (s.sim.state_hash(), s.sim.state().population.groups.values().filter(|g| g.entity.learned.is_some()).count());
    assert!(learned > 0, "someone learned in the first advance");
    // Long enough to learn some more before the budget is spent.
    assert!(s.advance(30).is_err(), "the budget refuses the advance");
    assert_eq!(s.sim.state_hash(), hash, "and what it learned goes back with it");
}
