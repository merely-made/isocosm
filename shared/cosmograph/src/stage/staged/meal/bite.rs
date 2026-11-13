// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eaten, and it harms (spoilage, lineage-rounds D37): a meal of matter the
//! eater's mouth admits but does not tolerate still happens, and its eater
//! takes the rule's wound after it. Without the rule nothing here acts and
//! a mouth refuses what it does not admit.

use super::*;
use crate::{
    ecology::Flesh,
    mosaic::ports::admission::{Cause, Eaten, Source},
};

/// What a harming bite was of: the age of its flesh, its lineage, and why.
pub(super) type Harmed = (Flesh, Key, Cause);

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
        if let Some(harmed) = self.eat(from, amount, into, of, whole)? {
            self.hurt_eater(from, harmed)?;
        }
        Ok(())
    }

    /// The class, lineage and cause of the first of `keys` (matter the meal
    /// took) that harms the eater's mouth; none where no rule is set, the
    /// meal has no declared mouth, or the mouth tolerates all of it.
    pub(super) fn untolerated<'k>(
        &mut self,
        from: Binding,
        keys: impl IntoIterator<Item = &'k Key>,
    ) -> Result<Option<Harmed>> {
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let (true, Some(feeding), Some(id)) = (
            rules.spoilage.is_some() || rules.categories.is_some(),
            self.stage.port_feeding.as_ref(),
            self.stage.intake,
        ) else {
            return Ok(None);
        };
        let intake = feeding.intake(&self.stage.bodies[&self.stage.actor], id)?;
        let Some(port) = crate::mosaic::ports::active(intake) else { return Ok(None) };
        // What the body's other organs tolerate counts too (a repertoire's
        // tolerance organ, D37); in a founded body only the mouth has any.
        let port = port.with_tolerance_bits(crate::mosaic::ports::ports(&self.stage.bodies[&self.stage.actor]).tolerance_bits());
        let bitten = self.stage.bitten;
        let prey = self.body(from)?;
        let flesh = rules.spoilage.map_or(Flesh::Fresh, |rule| rule.of(prey, sim.state.tick));
        let eaten = Eaten::part(prey, bitten, rules);
        for key in keys {
            let source = Source::of(key, eaten, rules, &sim.state.lineages)?;
            if let Some(cause) = source.harms(port, flesh) {
                return Ok(Some((flesh, source.lineage().to_owned(), cause)));
            }
        }
        Ok(None)
    }

    fn hurt_eater(&mut self, from: Binding, (flesh, lineage, cause): Harmed) -> Result<()> {
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let cells = match cause {
            Cause::Flesh => rules.spoilage.map(|rule| rule.harm),
            Cause::Tough => rules.categories.map(|rule| rule.harm),
        };
        // A planning pass only counts what a meal would take.
        let Some(cells) = cells.filter(|_| self.stage.demands.is_none()) else {
            return Ok(());
        };
        if crate::harm::cells(self.body(Binding::Actor)?) == 0 {
            return Ok(());
        }
        self.wound(Binding::Actor, u64::from(cells), WOUND_SLOT)?;
        let prey = sim.bound(self.stage.actor, self.stage.target, from)?;
        self.stage.harms.push(crate::flows::Harm {
            tick: sim.state.tick,
            made_by: self.stage.process.clone(),
            eater: self.stage.actor,
            prey,
            flesh,
            lineage,
            cells,
            tough: cause == Cause::Tough,
            place: self.stage.place,
        });
        Ok(())
    }
}
