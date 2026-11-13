// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eaten, and it harms (spoilage, lineage-rounds D37): a meal of matter the
//! eater's mouth admits but does not tolerate still happens, and its eater
//! takes the rule's wound after it. Without the rule nothing here acts and
//! a mouth refuses what it does not admit.

use super::*;
use crate::{ecology::Flesh, mosaic::ports::admission::Source};

/// The draw slot of the wound a bite deals its eater, apart from the slots
/// an authored `Wound` names.
const WOUND_SLOT: u8 = 250;

impl Staged<'_> {
    /// Eats, then pays for what the mouth does not tolerate.
    pub(in crate::stage::staged) fn meal(
        &mut self,
        from: Binding,
        amount: &Amount,
        into: &Key,
        of: &[Key],
        whole: bool,
    ) -> Result<()> {
        if let Some((flesh, lineage)) = self.eat(from, amount, into, of, whole)? {
            self.hurt_eater(from, flesh, lineage)?;
        }
        Ok(())
    }

    /// The class and lineage of the first of `keys` (matter the meal took)
    /// that harms the eater's mouth; none where the rule is unset, the meal
    /// has no declared mouth, or the mouth tolerates all of it.
    pub(super) fn untolerated<'k>(
        &mut self,
        from: Binding,
        keys: impl IntoIterator<Item = &'k Key>,
    ) -> Result<Option<(Flesh, Key)>> {
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let (Some(rule), Some(feeding), Some(id)) =
            (rules.spoilage, self.stage.port_feeding.as_ref(), self.stage.intake)
        else {
            return Ok(None);
        };
        let intake = feeding.intake(&self.stage.bodies[&self.stage.actor], id)?;
        let Some(port) = crate::mosaic::ports::active(intake) else { return Ok(None) };
        let prey = self.body(from)?;
        let flesh = rule.of(prey, sim.state.tick);
        for key in keys {
            let source = Source::of(key, prey.alive, rules, &sim.state.lineages)?;
            if source.harms(port, flesh) {
                return Ok(Some((flesh, source.lineage().to_owned())));
            }
        }
        Ok(None)
    }

    fn hurt_eater(&mut self, from: Binding, flesh: Flesh, lineage: Key) -> Result<()> {
        let sim = self.sim;
        // A planning pass only counts what a meal would take.
        let Some(rule) = sim.genesis.rules.spoilage.filter(|_| self.stage.demands.is_none()) else {
            return Ok(());
        };
        if crate::harm::cells(self.body(Binding::Actor)?) == 0 {
            return Ok(());
        }
        self.wound(Binding::Actor, u64::from(rule.harm), WOUND_SLOT)?;
        let prey = sim.bound(self.stage.actor, self.stage.target, from)?;
        self.stage.harms.push(crate::flows::Harm {
            tick: sim.state.tick,
            made_by: self.stage.process.clone(),
            eater: self.stage.actor,
            prey,
            flesh,
            lineage,
            cells: rule.harm,
            place: self.stage.place,
        });
        Ok(())
    }
}
