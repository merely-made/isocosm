// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::{Project, ProjectId};

impl Staged<'_> {
    fn project_bind(&mut self, id: ProjectId) {
        let actor = self.stage.actor;
        self.stage.projects.binding_before.entry(actor)
            .or_insert_with(|| self.sim.project_binding_at_start(actor));
        self.stage.projects.bindings.insert(actor, id);
    }

    pub(super) fn reserve_project(&mut self, key: &str, from: Binding, authority: Binding) -> Result<()> {
        if self.stage.count != 1 || from == Binding::Part {
            return Err("project reservation requires one worker and an addressed ledger".into());
        }
        let actor = self.stage.actor;
        let worker = &self.stage.bodies[&actor];
        let site = worker.resident_site().ok_or("project worker has no resident site")?;
        let definition = self.sim.project_definition(key)?.clone();
        if !worker.alive || !crate::social::Confidence::read(worker, &definition.work).sufficient() {
            return Err("project reservation lacks current craft eligibility".into());
        }
        let authority = self.sim.bound(actor, self.stage.target, authority)?;
        if !self.stage.bodies.get(&authority).is_some_and(|e| e.alive && !crate::directing::is_participant(e)) {
            return Err("project reservation names no native subject authority".into());
        }
        let id = ProjectId(self.stage.act);
        if self.stage.projects.records.contains_key(&id) || self.sim.state.projects.records.contains_key(&id) {
            return Err("one native action cannot overwrite or reserve two projects".into());
        }
        let created = self.stage.projects.before.values().filter(|p| p.is_none()).count() as u64;
        let catalogue = self.sim.genesis.rules.projects.as_ref().ok_or("project catalogue is absent")?;
        if (self.sim.state.projects.records.len() as u64).checked_add(created)
            .is_none_or(|count| count >= catalogue.limit) {
            return Err("project limit".into());
        }
        let recipe = self.sim.genesis.rules.material_recipes.get(&definition.recipe)
            .ok_or("project material recipe is absent")?;
        let stocks = recipe.stocks.clone();
        let source = self.holder(from).ok_or("project input source is absent")?;
        for (account, amount) in &stocks {
            if matches!(from, Binding::Place | Binding::Target) {
                if let Some(demands) = &mut self.stage.demands {
                    demands.accounts.push((source, account.clone(), *amount));
                }
                if let Some(shares) = &mut self.stage.shares {
                    let share = shares.accounts.entry((source, account.clone())).or_default();
                    if *share < *amount { return Err("project reservation lacks its complete native share".into()); }
                    *share -= *amount;
                }
            }
            self.take(from, account, *amount)?;
            if self.sim.flowing() { self.stage.legs.push(Leg {
                from: (source, account.clone()), to: (Holder::ProjectReservation(id), account.clone()), amount: *amount,
            }); }
        }
        self.stage.projects.before.insert(id, None);
        self.stage.projects.records.insert(id, Project {
            definition: key.into(), recipe: definition.recipe, maker: actor, authority, site,
            reserved: stocks, construction: Ledger::new(), progress: 0, steps: 0,
            authorized: BTreeSet::new(), output: None,
        });
        self.project_bind(id);
        Ok(())
    }

    pub(super) fn work_project(&mut self) -> Result<()> {
        if self.stage.count != 1 { return Err("project work requires one admitted worker".into()); }
        let actor = self.stage.actor;
        let id = self.stage.projects.bindings.get(&actor).copied()
            .or_else(|| self.sim.project_binding_at_start(actor)).ok_or("no project is selected")?;
        if !self.stage.projects.records.contains_key(&id) {
            let project = self.sim.project_at_start(id).ok_or("selected project is absent")?.clone();
            self.stage.projects.before.insert(id, Some(project.clone()));
            self.stage.projects.records.insert(id, project);
        }
        let mut project = self.stage.projects.records[&id].clone();
        let definition = self.sim.project_definition(&project.definition)?.clone();
        let worker = &self.stage.bodies[&actor];
        if !worker.alive || worker.resident_site() != Some(project.site) || project.output.is_some()
            || !project.authorized.contains(&actor)
            || !crate::social::Confidence::read(worker, &definition.work).sufficient() {
            return Err("project work lacks current craft, residence or native authorization".into());
        }
        let step = usize::try_from(project.steps).ok().and_then(|i| definition.steps.get(i))
            .ok_or("project has no unfinished authored step")?;
        for (account, amount) in &step.spend {
            debit(&mut project.reserved, account, *amount)?;
            credit(&mut project.construction, account, *amount)?;
            if self.sim.flowing() { self.stage.legs.push(Leg {
                from: (Holder::ProjectReservation(id), account.clone()),
                to: (Holder::ProjectConstruction(id), account.clone()), amount: *amount,
            }); }
        }
        project.reserved.retain(|_, amount| *amount > 0);
        project.steps = project.steps.checked_add(1).ok_or("project step overflow")?;
        project.progress = project.progress.checked_add(step.progress).ok_or("project progress overflow")?;
        if project.progress == definition.total {
            if !project.reserved.is_empty() || project.steps != definition.steps.len() as u64 {
                return Err("project completion retains unspent reservation or unfinished steps".into());
            }
            let rule = self.sim.genesis.rules.material_recipes.get(&project.recipe).ok_or("project recipe is absent")?;
            let output = rule.output(&self.sim.genesis.rules, &self.sim.state, actor, project.site, &project.construction)?;
            let count = self.sim.state.population.count().checked_add(self.stage.births.len() as u64)
                .and_then(|count| count.checked_add(1)).ok_or("project output count overflow")?;
            let next = self.sim.state.population.next_id.checked_add(self.stage.births.len() as u64)
                .filter(|id| id.checked_add(1).is_some()).ok_or("project output identity exhausted")?;
            if count > self.sim.genesis.rules.limits.entities { return Err("project output entity limit".into()); }
            if self.sim.flowing() { for (account, amount) in &project.construction { self.stage.legs.push(Leg {
                from: (Holder::ProjectConstruction(id), account.clone()),
                to: (Holder::Entity(next), account.clone()), amount: *amount,
            }); } }
            project.construction.clear();
            project.output = Some(next);
            self.stage.births.push(output);
        }
        self.stage.projects.records.insert(id, project);
        Ok(())
    }
}
