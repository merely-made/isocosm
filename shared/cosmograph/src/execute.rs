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
    Act,
    Scheduled(Option<Shares>),
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
        let run = Run::Act;
        self.run((actor, target), process, cause, (count, act), run)
            .0
    }

    pub(crate) fn scheduled_apply(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        count: u64,
    ) -> Receipt {
        let act = self.state.next_action;
        self.run(
            (actor, target),
            process,
            None,
            (count, act),
            Run::Scheduled(None),
        )
        .0
    }

    /// A planned act, with the target, number and shares its pass gave it.
    pub(crate) fn act(&mut self, actor: Id, process: &str, count: u64, plan: Planned) -> Receipt {
        let run = Run::Scheduled(Some(plan.shares));
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
        at: (Id, Option<Id>),
        process: &str,
        cause: Option<Key>,
        sequence: (u64, u64),
        run: Run,
    ) -> (Receipt, Option<Demands>) {
        // Read from the shared genesis, never copied per act (EX4).
        let genesis = std::sync::Arc::clone(&self.genesis);
        let definition = genesis.rules.processes.get(process);
        let producer = definition.is_some_and(|p| {
            p.requires
                .iter()
                .any(|q| matches!(q, Query::Institution { .. }))
        }) && self.pass.is_some();
        let mut cause = cause;
        let prior = if producer {
            let p = definition.expect("producer definition is present");
            if let Ok(declared) = self.institution_process_cause(at.0, at.1, p) {
                cause = declared;
            }
            let scope = self.institution_process_scope(at.0, at.1, p).ok().flatten();
            Some((
                std::mem::replace(&mut self.institution_decision, scope),
                self.institution_decision_stamp.take(),
            ))
        } else {
            None
        };
        let phase = match &run { Run::Plan => "plan", Run::Scheduled(_) => "scheduled", Run::Act => "direct" };
        let observed = self.diagnostic_begin(at.0, at.1, definition, sequence.0, phase);
        // Capture lives for this disposable stage and is cleared on every exit.
        let _ = self.begin_process_debits(at.0, at.1, process, sequence.1);
        let result = self.run_inner(at, process, cause, sequence, run);
        self.diagnostic_finish(observed, &result.0);
        self.finish_process_debits(&result.0);
        if let Some((decision, stamp)) = prior {
            self.institution_decision = decision;
            self.institution_decision_stamp = stamp;
        }
        result
    }

    fn run_inner(
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
        // A risk that does not fall on a process that otherwise does nothing
        // makes no act whatever the checks before it find (E7b), and its draw
        // is keyed, not sequenced (D1); so an unobserved scheduled or planned
        // act draws first and stops there (EX4). A direct act's receipt is
        // read, so it takes every check.
        if !matches!(run, Run::Act)
            && !self.diagnostic_attached()
            && definition.effects.is_empty()
            && definition.commitments.is_empty()
            && definition.risk.is_some()
            && !self.risk_falls(definition, actor, &id)
        {
            return refused(receipt, Outcome::Blocked("the risk did not fall".into()));
        }
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
        let event_place = entity.event_place();
        if !self.route_ready(actor, definition) {
            return refused(
                receipt,
                Outcome::Blocked("route actor is not resident or route is absent".into()),
            );
        }
        if definition.target.is_some() && !self.target_matches(actor, target, definition) {
            let why = "no target satisfies the declared scope".into();
            return refused(receipt, Outcome::Blocked(why));
        }
        if !self.foraging_walk_matches(actor, target, definition) {
            return refused(
                receipt,
                Outcome::Blocked("walking has no eligible food serving a current need".into()),
            );
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
        let intake = match self.bind_intake(actor, definition) {
            Ok(part) => part,
            Err(why) => return refused(receipt, Outcome::Blocked(why)),
        };
        if let Some(part) = intake {
            receipt
                .facts_read
                .push(format!("declared intake organ: {part:?}"));
        }
        self.diagnostic_stage("requirements");
        for query in &definition.requires {
            match self.process_query(actor, target, place, part, definition, query) {
                Ok(fact) => receipt.facts_read.push(fact),
                Err(why) => return refused(receipt, Outcome::Blocked(why)),
            }
        }
        // Only a direct act's receipt is kept to be read (685); a scheduled
        // or planned one is read only for whether it was accepted.
        if matches!(run, Run::Act) {
            receipt.foregone = genesis
                .rules
                .processes
                .values()
                .filter(|p| p.id != process && p.causation == Causation::Choice)
                .filter(|p| {
                    let part = self.bind_part(actor, p);
                    self.route_ready(actor, p)
                        && self.target_matches(actor, target, p)
                        && self.foraging_walk_matches(actor, target, p)
                        && self.owned_process_ready(actor, target, p)
                        && p.requires
                            .iter()
                            .all(|q| self.process_query(actor, target, place, part, p, q).is_ok())
                })
                .map(|p| p.id.clone())
                .collect();
        }
        self.diagnostic_stage("risk");
        let risky = self.risk_falls(definition, actor, &id);
        // A risk that does not fall, on a process that otherwise does
        // nothing, is no act: nothing is staged, numbered or written (E7b).
        if !risky
            && definition.risk.is_some()
            && definition.effects.is_empty()
            && definition.commitments.is_empty()
        {
            return refused(receipt, Outcome::Blocked("the risk did not fall".into()));
        }
        // Writes go to a stage of what the act binds, never to the world,
        // until every check below has passed.
        self.diagnostic_stage("staging");
        let mut stage = match self.stage((actor, target), place, part, count, act) {
            Ok(stage) => stage,
            Err(why) => return refused(receipt, Outcome::Blocked(why)),
        };
        let planning = matches!(run, Run::Plan);
        stage.scheduled = matches!(run, Run::Scheduled(_));
        stage.process = process.into();
        stage.action = id.clone();
        stage.intake = intake;
        stage.port_feeding = definition.port_feeding.clone();
        match run {
            Run::Plan => stage.demands = Some(Demands::default()),
            Run::Scheduled(shares) => stage.shares = shares,
            Run::Act => {},
        }
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
        for (index, effect) in effects.iter().enumerate() {
            if staged.sim.diagnostic_active() { staged.sim.diagnostic_stage(&format!("effect:{:?}", effect.kind())); }
            if planning || staged.stage.shares.is_some() {
                let commitments = definition.commitments.len();
                staged.stage.effect_at.clear();
                staged.stage.effect_at.extend(if index < commitments {
                    [0, index]
                } else {
                    [1 + usize::from(risky), index - commitments]
                });
            }
            match staged.effect(effect, &id) {
                Ok(feat) => legend |= feat,
                Err(why) => {
                    let terrain = staged.stage.terrain.clone();
                    let step = staged.stage.step.clone();
                    drop(staged);
                    self.capture_geometry(&terrain);
                    self.capture_step(&step);
                    return refused(receipt, Outcome::Blocked(why));
                },
            }
        }
        self.diagnostic_stage("commit_checks");
        if stage.direction.is_none()
            && let Err(why) = self.complete_process_debits()
        {
            return refused(receipt, Outcome::Blocked(why));
        }
        // A geometry answer was consumed even if a later conservation,
        // shared-ground or event check refuses the whole staged act.
        // Only commit_step records actual physical acceptance.
        let step = stage.step.clone();
        self.capture_step(&step);
        let terrain = stage.terrain.clone();
        self.capture_geometry(&terrain);
        if stage.shares.as_ref().is_some_and(|s| !s.meals.is_empty()) {
            return refused(
                receipt,
                Outcome::Blocked("scheduled meal path left an unused quota".into()),
            );
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
                place: event_place,
                subject: actor,
                object: target,
                process: process.into(),
                cause,
                agreement: None,
                strength: genesis.rules.field.strength,
                legend,
                authored: None,
            };
            if let Err(why) = self.stage_event(&mut stage, event) {
                return refused(receipt, Outcome::Refused(why));
            }
        }
        if let Err(why) = self.stage_goal_directives(&mut stage, definition, next) {
            return refused(receipt, Outcome::Blocked(why));
        }
        if let Err(why) = self.stage_graft_receipt(&mut stage) {
            return refused(receipt, Outcome::Blocked(why));
        }
        if let Err(why) = self.finish_staged_institution(&mut stage, risky) {
            return refused(receipt, Outcome::Blocked(why));
        }
        receipt.effects = effects.into_iter().cloned().collect();
        if risky {
            receipt.outcome = Outcome::RiskOutcome;
        }
        if planning {
            let demands = stage.demands.take();
            if stage.direction.is_some() {
                let mut preview = self.clone();
                let result = preview.commit_resolved(stage, next, process);
                self.state
                    .projects
                    .debit_capture
                    .set(preview.state.projects.debit_capture.take());
                if let Err(why) = result {
                    return refused(receipt, Outcome::Blocked(why));
                }
            }
            return (receipt, demands);
        }
        if let Err(why) = self.commit_resolved(stage, next, process) {
            return refused(receipt, Outcome::Blocked(why));
        }
        receipt.matter_after = self.conserved;
        debug_assert_eq!(
            self.matter(),
            self.conserved,
            "an accepted act changed the world's matter"
        );
        (receipt, None)
    }

    /// Whether a scheduled act of `p` by `actor`, numbered as the next act,
    /// is sure to make no act: `p` does nothing but a risk, its risk does not
    /// fall, and nothing observes or decides through the act (E7b, EX4). A
    /// blocked act leaves nothing a later act reads, so the pass need not
    /// stage it.
    pub(crate) fn risk_misses(&self, actor: Id, p: &Process) -> bool {
        p.effects.is_empty()
            && p.commitments.is_empty()
            && p.causation != Causation::Choice
            && !p.requires.iter().any(|q| matches!(q, Query::Institution { .. }))
            && !self.diagnostic_attached()
            && p.risk.is_some()
            && !self.risk_falls(p, actor, &self.owned_action(actor, &p.id, self.state.next_action))
    }

    /// Whether `p`'s risk falls on `actor` for the act named `id`: a keyed
    /// draw against its odds; for a rising hazard its per-tick odds at the
    /// actor's age (1077), or where that hazard is drawn once, whether the
    /// tick it names has come (E7b).
    fn risk_falls(&self, p: &Process, actor: Id, id: &str) -> bool {
        let Some(risk) = &p.risk else { return false };
        let seed = self.genesis.dynamics_seed();
        let born = self.state.population.get(actor).map(|e| e.born);
        match &risk.aging {
            Some(_) if let Some(drawn) = crate::death::Drawn::of(p, &self.genesis) => {
                born.and_then(|b| drawn.tick(actor, b)).is_some_and(|due| self.state.tick >= due)
            },
            Some(aging) => {
                let age = born.map_or(0, |b| self.state.tick.saturating_sub(b));
                aging.per_tick(age).is_some_and(|chance| {
                    crate::draw(seed, id, &[actor]) % 1_000_000_000_000 < chance
                })
            },
            None => crate::draw(seed, id, &[actor]) % 1_000_000 < u64::from(risk.per_million),
        }
    }
}
