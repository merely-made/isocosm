// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The edible matter reading on the authored flat world (a feed with no
//! meal contract, which reads what its plain `Eat` can take), its memo, its
//! demand helper and its query. The bodied and meal-contract worlds are read
//! in `tests/world_time_food_contract/edible.rs`.

use super::{tests::{fixture, start}, *};
use crate::{Execution, simulation::Genesis};

const FEED: &str = "ecology:feed-1";

#[test]
fn edible_matter_is_the_hand_sum_of_what_the_plain_eat_can_take() {
    // Five live prey of 10 at site 0, three at site 1, two dead at site 0.
    for (place, want) in [(0, 50), (1, 30)] {
        for mode in [Execution::Individuals, Execution::Grouped] {
            let (g, actor, ..) = fixture(3, place);
            let mut s = Simulation::new(g, mode).unwrap();
            start(&mut s);
            assert_eq!(s.edible_matter(actor, FEED).unwrap(), want, "site {place}");
            s.targets = None;
            assert_eq!(s.edible_matter(actor, FEED).unwrap(), want, "without the index");
        }
    }
}

#[test]
fn many_heads_with_little_matter_read_low() {
    let (mut g, actor, first, _) = fixture(3, 0);
    let prey = g.population.groups.get_mut(&first).unwrap();
    std::sync::Arc::make_mut(&mut prey.entity).accounts = Ledger::from([("matter:0-0".into(), 1)]);
    let mut s = Simulation::new(g, Execution::Individuals).unwrap();
    start(&mut s);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), 5, "five heads of one each");
}

#[test]
fn the_actor_is_not_its_own_food_and_dormant_holders_are_not_food() {
    let (mut g, actor, first, _) = fixture(3, 0);
    // Let the grazer eat its own lineage too: its own member is left out.
    g.rules.processes.get_mut(FEED).unwrap().target.as_mut().unwrap().among.insert("lineage:1".into());
    let mine = g.population.groups.get_mut(&actor).unwrap();
    std::sync::Arc::make_mut(&mut mine.entity).accounts = Ledger::from([("matter:1-0".into(), 7)]);
    let mut s = Simulation::new(g.clone(), Execution::Individuals).unwrap();
    start(&mut s);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), 50, "its own member and a lone group of one add nothing");
    // A second member of the grazer's group would be food: the count is whole, less one.
    let mut more = g.clone();
    let groups: Vec<_> = more.population.groups.iter().map(|(&id, c)| (id, c.entity.as_ref().clone(), c.count)).collect();
    more.population = crate::population::Population::default();
    let mut actor = actor;
    for (id, entity, count) in groups {
        let made = more.population.insert(entity, if id == actor { 3 } else { count }).unwrap();
        if id == actor { actor = made; }
    }
    let mut s = Simulation::new(more, Execution::Individuals).unwrap();
    start(&mut s);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), 50 + 2 * 7);

    // Dormant prey are passed by: all five are, so nothing is left (the grazer's own group is one).
    let actor = fixture(3, 0).1;
    g.rules.dormancy = Some(crate::rules::Dormancy::default());
    g.rules.traits.insert("stage:dormant".into());
    let prey = g.population.groups.get_mut(&first).unwrap();
    std::sync::Arc::make_mut(&mut prey.entity).traits.insert("stage:dormant".into());
    let mut s = Simulation::new(g, Execution::Individuals).unwrap();
    start(&mut s);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), 0, "the five dormant are passed by, the lone grazer is only itself");
}

#[test]
fn the_memo_equals_direct_and_does_not_outlive_its_pass() {
    let (g, actor, ..) = fixture(5, 0);
    let mut s = Simulation::new(g, Execution::Individuals).unwrap();
    start(&mut s);
    let direct = {
        let frame = s.frame.take();
        let v = s.edible_matter(actor, FEED).unwrap();
        s.frame = frame;
        v
    };
    assert_eq!(s.frame.as_ref().unwrap().edible_len(), 0);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), direct);
    assert_eq!(s.edible_matter(actor, FEED).unwrap(), direct, "a second read is the first");
    assert_eq!(s.frame.as_ref().unwrap().edible_len(), 1);
    assert_eq!(s.frame.as_ref().unwrap().clone().edible_len(), 0, "a clone starts empty");
    start(&mut s);
    assert_eq!(s.frame.as_ref().unwrap().edible_len(), 0, "the next pass starts empty");
}

#[test]
fn a_meal_is_the_effects_amount_times_the_lineages_pace() {
    let (g, ..) = fixture(3, 0);
    let feed = &g.rules.processes[FEED];
    assert_eq!(feed.meal_demand(None).unwrap(), 1);
    assert_eq!(feed.meal_demand(Some(2)).unwrap(), 2);
    let mut transition = feed.clone();
    transition.causation = Causation::Transition;
    assert_eq!(transition.meal_demand(Some(2)).unwrap(), 1, "only a choice is paced");
    transition.effects.clear();
    assert!(transition.meal_demand(None).is_err());
}

#[test]
fn the_query_compares_matter_with_heads() {
    let (mut g, actor, ..) = fixture(3, 0);
    let gate = |at_least, per_members| Query::EdiblePerHead { feed: FEED.into(), at_least, per_members };
    g.rules.processes.get_mut(FEED).unwrap().requires.push(gate(50, 1));
    g.validate().unwrap();
    let mut s = Simulation::new(g, Execution::Individuals).unwrap();
    start(&mut s);
    let place = s.body_at_start(actor).unwrap().place;
    let asked = |s: &Simulation, q: &Query| s.query(actor, None, place, None, q);
    assert!(asked(&s, &gate(50, 1)).is_ok(), "50 matter to one head");
    assert!(asked(&s, &gate(51, 1)).is_err());
    assert!(asked(&s, &gate(100, 2)).is_ok(), "per two heads");
    assert!(asked(&s, &gate(1, 0)).is_err(), "no denominator");
    assert!(asked(&s, &Query::EdiblePerHead { feed: "no:feed".into(), at_least: 0, per_members: 1 }).is_err());
}

#[test]
fn the_query_names_a_positive_denominator_and_a_feed_that_eats() {
    let (mut g, ..) = fixture(3, 0);
    let bad = |g: &Genesis| g.validate().is_err();
    g.rules.processes.get_mut(FEED).unwrap().requires.push(Query::EdiblePerHead { feed: FEED.into(), at_least: 1, per_members: 0 });
    assert!(bad(&g), "a nought denominator");
    let (mut g, ..) = fixture(3, 0);
    g.rules.processes.get_mut(FEED).unwrap().requires.push(Query::EdiblePerHead { feed: "ecology:birth-1".into(), at_least: 1, per_members: 1 });
    assert!(bad(&g), "a birth is no feed");
}

#[test]
fn the_query_serializes_as_the_last_variant() {
    let q = Query::EdiblePerHead { feed: FEED.into(), at_least: 4, per_members: 1 };
    let json = serde_json::to_string(&q).unwrap();
    assert_eq!(serde_json::from_str::<Query>(&json).unwrap(), q);
}
