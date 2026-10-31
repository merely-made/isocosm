// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Opt-in test observation before the scheduled advance clears its cache.

use super::*;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub(crate) struct Attempt {
    pub phase: &'static str,
    pub tick: Tick,
    pub actor: Id,
    pub site: Id,
    pub pass: Key,
    pub mood: i64,
    pub choice: Choice,
    pub actor_held: u128,
    pub selected_target: serde_json::Value,
    pub food_due: bool,
    pub food_target: Option<Id>,
    pub food_feasible: bool,
}

struct Recording { lineage: Key, food: Key, attempts: Vec<Attempt> }
thread_local! { static RECORDING: RefCell<Option<Recording>> = const { RefCell::new(None) }; }

pub(crate) fn start(lineage: &str, food: &str) {
    RECORDING.with(|slot| *slot.borrow_mut() = Some(Recording {
        lineage: lineage.into(), food: food.into(), attempts: vec![] }));
}

pub(crate) fn take() -> Vec<Attempt> {
    RECORDING.with(|slot| slot.borrow_mut().as_mut()
        .map(|record| std::mem::take(&mut record.attempts)).unwrap_or_default())
}

pub(crate) fn stop() { RECORDING.with(|slot| *slot.borrow_mut() = None); }

impl Simulation {
    pub(super) fn record_choice_attempt(&self, actor: Id, pass: &str, choice: &Choice) {
        self.record_choice_stage(actor, pass, choice, "choice");
    }

    pub(crate) fn record_food_pass(&self, pass: &str) {
        let interest = RECORDING.with(|slot| slot.borrow().as_ref().map(|r| (r.lineage.clone(), r.food.clone())));
        let Some((lineage, food)) = interest.filter(|(_, food)| food == pass) else { return; };
        for (first, group) in &self.state.population.groups {
            if group.entity.alive && group.entity.lineage == lineage {
                for actor in *first..*first + group.count {
                    let choice = self.chosen.by.get(&actor).cloned().unwrap_or_default();
                    self.record_choice_stage(actor, &food, &choice, "food-pass-start");
                }
            }
        }
    }

    pub(crate) fn record_food_outcome(&self, actor: Id, pass: &str, accepted: bool) {
        let interested = RECORDING.with(|slot| slot.borrow().as_ref().is_some_and(|r| r.food == pass));
        if interested {
            let choice = self.chosen.by.get(&actor).cloned().unwrap_or_default();
            self.record_choice_stage(actor, pass, &choice, if accepted { "food-accepted" } else { "food-blocked" });
        }
    }

    fn record_choice_stage(&self, actor: Id, pass: &str, choice: &Choice, phase: &'static str) {
        let interest = RECORDING.with(|slot| slot.borrow().as_ref()
            .map(|record| (record.lineage.clone(), record.food.clone())));
        let Some((lineage, food)) = interest else { return; };
        let Some(body) = self.body_at_start(actor).filter(|body| body.lineage == lineage) else { return; };
        let Some(site) = body.resident_site() else { return; };
        let process = &self.genesis.rules.processes[&food];
        let target = self.choose_target(actor, process);
        let attempt = Attempt { phase, tick: self.state.tick, actor, site, pass: pass.into(),
            mood: self.choice_minded(actor).0, choice: choice.clone(),
            actor_held: crate::meaning::mass(&crate::anatomy::books(body), &self.genesis.rules),
            selected_target: choice.target.and_then(|id| self.body_at_start(id).map(|target| serde_json::json!({
                "id": id, "lineage": target.lineage, "alive": target.alive,
                "place": target.place, "resident_site": target.resident_site(), "journey": target.journey,
                "held": crate::meaning::mass(&crate::anatomy::books(target), &self.genesis.rules)
            }))).unwrap_or(serde_json::Value::Null),
            food_due: process.period.is_some_and(|period| self.state.tick % period == 0),
            food_target: target, food_feasible: self.feasible(actor, target, process, site) };
        RECORDING.with(|slot| {
            if let Some(record) = slot.borrow_mut().as_mut() { record.attempts.push(attempt); }
        });
    }
}
