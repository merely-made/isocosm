// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Optional native food observations, outside saved state and rules. Plans
//! and refused staged effects are distinct from committed matter changes.

use super::*;
use crate::{rules::*, simulation::Receipt};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

mod breeding;
mod types;
pub use types::*;

type FeedKey = (
    Tick,
    Key,
    Key,
    Option<Key>,
    Option<bool>,
    String,
    bool,
    String,
);
type ExampleKey = (Key, Key, Option<Key>, Option<bool>, String, bool, String);
type LifeKey = (Tick, Key, Key, String);
type BirthKey = (Tick, Key, Key);

/// Like the native debit capture, clones receive independent scratch.
/// The enclosing Option keeps disabled observations off the lock path.
#[derive(Debug)]
pub(super) struct Sink(Mutex<Capture>);

impl Clone for Sink {
    fn clone(&self) -> Self {
        Self(Mutex::new(self.lock().clone()))
    }
}

impl Sink {
    fn lock(&self) -> MutexGuard<'_, Capture> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    fn get_mut(&mut self) -> &mut Capture {
        self.0
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Clone, Debug)]
pub(super) struct Capture {
    config: FoodDiagnosticConfig,
    started: Tick,
    through: Tick,
    feeds: BTreeMap<FeedKey, FeedTally>,
    life: BTreeMap<LifeKey, LifeTally>,
    breeding: BTreeMap<BirthKey, BirthTally>,
    examples: Vec<FeedAttempt>,
    birth_examples: Vec<BirthOpportunity>,
    seen: BTreeMap<ExampleKey, usize>,
    birth_seen: BTreeMap<(Key, Key, bool), usize>,
    active: Vec<FeedAttempt>,
    kept: Option<Box<Capture>>,
}

impl Capture {
    fn new(config: FoodDiagnosticConfig, started: Tick) -> Self {
        Self {
            config,
            started,
            through: started,
            feeds: BTreeMap::new(),
            life: BTreeMap::new(),
            breeding: BTreeMap::new(),
            examples: vec![],
            birth_examples: vec![],
            seen: BTreeMap::new(),
            birth_seen: BTreeMap::new(),
            active: vec![],
            kept: None,
        }
    }
    fn bucket(&self, tick: Tick) -> Tick {
        self.started
            + (tick.saturating_sub(self.started) / self.config.bucket_ticks)
                * self.config.bucket_ticks
    }
    fn report(&self) -> FoodDiagnosticReport {
        FoodDiagnosticReport {
            started: self.started,
            through: self.through,
            bucket_ticks: self.config.bucket_ticks,
            feeds: self.feeds.values().cloned().collect(),
            life: self.life.values().cloned().collect(),
            breeding: self.breeding.values().cloned().collect(),
            feed_examples: self.examples.clone(),
            breeding_examples: self.birth_examples.clone(),
        }
    }
    fn record(&mut self, mut act: FeedAttempt, receipt: &Receipt) {
        act.accepted = receipt.accepted();
        act.outcome = format!("{:?}", receipt.outcome);
        if act.accepted {
            act.stage = "complete".into();
        }
        self.through = self.through.max(act.tick);
        let bucket = self.bucket(act.tick);
        let key = (
            bucket,
            act.process.clone(),
            act.lineage.clone(),
            act.target_lineage.clone(),
            act.target_alive,
            act.phase.clone(),
            act.accepted,
            act.stage.clone(),
        );
        let tally = self.feeds.entry(key).or_insert_with(|| FeedTally {
            bucket,
            first_tick: act.tick,
            last_tick: act.tick,
            process: act.process.clone(),
            lineage: act.lineage.clone(),
            target_lineage: act.target_lineage.clone(),
            target_alive: act.target_alive,
            phase: act.phase.clone(),
            accepted: act.accepted,
            stage: act.stage.clone(),
            attempts: 0,
            selected: 0,
            no_target: 0,
            requested_by_destination: Ledger::new(),
            offered: Ledger::new(),
            reserved: Ledger::new(),
            granted: Ledger::new(),
            staged_taken: Ledger::new(),
            selected_parts: BTreeMap::new(),
            staged_given: Ledger::new(),
            committed_given: Ledger::new(),
        });
        tally.last_tick = act.tick;
        tally.attempts += act.count;
        if act.target.is_some() {
            tally.selected += act.count;
        } else {
            tally.no_target += act.count;
        }
        for meal in &act.meals {
            let add = |to: &mut Ledger, from: &Ledger| {
                for (key, n) in from {
                    let slot = to.entry(key.clone()).or_default();
                    *slot = slot.saturating_add(n.saturating_mul(act.count));
                }
            };
            let requested = tally
                .requested_by_destination
                .entry(meal.into.clone())
                .or_default();
            *requested = requested.saturating_add(meal.asked.saturating_mul(act.count));
            add(&mut tally.offered, &meal.offered);
            if let Some(v) = &meal.reserved {
                add(&mut tally.reserved, v);
            }
            if let Some(v) = &meal.quota {
                add(&mut tally.granted, v);
            }
            add(&mut tally.staged_taken, &meal.taken);
            if let Some(part) = meal.part {
                *tally.selected_parts.entry(part).or_default() += act.count;
            }
            add(&mut tally.staged_given, &meal.given);
            if act.accepted && act.phase != "plan" {
                add(&mut tally.committed_given, &meal.given);
            }
        }
        let key = (
            act.lineage.clone(),
            act.process.clone(),
            act.target_lineage.clone(),
            act.target_alive,
            act.phase.clone(),
            act.accepted,
            act.stage.clone(),
        );
        let seen = self.seen.entry(key).or_default();
        if self.config.focus_lineages.contains(&act.lineage)
            && *seen < self.config.examples_per_kind
        {
            self.examples.push(act);
            *seen += 1;
        }
    }
    fn life(&mut self, tick: Tick, process: &str, lineage: &str, kind: &str, count: u64) {
        let bucket = self.bucket(tick);
        let key = (bucket, process.into(), lineage.into(), kind.into());
        let row = self.life.entry(key).or_insert_with(|| LifeTally {
            bucket,
            first_tick: tick,
            last_tick: tick,
            process: process.into(),
            lineage: lineage.into(),
            kind: kind.into(),
            members: 0,
        });
        row.last_tick = tick;
        row.members += count;
        self.through = self.through.max(tick);
    }
}

pub(crate) struct Before {
    feed: bool,
    planning: bool,
    old_next: Id,
    living: [(Id, bool); 2],
}

impl Simulation {
    pub fn begin_food_diagnostic(&mut self, config: FoodDiagnosticConfig) -> crate::Result<()> {
        if config.bucket_ticks == 0 {
            return Err("food diagnostic needs a positive tick bucket".into());
        }
        if self.watch.diagnostic.is_some() {
            return Err("food diagnostic is already active".into());
        }
        self.watch.diagnostic = Some(Sink(Mutex::new(Capture::new(config, self.state.tick))));
        Ok(())
    }
    /// An owned immutable snapshot; no simulation record or RNG is touched.
    pub fn food_diagnostic(&self) -> Option<FoodDiagnosticReport> {
        self.watch.diagnostic.as_ref().map(|c| {
            let mut r = c.lock().report();
            r.through = self.state.tick;
            r
        })
    }
    /// Drain an interval outside an advance, retaining the capture settings.
    pub fn take_food_diagnostic(&mut self) -> Option<FoodDiagnosticReport> {
        let sink = self.watch.diagnostic.as_mut()?.get_mut();
        let mut out = sink.report();
        out.through = self.state.tick;
        *sink = Capture::new(sink.config.clone(), self.state.tick);
        Some(out)
    }
    pub fn end_food_diagnostic(&mut self) -> Option<FoodDiagnosticReport> {
        self.watch.diagnostic.take().map(|c| {
            let mut r = c.lock().report();
            r.through = self.state.tick;
            r
        })
    }
    pub(crate) fn diagnostic_begin(
        &self,
        actor: Id,
        target: Option<Id>,
        p: Option<&Process>,
        count: u64,
        phase: &str,
    ) -> Option<Before> {
        let mut capture = self.watch.diagnostic.as_ref()?.lock();
        let p = p?;
        let feed = p
            .commitments
            .iter()
            .chain(&p.effects)
            .chain(p.risk.iter().flat_map(|r| &r.effects))
            .any(|e| {
                matches!(e, Effect::Eat { .. })
                    || e.branches().any(|b| matches!(b, Effect::Eat { .. }))
            });
        if feed {
            capture.active.push(FeedAttempt {
                tick: self.state.tick,
                process: p.id.clone(),
                actor,
                lineage: self
                    .body_at_start(actor)
                    .map_or_else(String::new, |e| e.lineage.clone()),
                target,
                target_lineage: target
                    .and_then(|id| self.body_at_start(id))
                    .map(|e| e.lineage.clone()),
                target_alive: target
                    .and_then(|id| self.body_at_start(id))
                    .map(|e| e.alive),
                count,
                phase: phase.into(),
                accepted: false,
                stage: "entry".into(),
                outcome: String::new(),
                meals: vec![],
            });
        }
        let planning = phase == "plan";
        let alive = |id| !planning && self.state.population.get(id).is_some_and(|e| e.alive);
        let other = target.filter(|id| *id != actor);
        let living = [
            (actor, alive(actor)),
            (other.unwrap_or(actor), other.is_some_and(alive)),
        ];
        Some(Before {
            feed,
            planning,
            old_next: self.state.population.next_id,
            living,
        })
    }
    pub(crate) fn diagnostic_finish(&self, before: Option<Before>, receipt: &Receipt) {
        let Some(before) = before else { return };
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut capture = sink.lock();
        if before.feed
            && let Some(act) = capture.active.pop()
        {
            capture.record(act, receipt);
        }
        if before.planning || !receipt.accepted() {
            return;
        }
        for (id, alive) in before.living {
            if alive && let Some(e) = self.state.population.get(id).filter(|e| !e.alive) {
                capture.life(
                    receipt.tick,
                    &receipt.chosen,
                    &e.lineage,
                    "death",
                    receipt.count,
                );
            }
        }
        for (_, group) in self.state.population.groups.range(before.old_next..) {
            capture.life(
                receipt.tick,
                &receipt.chosen,
                &group.entity.lineage,
                "birth",
                group.count,
            );
        }
    }
    pub(crate) fn diagnostic_stage(&self, stage: &str) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut() {
            a.stage = stage.into();
        }
    }
    pub(crate) fn diagnostic_meal(&self, meal: MealObservation) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut() {
            a.meals.push(meal);
        }
    }
    pub(crate) fn diagnostic_reservation(
        &self,
        spec: &crate::schedule::MealSpec,
        portion: &Ledger,
    ) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut()
            && let Some(m) = a.meals.last_mut()
        {
            m.reserved = Some(portion.clone());
            m.planned_at = spec.at.clone();
            m.planned_part = spec.part.and_then(|h| match h {
                crate::flows::Holder::Part(_, id) => Some(id),
                _ => None,
            });
        }
    }
    pub(crate) fn diagnostic_meal_update(
        &self,
        offered: Option<&Ledger>,
        quota: Option<Option<&Ledger>>,
        taken: Option<&Ledger>,
    ) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut()
            && let Some(m) = a.meals.last_mut()
        {
            if let Some(v) = offered {
                m.offered = v.clone();
            }
            if let Some(v) = quota {
                m.quota = v.cloned();
            }
            if let Some(v) = taken {
                m.taken = v.clone();
            }
        }
    }
    pub(crate) fn diagnostic_given(&self, given: &Ledger) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut()
            && let Some(m) = a.meals.last_mut()
        {
            m.given = given.clone();
        }
    }
    pub(crate) fn diagnostic_partition(&self, before: u64, after: u64, given: &Ledger) {
        let Some(sink) = self.watch.diagnostic.as_ref() else {
            return;
        };
        let mut c = sink.lock();
        if let Some(a) = c.active.last_mut()
            && let Some(m) = a.meals.last_mut()
        {
            m.ingesta_before = Some(before);
            m.ingesta_after = Some(after);
            m.given = given.clone();
        }
    }
    pub(crate) fn diagnostic_active(&self) -> bool {
        self.watch
            .diagnostic
            .as_ref()
            .is_some_and(|c| !c.lock().active.is_empty())
    }
    pub(crate) fn diagnostic_checkpoint(&mut self) {
        if let Some(sink) = self.watch.diagnostic.as_mut() {
            let c = sink.get_mut();
            c.kept = None;
            c.kept = Some(Box::new(c.clone()));
        }
    }
    pub(crate) fn diagnostic_rollback(&mut self) {
        if let Some(sink) = self.watch.diagnostic.as_mut() {
            let c = sink.get_mut();
            if let Some(kept) = c.kept.take() {
                *c = *kept;
            }
        }
    }
}
