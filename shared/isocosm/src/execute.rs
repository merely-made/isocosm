// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{rules::*, schema::*, simulation::*, stage::Staged};

impl Simulation {
    pub(crate) fn apply(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        cause: Option<Key>,
        count: u64,
    ) -> Receipt {
        let before = self.conserved;
        let id = format!(
            "event:{}",
            crate::digest(&(
                self.genesis.seed,
                self.state.tick,
                self.state.next_action,
                actor,
                process
            ))
        );
        let mut receipt = Receipt {
            id: id.clone(),
            tick: self.state.tick,
            revision: self.revision.clone(),
            actor,
            target,
            count,
            chosen: process.into(),
            foregone: vec![],
            cause: cause.clone(),
            facts_read: vec![],
            effects: vec![],
            outcome: Outcome::Accepted,
            matter_before: before,
            matter_after: before,
        };
        // The definition is read from the shared genesis while the world is
        // written.
        let genesis = std::sync::Arc::clone(&self.genesis);
        let Some(definition) = genesis.rules.processes.get(process) else {
            receipt.outcome = Outcome::Refused(format!("unknown process {process}"));
            return receipt;
        };
        if count == 0 || (count > 1 && !definition.bulk_safe()) {
            receipt.outcome = Outcome::Refused("process cannot execute as a cohort".into());
            return receipt;
        }
        if count > 1
            && self
                .state
                .population
                .groups
                .get(&actor)
                .is_none_or(|g| g.count != count)
        {
            receipt.outcome = Outcome::Refused("cohort identity interval changed".into());
            return receipt;
        }
        let Some(entity) = self.state.population.get(actor) else {
            receipt.outcome = Outcome::Refused("actor is absent".into());
            return receipt;
        };
        let place = entity.place;
        if definition.target.is_some() && !self.target_matches(actor, target, definition) {
            receipt.outcome = Outcome::Blocked("no target satisfies the declared scope".into());
            return receipt;
        }
        if cause
            .as_ref()
            .is_some_and(|id| !self.state.events.contains_key(id))
        {
            receipt.outcome = Outcome::Refused("unknown causal event".into());
            return receipt;
        }
        for query in &definition.requires {
            match self.query(actor, target, place, query) {
                Ok(fact) => receipt.facts_read.push(fact),
                Err(why) => {
                    receipt.outcome = Outcome::Blocked(why);
                    return receipt;
                },
            }
        }
        receipt.foregone = genesis
            .rules
            .processes
            .values()
            .filter(|p| p.id != process && p.shape == Shape::Choice)
            .filter(|p| {
                self.target_matches(actor, target, p)
                    && p.requires
                        .iter()
                        .all(|q| self.query(actor, target, place, q).is_ok())
            })
            .map(|p| p.id.clone())
            .collect();
        // Writes go to a stage of what the act binds, never to the world,
        // until every check below has passed.
        let mut stage = match self.stage(actor, target, place, count) {
            Ok(stage) => stage,
            Err(why) => {
                receipt.outcome = Outcome::Blocked(why);
                return receipt;
            },
        };
        let risky = definition.risk.as_ref().is_some_and(|r| {
            crate::draw(self.genesis.dynamics_seed(), &id, &[actor]) % 1_000_000
                < u64::from(r.per_million)
        });
        let outcomes = if risky {
            &definition.risk.as_ref().unwrap().effects
        } else {
            &definition.effects
        };
        let effects: Vec<&Effect> = definition.commitments.iter().chain(outcomes).collect();
        let mut legend = false;
        let mut staged = Staged {
            sim: self,
            stage: &mut stage,
        };
        for effect in &effects {
            match staged.effect(effect, &id) {
                Ok(feat) => legend |= feat,
                Err(why) => {
                    receipt.outcome = Outcome::Blocked(why);
                    return receipt;
                },
            }
        }
        // Only what the act bound can have changed, so weighing it alone
        // decides whether the world's matter would change.
        if self.moves_matter(&stage) {
            receipt.outcome = Outcome::Refused("matter invariant would be violated".into());
            return receipt;
        }
        let Some(next) = self.state.next_action.checked_add(count) else {
            receipt.outcome = Outcome::Refused("action sequence exhausted".into());
            return receipt;
        };
        if definition.note {
            if self.state.events.len() >= genesis.rules.limits.history {
                receipt.outcome = Outcome::Refused("event budget exhausted".into());
                return receipt;
            }
            let event = Event {
                id: id.clone(),
                tick: self.state.tick,
                place,
                subject: actor,
                process: process.into(),
                cause,
                strength: genesis.rules.field.strength,
                legend,
            };
            if let Err(why) = self.stage_event(&mut stage, event) {
                receipt.outcome = Outcome::Refused(why);
                return receipt;
            }
        }
        receipt.effects = effects.into_iter().cloned().collect();
        if risky {
            receipt.outcome = Outcome::RiskOutcome;
        }
        self.commit(stage, next);
        debug_assert_eq!(
            self.matter(),
            self.conserved,
            "an accepted act changed the world's matter"
        );
        receipt
    }
}
