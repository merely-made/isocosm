// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    rules::*,
    schedule::{Demands, Planned, Shares},
    schema::*,
    simulation::*,
    stage::Staged,
};

/// How an act runs: written to the world, with the shares of shared ground
/// its pass gave it, or planned and written nowhere (ruling 454).
enum Run {
    Act(Option<Shares>),
    Plan,
}

impl Simulation {
    pub(crate) fn apply(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        cause: Option<Key>,
        count: u64,
    ) -> Receipt {
        let act = self.state.next_action;
        let run = Run::Act(None);
        self.run((actor, target), process, cause, (count, act), run)
            .0
    }

    /// A planned act, with the target, number and shares its pass gave it.
    pub(crate) fn act(&mut self, actor: Id, process: &str, count: u64, plan: Planned) -> Receipt {
        let run = Run::Act(Some(plan.shares));
        let at = (actor, plan.target);
        self.run(at, process, None, (count, plan.act), run).0
    }

    /// What an act acting as number `act` would take of shared ground, if
    /// it would be accepted; nothing is written.
    pub(crate) fn demands(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        count: u64,
        act: u64,
    ) -> Option<Demands> {
        let (receipt, demands) = self.run((actor, target), process, None, (count, act), Run::Plan);
        demands.filter(|_| receipt.accepted())
    }

    fn run(
        &mut self,
        (actor, target): (Id, Option<Id>),
        process: &str,
        cause: Option<Key>,
        (count, act): (u64, u64),
        run: Run,
    ) -> (Receipt, Option<Demands>) {
        let before = self.conserved;
        let id = format!(
            "event:{}",
            crate::digest(&(self.genesis.seed, self.state.tick, act, actor, process))
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
            issued: self.issued,
        };
        let refused = |mut receipt: Receipt, outcome: Outcome| {
            receipt.outcome = outcome;
            (receipt, None)
        };
        // The definition is read from the shared genesis while the world is
        // written.
        let genesis = std::sync::Arc::clone(&self.genesis);
        let Some(definition) = genesis.rules.processes.get(process) else {
            return refused(
                receipt,
                Outcome::Refused(format!("unknown process {process}")),
            );
        };
        if count == 0 || (count > 1 && !definition.bulk_safe()) {
            let why = "process cannot execute as a cohort".into();
            return refused(receipt, Outcome::Refused(why));
        }
        if count > 1
            && self
                .state
                .population
                .groups
                .get(&actor)
                .is_none_or(|g| g.count != count)
        {
            let why = "cohort identity interval changed".into();
            return refused(receipt, Outcome::Refused(why));
        }
        let Some(entity) = self.body_at_start(actor) else {
            return refused(receipt, Outcome::Refused("actor is absent".into()));
        };
        let place = entity.place;
        if definition.target.is_some() && !self.target_matches(actor, target, definition) {
            let why = "no target satisfies the declared scope".into();
            return refused(receipt, Outcome::Blocked(why));
        }
        if cause
            .as_ref()
            .is_some_and(|id| !self.state.events.contains_key(id))
        {
            return refused(receipt, Outcome::Refused("unknown causal event".into()));
        }
        // The part the act binds, read before any requirement so that each
        // reads the same one (ruling 338).
        let part = self.bind_part(actor, definition);
        for query in &definition.requires {
            match self.query(actor, target, place, part, query) {
                Ok(fact) => receipt.facts_read.push(fact),
                Err(why) => return refused(receipt, Outcome::Blocked(why)),
            }
        }
        receipt.foregone = genesis
            .rules
            .processes
            .values()
            .filter(|p| p.id != process && p.causation == Causation::Choice)
            .filter(|p| {
                let part = self.bind_part(actor, p);
                self.target_matches(actor, target, p)
                    && p.requires
                        .iter()
                        .all(|q| self.query(actor, target, place, part, q).is_ok())
            })
            .map(|p| p.id.clone())
            .collect();
        // Writes go to a stage of what the act binds, never to the world,
        // until every check below has passed.
        let mut stage = match self.stage((actor, target), place, part, count, act) {
            Ok(stage) => stage,
            Err(why) => return refused(receipt, Outcome::Blocked(why)),
        };
        let planning = matches!(run, Run::Plan);
        match run {
            Run::Plan => stage.demands = Some(Demands::default()),
            Run::Act(shares) => stage.shares = shares,
        }
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
                Err(why) => return refused(receipt, Outcome::Blocked(why)),
            }
        }
        // Only what the act bound can have changed, so weighing it alone
        // decides whether the world's matter would change.
        if self.moves_matter(&stage) {
            let why = "matter invariant would be violated".into();
            return refused(receipt, Outcome::Refused(why));
        }
        // What the act changed must land whole where the pass has written
        // already, which its shares make so for shared ground (ruling 454).
        if !self.fits(&stage) {
            let why = "shared ground was taken by its pass".into();
            return refused(receipt, Outcome::Blocked(why));
        }
        let Some(next) = act.checked_add(count) else {
            return refused(
                receipt,
                Outcome::Refused("action sequence exhausted".into()),
            );
        };
        if definition.note {
            if self.state.events.len() >= genesis.rules.limits.history {
                return refused(receipt, Outcome::Refused("event budget exhausted".into()));
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
                return refused(receipt, Outcome::Refused(why));
            }
        }
        receipt.effects = effects.into_iter().cloned().collect();
        if risky {
            receipt.outcome = Outcome::RiskOutcome;
        }
        if planning {
            let demands = stage.demands.take();
            return (receipt, demands);
        }
        self.commit(stage, next, process);
        debug_assert_eq!(
            self.matter(),
            self.conserved,
            "an accepted act changed the world's matter"
        );
        (receipt, None)
    }
}
