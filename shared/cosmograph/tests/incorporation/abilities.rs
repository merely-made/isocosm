// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A borrowed ability ends with its part (cases 1233 and 1243).
use super::*;
use cosmograph::directing::readings::{abilities, unable};

fn fixing(g: &mut Genesis) {
    let mut p = world_process("test:fixing");
    p.requires.push(Query::Expresses { function: "function:fix".into() });
    g.rules.processes.insert(p.id.clone(), p);
}

fn world_process(id: &str) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires: vec![],
        commitments: vec![],
        effects: vec![],
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
        port_feeding: None,
        goal_directives: Vec::new(),
    }
}

fn fix(s: &Session, id: Id) -> Option<Ability> {
    abilities(&s.sim, id).into_iter().find(|a| a.function == "function:fix")
}

fn barred(s: &Session, id: Id) -> bool {
    unable(&s.sim, id).iter().any(|u| u.process == "test:fixing")
}

use cosmograph::directing::readings::Ability;

#[test]
fn a_native_whole_part_lends_its_function_until_it_is_lost() {
    let (mut g, eater, prey) = world(0, 1000, true);
    fixing(&mut g);
    let mut s = Session::new(g.clone(), Execution::Individuals).unwrap();
    assert!(fix(&s, eater).is_none(), "the grazer grows nothing that fixes");
    assert!(barred(&s, eater));
    assert!(!barred(&s, prey), "the producer fixes with its own");
    assert!(!fix(&s, prey).unwrap().borrowed);
    let eaten = s.command(Command::Act {
        actor: eater,
        target: Some(prey),
        process: "test:graze".into(),
        cause: None,
    });
    assert!(eaten.unwrap().contains("Accepted"));
    let lent = fix(&s, eater).expect("the frond landed");
    assert_eq!(lent.cells, 3);
    assert!(lent.borrowed);
    assert_eq!(lent.from, BTreeSet::from(["lineage:0".to_string()]));
    assert!(!barred(&s, eater), "the borrowed part lets it act");
    let own = abilities(&s.sim, eater).into_iter().filter(|a| a.function != "function:fix");
    assert!(own.into_iter().all(|a| !a.borrowed), "what it grew is its own");
    // The part goes: severed from the body, its ability goes with it.
    g.population = s.sim.state().population.clone();
    g.sites = s.sim.state().sites.clone();
    let frond = *body(&s, eater)
        .parts
        .iter()
        .find(|(_, p)| p.functions.contains("function:fix"))
        .unwrap()
        .0;
    g.population.lift(eater).unwrap().take_part(frond).unwrap();
    let s = Session::new(g, Execution::Individuals).unwrap();
    assert!(fix(&s, eater).is_none(), "a borrowed ability ends with its part");
    assert!(barred(&s, eater));
}
