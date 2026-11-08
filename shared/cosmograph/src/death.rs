// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Death drawn once (the execution plan's E7b, case 1181). A rising hazard
//! whose only outcome is death, run every tick, falls on the tick a single
//! draw names, at the per-tick odds (`Aging::falls_after`), keyed by actor,
//! tick and process (D1). Its odds read only the actor's age and the rules,
//! which never change in a world, so the tick is drawn once, when the actor
//! first comes under the hazard: at maturity, or at the world's first tick
//! for a founder already mature. A process whose gates could open and close
//! again (an account, a site, a trait some process sets) keeps its draw
//! every tick: one draw could not follow it; so does every hazard where
//! far members fold into bands, whose identities a draw could not follow.

use crate::{rules::*, schema::*, simulation::Genesis};

/// A hazard drawn once, with what its draw reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Drawn {
    process: Key,
    aging: Aging,
    /// The age from which the actor is under the hazard: its gate's or its
    /// maturity, whichever is later.
    from_age: Tick,
    /// The first tick the hazard's pass runs.
    first: Tick,
    seed: u64,
}

impl Drawn {
    /// How `p` draws, where it is drawn once.
    pub(crate) fn of(p: &Process, genesis: &Genesis) -> Option<Self> {
        let risk = p.risk.as_ref()?;
        let aging = risk.aging?;
        let only_death = risk.effects == [Effect::Death];
        let fixed_gates = p.requires.iter().all(|q| {
            matches!(q, Query::Alive(Binding::Actor) | Query::Trait { who: Binding::Actor, .. } | Query::Age { .. })
        });
        let gate_age = p.requires.iter().filter_map(|q| match q {
            Query::Age { at_least } => Some(*at_least),
            _ => None,
        }).max().unwrap_or(0);
        let ok = only_death
            && fixed_gates
            && p.effects.is_empty()
            && p.commitments.is_empty()
            && p.target.is_none()
            && p.period == Some(1)
            && p.causation == Causation::Transition
            && p.goal_directives.is_empty()
            && genesis.rules.banding.is_none()
            && !sets_trait(p, &genesis.rules);
        ok.then(|| Self {
            process: p.id.clone(),
            aging,
            from_age: gate_age.max(aging.maturity),
            first: genesis.started.saturating_add(1),
            seed: genesis.dynamics_seed(),
        })
    }

    /// The tick the hazard falls on an actor born at `born`, none if never.
    pub(crate) fn tick(&self, actor: Id, born: Tick) -> Option<Tick> {
        let from = born.saturating_add(self.from_age).max(self.first);
        let u = crate::draw(self.seed, &self.process, &[actor, from]);
        let after = self.aging.falls_after(from.saturating_sub(born), u)?;
        from.checked_add(after)
    }

    /// The first tick any of a group's members falls, none if never.
    pub(crate) fn first_of(&self, first: Id, count: u64, born: Tick) -> Option<Tick> {
        (first..first.saturating_add(count)).filter_map(|actor| self.tick(actor, born)).min()
    }
}

/// Whether any process could set or clear one of the traits `p` gates on.
fn sets_trait(p: &Process, rules: &Rules) -> bool {
    let gated: Vec<&Key> = p.requires.iter().filter_map(|q| match q {
        Query::Trait { key, .. } => Some(key),
        _ => None,
    }).collect();
    fn sets(e: &Effect, gated: &[&Key]) -> bool {
        match e {
            Effect::Trait { key, .. } => gated.contains(&key),
            _ => e.branches().any(|b| sets(b, gated)),
        }
    }
    rules.processes.values().any(|q| {
        let risky = q.risk.iter().flat_map(|r| &r.effects);
        q.commitments.iter().chain(&q.effects).chain(risky).any(|e| sets(e, &gated))
    })
}
