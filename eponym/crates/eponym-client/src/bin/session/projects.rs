// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored material actions reach the same controlled Session as movement.

use cambium::{el, text};
use eponym_play::{GameEvent, GameIntent};
use isocosm::projects::{Phase, ProjectId};
use isocosm::rules::{Effect, Query};
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
    if let Some(item) = state.project_target {
        rows.push(Box::new(el("p", text(format!("Selected material: item {}", item.0)))));
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
                || !project.authorized.contains(&actor) || site != Some(project.site)
            { continue; }
            rows.push(Box::new(el("p", text(format!("{}: {} / {}",
                label(&project.recipe), project.progress, definition.total)))));
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
    pub(super) fn use_for_work(&mut self, item: eponym_play::ItemId) {
        self.project_target = Some(item);
        self.status = vec![format!("Item {} selected for material work", item.0)];
    }
    fn material_act(&mut self, process: String, project: Option<ProjectId>) {
        let intent = {
            let model = self.model.borrow();
            let game = model.game();
            GameIntent::ProjectAct {
                tick: game.next_tick(),
                subject: model.played(),
                target: self.project_target.map(|item| item.0)
                    .or_else(|| self.selected.and_then(|(subject, _)| game.entity(subject))),
                process,
                project,
            }
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
