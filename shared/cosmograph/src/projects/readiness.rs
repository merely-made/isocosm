// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{Binding, Query};

impl Simulation {
    pub fn project_reservation_readiness(&self, actor:Id, target:Option<Id>, process:&str, lots:&[Key]) -> Result<String> {
        crate::society()?;
        if lots.is_empty() { return Err("reservation needs an explicit native lot selection".into()); }
        if !self.genesis.rules.processes.get(process).is_some_and(|p| p.requires.iter()
            .any(|q| matches!(q, Query::Project { phase:Phase::Reserve, .. }))) {
            return Err("process declares no native reservation".into());
        }
        let mut candidate = self.clone();
        candidate.project_inputs = Some((actor,lots.to_vec()));
        candidate.project_process_readiness(actor,target,process)
    }
    /// Conservative until an actual target is supplied for target-bound inputs.
    pub fn project_readiness(&self, actor: Id, definition: &str, phase: Phase) -> Result<String> {
        let mut refusal = "no authored native project process".into();
        for process in self.genesis.rules.processes.values().filter(|p| p.requires.iter().any(|q|
            matches!(q, Query::Project { definition: key, phase: held, .. } if key == definition && *held == phase))) {
            match self.project_process_readiness(actor, None, &process.id) {
                Ok(ready) => return Ok(ready),
                Err(why) => refusal = why,
            }
        }
        Err(refusal)
    }

    pub fn project_process_readiness(&self, actor: Id, target: Option<Id>, process: &str) -> Result<String> {
        let p = self.genesis.rules.processes.get(process).ok_or("project process is absent")?;
        if !p.requires.iter().any(|q| matches!(q, Query::Project { .. })) {
            return Err("process declares no native project operation".into());
        }
        let body = self.body_at_start(actor).ok_or("project actor is absent")?;
        if !self.route_ready(actor, p) || !self.target_matches(actor, target, p) {
            return Err("project route or target is unavailable".into());
        }
        self.bind_intake(actor, p)?;
        let part = self.bind_part(actor, p);
        let mut ready = String::new();
        for query in &p.requires { ready = self.query(actor, target, body.place, part, query)?; }
        Ok(ready)
    }

    pub(crate) fn project_ready(&self, actor: Id, target: Option<Id>, key: &str,
        phase: Phase, process: &str) -> Result<String> {
        let definition = self.project_definition(key)?;
        let body = self.body_at_start(actor).ok_or("project actor is absent")?;
        if !body.alive || body.resident_site().is_none()
            || !crate::social::Confidence::read(body, &definition.work).sufficient()
        { return Err("project actor lacks current craft or residence".into()); }
        let project = self.project_binding_at_start(actor).and_then(|id| self.project_at_start(id));
        match phase {
            Phase::Reserve => {
                if project.is_some_and(|p| p.output.is_none()) {
                    return Err("project actor already has selected unfinished work".into());
                }
                let p = self.genesis.rules.processes.get(process).ok_or("project process is absent")?;
                let (from, authored) = reserve_bindings(p, key)?;
                let stocks = match from {
                    Binding::Place => self.site_at_start(body.resident_site().expect("checked above"))
                        .map(|s| s.accounts.clone()).ok_or("project stock site is absent")?,
                    Binding::Actor | Binding::Target => {
                        let id = self.bound(actor, target, from)?;
                        crate::anatomy::books(self.body_at_start(id).ok_or("project stock body is absent")?)
                    },
                    Binding::Part => return Err("project source needs an addressed native ledger".into()),
                };
                let recipe = self.genesis.rules.material_recipes.get(&definition.recipe).ok_or("project recipe is absent")?;
                let selected = self.selected_project_lots(actor, &authored)?;
                self.check_project_lots(self.stock_source(actor, target, from)?, &selected, &recipe.stocks)?;
                if recipe.stocks.iter().any(|(key, amount)| stocks.get(key).copied().unwrap_or(0) < *amount) {
                    return Err("project source lacks exact reserved recipe stocks".into());
                }
                let catalogue = self.genesis.rules.projects.as_ref().ok_or("project catalogue is absent")?;
                if self.state.projects.records.len() as u64 >= catalogue.limit { return Err("project limit".into()); }
                Ok(format!("project reservation ready: {key}"))
            },
            Phase::Work => {
                let project = project.ok_or("no project is selected")?;
                if project.definition != key || project.recipe != definition.recipe
                    || usize::try_from(project.steps).ok().and_then(|i| definition.steps.get(i)).is_none()
                { return Err("selected project has no matching unfinished work step".into()); }
                self.project_eligible(actor, project)?;
                let step = &definition.steps[project.steps as usize];
                if project.carrier.is_some() && project.progress.checked_add(step.progress) == Some(definition.total)
                    && definition.output_custody.is_none() {
                    return Err("carried final work needs its declared native output custody relation".into());
                }
                Ok(format!("project work ready: {key}, progress {}", project.progress))
            },
        }
    }
}
