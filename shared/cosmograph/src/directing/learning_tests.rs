// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Members' learning's controls (cases 1051, 1058; mesocosm-overlay M3.6).
//! The fixture is the played ecology's consumer, lineage 1: always hungry,
//! fed by `ecology:feed-1` where producers share its site, and otherwise
//! taking `ecology:upkeep-1`, which serves no need. Upkeep is a rate in the
//! ecology (D16), so the fixture authors it as a chosen act.

use super::policy;
use super::tests::{chosen_upkeep, founding, played};
use super::*;
use crate::{
    Execution, Session,
    history::Command,
    rules::{Learning, PolicyVariation},
};

mod cohorts;
mod credit;
mod mechanics;
mod tasks;

/// The consumer's own hunger, the one need its acts serve: its place in the
/// world's list.
pub(super) const HUNGER: u32 = 1;

pub(super) const GATE: PolicyVariation = PolicyVariation { founding: 0, expression: 0, candidates: 1 };

/// How a fixture world is made.
pub(super) struct Setup {
    pub learning: Option<Learning>,
    /// The priority of the act that competes with feeding.
    pub upkeep: i32,
    pub mode: Execution,
    /// Whether the consumers' lineage is given its policy.
    pub found: bool,
}

impl Setup {
    pub fn new(learning: Option<Learning>, upkeep: i32) -> Self {
        Self { learning, upkeep, mode: Execution::Individuals, found: true }
    }
}

/// A world whose consumers hold a lineage policy, learning as `learning`
/// says: always hungry, so no run has to take matter away, and long-lived,
/// so no run ends in old age.
pub(super) fn build(seed: u64, setup: Setup, tune: impl FnOnce(&mut crate::simulation::Genesis)) -> Session {
    // The competing act is upkeep authored as a chosen act (D16, case 1076).
    let mut g = chosen_upkeep(founding(seed));
    let d = g.rules.directing.as_mut().unwrap();
    d.policy = GATE;
    d.learning = setup.learning;
    g.rules.processes.get_mut("ecology:upkeep-1").unwrap().priority = setup.upkeep;
    for q in &mut g.rules.processes.get_mut("ecology:age-1").unwrap().requires {
        if let crate::rules::Query::Age { at_least } = q {
            *at_least = 1_000_000;
        }
    }
    for n in &mut g.rules.mind.as_mut().unwrap().needs {
        if n.traits.contains("ability:cycle-1") {
            n.query = crate::rules::Query::Below { who: crate::rules::Binding::Actor, key: "matter:1-0".into(), amount: 1_000_000 };
        }
    }
    tune(&mut g);
    let mut s = Session::new(g, setup.mode).unwrap();
    let (_, c) = played(&mut s);
    if setup.found {
        let lineage = s.sim.state().population.get(c).unwrap().lineage.clone();
        let found = policy::founding(&s, &lineage);
        s.command(found).unwrap();
    }
    s
}

pub(super) fn fixture(seed: u64, learning: Option<Learning>, upkeep: i32) -> Session {
    build(seed, Setup::new(learning, upkeep), |_| {})
}

/// A living producer sharing `c`'s site.
pub(super) fn prey(s: &Session, c: Id) -> bool {
    let pop = &s.sim.state().population;
    let place = pop.get(c).unwrap().place;
    pop.groups.values().any(|g| g.entity.alive && g.entity.place == place && g.entity.lineage == "lineage:0")
}

/// The first seed whose played consumer has prey beside it.
pub(super) fn fed_seed(make: impl Fn(u64) -> Session) -> (u64, Session) {
    (1..40)
        .map(|seed| (seed, make(seed)))
        .find(|(_, s)| prey(s, super::tests::critter(s)))
        .expect("some seed puts prey beside the consumer")
}

/// `c`'s stored delta on hunger, unfaded.
pub(super) fn stored(s: &Session, c: Id) -> Option<i32> {
    let e = s.sim.state().population.get(c)?;
    e.learned.as_ref()?.weights.get(&HUNGER).copied()
}

