// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Edible matter beside headcount: the same admission, summed by the matter
//! each exposed holder offers.

use super::*;

const FEED: &str = "ecology:feed-1";

fn edible_gate(g: &mut Genesis, at_least: u64, per_members: u64) -> String {
    let key = "test:edible-gate";
    g.rules.traits.insert("test:enough-edible".into());
    auxiliary(
        g,
        FEED,
        key,
        vec![Effect::Trait {
            who: Binding::Actor,
            key: "test:enough-edible".into(),
            present: true,
        }],
    );
    g.rules.processes.get_mut(key).unwrap().requires = vec![
        Query::Alive(Binding::Actor),
        Query::EdiblePerHead { feed: FEED.into(), at_least, per_members },
    ];
    key.into()
}

/// One grazer and `count` plants, each with `amount` in its exposed part.
fn meadow(amount: u64, count: u64) -> (Genesis, Id, Id) {
    let (mut g, eater, prey) = controlled(1, 0);
    stripped_plant(&mut g, prey, amount);
    let (animal, plant) = (
        g.population.get(eater).unwrap().clone(),
        g.population.get(prey).unwrap().clone(),
    );
    g.population = Population::default();
    let eater = g.population.insert(animal, 1).unwrap();
    let prey = g.population.insert(plant, count).unwrap();
    (g, eater, prey)
}

#[test]
fn the_reading_is_the_hand_sum_of_exposed_matter_and_heads_do_not_move_it() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for (amount, count) in [(3, 5), (1, 5), (3, 1), (2, 7)] {
            let (g, eater, _) = meadow(amount, count);
            let s = Session::new(g, mode).unwrap();
            assert_eq!(s.sim.available_food(eater, FEED).unwrap(), count);
            assert_eq!(s.sim.edible_matter(eater, FEED).unwrap(), amount * count, "{amount} x {count}");
        }
    }
}

#[test]
fn grazed_down_prey_pass_the_headcount_and_fail_the_matter() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, ..) = meadow(1, 5);
        let heads = food_gate(&mut g, FEED, 2, 1);
        let matter = edible_gate(&mut g, 10, 1);
        let (eater, _) = (member_id(&g, 1), ());
        let mut s = Session::new(g, mode).unwrap();
        assert!(act(&mut s, eater, None, &heads).accepted(), "five holders for one head");
        assert!(!act(&mut s, eater, None, &matter).accepted(), "five units for ten wanted");
        // The same meadow, ungrazed: fifteen units pass ten per head.
        let (mut g, eater, _) = meadow(3, 5);
        let matter = edible_gate(&mut g, 10, 1);
        let mut s = Session::new(g, mode).unwrap();
        assert!(act(&mut s, eater, None, &matter).accepted());
    }
}

#[test]
fn self_is_excluded_and_the_reading_follows_the_cohort() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut per = 0;
        for count in [1, 2, 3, 5] {
            let (mut g, eater, _) = controlled(3, 8);
            put_ingesta(&mut g, eater, 3, 2);
            let animal = g.population.get(eater).unwrap().clone();
            g.population = Population::default();
            let eater = g.population.insert(animal, count).unwrap();
            let s = Session::new(g, mode).unwrap();
            for member in eater..eater + count {
                let got = s.sim.edible_matter(member, "ecology:feed-3").unwrap();
                if count == 2 {
                    per = got;
                }
                assert_eq!(got, per * (count - 1).min(count), "{count}");
            }
            if count == 1 {
                assert_eq!(s.sim.edible_matter(eater, "ecology:feed-3").unwrap(), 0);
            }
        }
        assert!(per > 0, "a predator's tissue is food for its own kind");
    }
}

#[test]
fn dormant_holders_are_not_edible() {
    let (mut g, eater, prey) = meadow(3, 4);
    g.rules.dormancy = Some(cosmograph::rules::Dormancy::default());
    g.rules.traits.insert("stage:dormant".into());
    let awake = Session::new(g.clone(), Execution::Individuals).unwrap();
    assert_eq!(awake.sim.edible_matter(eater, FEED).unwrap(), 12);
    let mut one = g.clone();
    one.population.lift(prey).unwrap().traits.insert("stage:dormant".into());
    let one = Session::new(one, Execution::Individuals).unwrap();
    assert_eq!(one.sim.edible_matter(eater, FEED).unwrap(), 9, "one of four sleeps");
    let sleepers = g.population.groups.get_mut(&prey).unwrap();
    std::sync::Arc::make_mut(&mut sleepers.entity).traits.insert("stage:dormant".into());
    let asleep = Session::new(g, Execution::Individuals).unwrap();
    assert_eq!(asleep.sim.edible_matter(eater, FEED).unwrap(), 0);
}

#[test]
fn a_dead_or_dormant_intake_reads_nothing_and_the_demand_follows_the_contract() {
    let (g, eater, _) = meadow(3, 5);
    let s = Session::new(g, Execution::Individuals).unwrap();
    let p = &s.sim.genesis().rules.processes[FEED];
    let Effect::Eat { amount, .. } = &p.effects[0] else { panic!("one ordinary Eat") };
    let bite = amount.resolved().unwrap();
    let lineage = &s.sim.state().population.get(eater).unwrap().lineage;
    let paced = s.sim.state().lineages[lineage].decides.unwrap_or(1);
    assert_eq!(s.sim.feed_demand(FEED, lineage).unwrap(), bite * paced);
}
