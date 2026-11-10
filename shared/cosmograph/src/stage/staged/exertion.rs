// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An act's exertion, staged with it (case 1234; see `crate::exertion`).

use super::Staged;
use crate::{
    climate::upkeep_of,
    rules::{Amount, Binding, Causation, Effect, Process},
};
use std::sync::Arc;

impl Staged<'_> {
    /// Counts an act of `process` by the staged actor, and spends `cost`
    /// where it is past the actor's ease. A no-op where the world has no
    /// exertion, or the process is no chosen act: not a Choice, or a
    /// lineage's upkeep.
    pub(crate) fn exert(&mut self, process: &Process, cause: &str) {
        let sim = self.sim;
        let rules = &sim.genesis().rules;
        let Some(exertion) = &rules.exertion else { return };
        if process.causation != Causation::Choice || upkeep_of(&process.id).is_some() {
            return;
        }
        let actor = self.stage.actor;
        // The live count: acts of the pass before this one are already in it.
        let Some(live) = sim.state().population.get(actor) else { return };
        let span = sim.state().tick / exertion.span.max(1);
        let made = live.spent.filter(|(was, _)| *was == span).map_or(0, |(_, n)| n).saturating_add(1);
        let past = made > crate::exertion::easy(rules, live) && exertion.cost > 0;
        let Some(body) = self.stage.bodies.get_mut(&actor) else { return };
        Arc::make_mut(body).spent = Some((span, made));
        if !past {
            return;
        }
        let Some((from, into)) = crate::exertion::upkeep_for(rules, live) else { return };
        let spend = Effect::Spend {
            from,
            to: Binding::Place,
            amount: Amount::Fixed(exertion.cost),
            into: Some(into),
        };
        // Whatever the spend cannot land is left as it was: the act still happens.
        let kept = (Arc::clone(&self.stage.bodies[&actor]), self.stage.site.clone(), self.stage.legs.len());
        if self.effect(&spend, cause).is_err() {
            self.stage.bodies.insert(actor, kept.0);
            self.stage.site = kept.1;
            self.stage.legs.truncate(kept.2);
        }
    }
}
