// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Execution, Founding, Simulation, rules::*, simulation::Outcome};
use std::collections::BTreeMap;

#[test]
fn an_act_that_would_make_matter_is_refused_and_changes_nothing() {
    let founding = Founding {
        seed: 5,
        sites: 2,
        population: 8,
        cohort_size: 4,
        lineages: 1,
        ..Default::default()
    };
    let mut sim = Simulation::new(founding.generate().unwrap(), Execution::Grouped).unwrap();
    // Admission refuses an unbalanced transform, so this one enters the
    // rules after it. The note before it must not survive either.
    let mut conjure = sim.genesis.rules.processes["sim:remember"].clone();
    conjure.id = "test:conjure".into();
    conjure.effects.push(Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([("world:soil".into(), 1)]),
    });
    let rules = &mut std::sync::Arc::make_mut(&mut sim.genesis).rules;
    rules.processes.insert(conjure.id.clone(), conjure);
    let hash = sim.state_hash();
    let groups = sim.state.population.groups.len();
    let r = sim.execute(3, None, "test:conjure", None);
    let refusal = "matter invariant would be violated";
    assert_eq!(r.outcome, Outcome::Refused(refusal.into()));
    assert_eq!(
        (r.matter_before, r.matter_after),
        (sim.matter(), sim.matter())
    );
    assert_eq!(sim.state_hash(), hash);
    assert_eq!(sim.state.population.groups.len(), groups);
    assert!(sim.state.notes.is_empty() && sim.state.events.is_empty());
}
