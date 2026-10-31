// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored material actions reach the same controlled Session as movement.

use cambium::{el, text};
use eponym_play::{GameEvent, GameIntent};
use cosmograph::projects::{Phase, ProjectId, StockSource};
use cosmograph::rules::{Effect, Query};
use super::{SessionApp, view::{Child, button}};

fn label(key: &str) -> &str { key.split_once(':').map_or(key, |(_, word)| word) }

pub(super) fn panel(state: &SessionApp) -> Child {
    let model = state.model.borrow();
    let game = model.game();
    let sim = game.world().sim();
    let Some(catalogue) = &sim.genesis().rules.projects else {
        return Box::new(el("aside", text("This world declares no material work.")).attr("class", "panel"));
    };
    let Some(actor) = game.entity(model.played()) else {
        return Box::new(el("aside", text("Choose a living subject to work.")).attr("class", "panel"));
    };
    let site = sim.state().population.get(actor).and_then(|body| body.resident_site());
    let mut rows: Vec<Child> = Vec::new();
    rows.extend(super::project_permissions::rows(state));
    if let Some(item) = state.project_target {
        rows.push(Box::new(el("p", text(format!("Selected material: item {}", item.0)))));
    }
    let selected = state.project_target.map(|item| item.0)
        .or_else(|| state.selected.and_then(|(subject, _)| game.entity(subject)));
    for (_, lot) in sim.available_lots().filter(|(source, _)| match *source {
        StockSource::Entity(id) => id == actor || Some(id) == selected,
        StockSource::Site(id) => Some(id) == site,
        StockSource::Place(_) | StockSource::Transit(_) => false,
    }) {
        let key = lot.key.clone();
        rows.push(button(format!("{} {}: {}mg {}",
            if state.project_lots.contains(&key) { "Selected" } else { "Select" },
            label(&key), lot.amount, label(&lot.account)), "control",
            move |state| state.select_lot(key.clone())));
    }
    for (key, definition) in &catalogue.definitions {
        for process in sim.genesis().rules.processes.values().filter(|process|
            process.requires.iter().any(|query| matches!(query,
                Query::Project { definition, phase:Phase::Reserve, .. } if definition == key))
            && process.effects.iter().any(|effect|
                matches!(effect, Effect::ReserveProject { definition, .. } if definition == key)))
        {
            let process = process.id.clone();
            rows.push(button(format!("Start {}", label(&definition.recipe)), "control",
                move |state| state.material_act(process.clone(), None)));
        }
        for (&id, project) in &sim.state().projects.records {
            if project.definition != *key || project.output.is_some()
                || !sim.project_authorized(project, actor) || site != sim.project_site(project)
            { continue; }
            let Ok(location) = game.project_location(id) else { continue; };
            let (work_site, at) = location.position();
            if site != Some(work_site) || game.movement().position(model.played()) != Some(at) { continue; }
            rows.push(Box::new(el("p", text(format!("{}: {} / {}",
                label(&project.recipe), project.progress, definition.total)))));
            let carrying = project.carrier == Some(actor);
            rows.push(button(if carrying { "Put work down" } else { "Carry work" }.into(), "control",
                move |state| state.carry_work(id, !carrying)));
            for process in sim.genesis().rules.processes.values().filter(|process|
                process.requires.iter().any(|query| matches!(query,
                    Query::Project { definition, phase:Phase::Work, .. } if definition == key))
                && process.effects.iter().any(|effect|
                    matches!(effect, Effect::WorkProject)))
            {
                let process = process.id.clone();
                rows.push(button(format!("Work on {}", label(&project.recipe)), "control",
                    move |state| state.material_act(process.clone(), Some(id))));
            }
        }
    }
    if rows.is_empty() { rows.push(Box::new(el("p", text("No authored work is available here.")))); }
    Box::new(el("aside", (el("h2", text("Material work")), el("div", rows))).attr("class", "panel"))
}

impl SessionApp {
    fn select_lot(&mut self, key:String) {
        if !self.project_lots.remove(&key) { self.project_lots.insert(key); }
    }
    fn carry_work(&mut self, project:ProjectId, carried:bool) {
        let intent = {
            let model = self.model.borrow();
            GameIntent::ProjectCarry { tick:model.game().next_tick(),
                subject:model.played(), project, carried }
        };
        self.status = vec![match self.model.borrow_mut().apply(intent) {
            Ok(_) => if carried { "Work is now carried." } else { "Work is placed here." }.into(),
            Err(error) => format!("Moving work refused: {error:?}"),
        }];
    }
    pub(super) fn use_for_work(&mut self, item: eponym_play::ItemId) {
        self.project_target = Some(item);
        self.project_lots.clear();
        self.status = vec![format!("Item {} selected for material work", item.0)];
    }
    fn material_act(&mut self, process: String, project: Option<ProjectId>) {
        let intent = {
            let model = self.model.borrow();
            let game = model.game();
            let target = self.project_target.map(|item| item.0)
                .or_else(|| self.selected.and_then(|(subject, _)| game.entity(subject)));
            if project.is_none() { GameIntent::ProjectReserve {
                tick:game.next_tick(), subject:model.played(), process, target,
                lots:self.project_lots.iter().cloned().collect(),
            }} else { GameIntent::ProjectAct {
                tick: game.next_tick(),
                subject: model.played(),
                target,
                process,
                project,
            }}
        };
        self.status = match self.model.borrow_mut().apply(intent) {
            Ok(events) => events.iter().filter_map(|event| match event {
                GameEvent::ProjectAct { receipt, output, .. } => Some(match output {
                    Some(item) => format!("Work complete: item {}", item.0),
                    None => format!("Work accepted: {:?}", receipt.outcome),
                }),
                _ => None,
            }).collect(),
            Err(error) => vec![format!("Work refused: {error:?}")],
        };
    }
}
