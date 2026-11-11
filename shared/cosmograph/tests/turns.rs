// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Turn groups: processes that share ground and name one group take first
//! place in rotation, so no identity decides who takes first. Unnamed, the
//! order is as it always was; the rotation is a function of the tick, so
//! cutting an advance, or grouping the members, changes nothing.

use cosmograph::{Execution, Founding, Simulation, population::Population, rules::*, simulation::Genesis};
use std::collections::{BTreeMap, BTreeSet};

/// A site holding `soil`, and two lineages of one body each, each with its
/// own pass taking one soil at the site every second tick: `turn` names the
/// group of both, or neither.
fn contest(soil: u64, turn: Option<&str>) -> Genesis {
    let mut g = Founding { seed: 4, sites: 1, population: 3, lineages: 2, cohort_size: 1, ..Default::default() }
        .generate()
        .unwrap();
    let world = g.population.groups[&0].entity.as_ref().clone();
    let mut body = g.population.get(1).unwrap().clone();
    body.accounts = BTreeMap::new();
    body.born = 0;
    let mut population = Population::default();
    population.insert(world, 1).unwrap();
    for i in 0..2 {
        let mut e = body.clone();
        e.lineage = format!("lineage:{i}");
        e.traits = BTreeSet::from([format!("test:own-{i}")]);
        population.insert(e, 1).unwrap();
        g.rules.traits.insert(format!("test:own-{i}"));
    }
    g.population = population;
    g.sites.values_mut().for_each(|s| { s.accounts.insert("world:soil".into(), soil); });
    g.rules.processes.retain(|_, p| p.period.is_none());
    for i in 0..2 {
        let mut take = cosmograph::rules::Process {
            id: format!("test:take-{i}"),
            causation: Causation::Transition,
            requires: vec![Query::Alive(Binding::Actor), Query::Trait { who: Binding::Actor, key: format!("test:own-{i}") }],
            commitments: vec![],
            effects: vec![Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: "world:soil".into(), amount: 1.into() }],
            risk: None,
            target: None,
            period: Some(2),
            priority: 0,
            need_account: None,
            need_below: 0,
            glyphs: BTreeSet::new(),
            invariants: BTreeSet::new(),
            note: false,
            port_feeding: None,
            goal_directives: Vec::new(), turn: None,
        };
        take.turn = turn.map(Into::into);
        g.rules.processes.insert(take.id.clone(), take);
    }
    g
}

/// What each lineage's body holds of soil after `ticks`, advanced in `cuts`.
fn held(turn: Option<&str>, mode: Execution, cuts: &[u64]) -> [u64; 2] {
    let mut sim = Simulation::new(contest(5, turn), mode).unwrap();
    for &cut in cuts { sim.advance(cut).unwrap(); }
    let at = |id| sim.state().population.get(id).unwrap().accounts.get("world:soil").copied().unwrap_or(0);
    [at(1), at(2)]
}

#[test]
fn without_a_group_the_first_identity_always_eats_first() {
    assert_eq!(held(None, Execution::Individuals, &[12]), [3, 2]);
}

#[test]
fn processes_in_a_group_alternate_first_place_across_occasions() {
    // Five soil, two takers: the first two occasions give each one apiece
    // and the last unit goes to whoever has first place at the third, which
    // is the second lineage where identity alone gives it to the first.
    assert_eq!(held(Some("turn:test"), Execution::Individuals, &[12]), [2, 3]);
}

#[test]
fn a_turn_group_replays_the_same_however_it_is_cut_or_grouped() {
    let whole = held(Some("turn:test"), Execution::Individuals, &[12]);
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(held(Some("turn:test"), mode, &[12]), whole, "{mode:?} whole");
        assert_eq!(held(Some("turn:test"), mode, &[1; 12]), whole, "{mode:?} ticks");
        assert_eq!(held(Some("turn:test"), mode, &[5, 3, 4]), whole, "{mode:?} uneven");
    }
}

#[test]
fn a_turn_is_saved_only_where_named() {
    let g = contest(5, Some("turn:test"));
    let named = serde_json::to_value(&g.rules.processes["test:take-0"]).unwrap();
    assert_eq!(named["turn"], "turn:test");
    let back: Process = serde_json::from_value(named).unwrap();
    assert_eq!(back.turn.as_deref(), Some("turn:test"));
    let bare = serde_json::to_value(&contest(5, None).rules.processes["test:take-0"]).unwrap();
    assert!(bare.get("turn").is_none(), "an unnamed turn is not saved, so it hashes as before");
    let back: Process = serde_json::from_value(bare).unwrap();
    assert_eq!(back.turn, None);
}
