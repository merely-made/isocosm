// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The player's draft (cases 1200, 1203; lineage-rounds D10): a whole meal
//! in a real stage is counted against the donor's line, and teaches its
//! kind only at the count its kinship asks.
use super::*;
use cosmograph::lineage::draft;

fn asked(g: &Genesis) -> u8 {
    draft::needed(&g.lineages, &g.rules, "lineage:1", "lineage:0", "kind:frond")
}

fn drafted() -> (Genesis, Id, Id) {
    let (mut g, eater, prey) = world(0, 1000, true);
    g.rules.directing = Some(Directing { draft: Some(Draft::default()), ..Default::default() });
    (g, eater, prey)
}

fn eat(g: Genesis, eater: Id, prey: Id) -> Session {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let command = Command::Act {
        actor: eater,
        target: Some(prey),
        process: "test:graze".into(),
        cause: None,
    };
    s.command(command).unwrap();
    s
}

#[test]
fn a_whole_meal_is_counted_and_teaches_only_at_the_needed_count() {
    let (g, eater, prey) = drafted();
    let needed = asked(&g);
    assert!(needed > 1, "another line's frond is not learned at a bite");
    let s = eat(g.clone(), eater, prey);
    let line = &s.sim.state().lineages["lineage:1"];
    let meal = line.meals["kind:frond"];
    assert_eq!((meal.count, meal.needed), (1, needed));
    assert_eq!(meal.last, s.sim.state().tick);
    assert!(!lexicon(&s).contains("kind:frond"), "one meal does not teach it");
    assert!(s.sim.state().lineages["lineage:0"].meals.is_empty(), "the donor learned nothing");
    // With one meal short already eaten, this one teaches it.
    let mut g = g;
    let owed = Eaten { count: u32::from(needed) - 1, needed, last: 0 };
    g.lineages.get_mut("lineage:1").unwrap().meals.insert("kind:frond".into(), owed);
    let s = eat(g, eater, prey);
    assert_eq!(s.sim.state().lineages["lineage:1"].meals["kind:frond"].count, u32::from(needed));
    assert!(lexicon(&s).contains("kind:frond"));
}

#[test]
fn with_the_rule_unset_a_meal_teaches_at_once_and_counts_nothing() {
    let (g, eater, prey) = world(0, 1000, true);
    let s = eat(g, eater, prey);
    assert!(lexicon(&s).contains("kind:frond"));
    assert!(s.sim.state().lineages["lineage:1"].meals.is_empty());
}

#[test]
fn a_run_with_a_drafted_meal_replays_to_the_same_hash() {
    let (g, eater, prey) = drafted();
    let mut s = eat(g, eater, prey);
    s.advance(3).unwrap();
    assert!(!s.sim.state().lineages["lineage:1"].meals.is_empty());
    let tick = s.sim.state().tick;
    let again = s.prefix(s.entries.len(), tick).unwrap();
    assert_eq!(again.sim.state_hash(), s.sim.state_hash());
    assert_eq!(again.sim.state().lineages, s.sim.state().lineages);
}

#[test]
fn a_real_meal_is_recorded_as_eaten_and_dealt_between_two_lines_when_regard_is_set() {
    let (mut g, eater, prey) = world(0, 1000, true);
    g.rules.directing = Some(Directing { regard: Some(Default::default()), ..Default::default() });
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let command = Command::Act { actor: eater, target: Some(prey), process: "test:graze".into(), cause: None };
    let moves = s.command_with_flows(command).unwrap().flows;
    assert_eq!(s.sim.state().lineages["lineage:1"].ate.iter().collect::<Vec<_>>(), ["lineage:0"]);
    assert!(s.sim.state().lineages["lineage:0"].ate.is_empty(), "the donor ate nothing");
    let between: Vec<_> = moves.iter().filter(|f| matches!((&f.from_kind, &f.to_kind),
        (Some(a), Some(b)) if a.lineage == "lineage:0" && b.lineage == "lineage:1")).collect();
    assert_eq!(between.len(), 1, "a meal is one move from the prey's body to the eater's");
}
