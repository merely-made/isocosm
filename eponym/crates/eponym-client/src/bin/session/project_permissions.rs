// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Every input owner supplies its own decision for the explicitly selected worker.

use cambium::{el, text};
use eponym_play::{GameEvent, GameIntent};
use eponym_play::identity::SubjectId;
use isocosm::projects::{Owner, ProjectId};
use super::{SessionApp, view::{Child, button}};

pub(super) fn rows(state:&SessionApp) -> Vec<Child> {
    let model = state.model.borrow();
    let game = model.game();
    let sim = game.world().sim();
    let Some(actor) = game.entity(model.played()) else { return vec![]; };
    let worker = state.selected.map(|(subject, _)| subject);
    let mut rows = Vec::new();
    for (&id, project) in &sim.state().projects.records {
        if project.output.is_some() { continue; }
        for &owner in &project.owners {
            let Ok(process) = sim.project_consent_ready(project, owner, actor) else { continue; };
            let Some(worker) = worker else {
                rows.push(Box::new(el("p", text("Select a subject to decide its work permission."))) as Child);
                continue;
            };
            for grant in [true, false] {
                let process = process.clone();
                rows.push(button(format!("{} project {} work for subject {}",
                    if grant { "Allow" } else { "Withdraw" }, id.0, worker.0), "control",
                    move |state| state.decide_work(id, owner, worker, grant, process.clone())));
            }
        }
    }
    rows
}

impl SessionApp {
    fn decide_work(&mut self, project:ProjectId, owner:Owner, worker:SubjectId,
        grant:bool, process:Option<String>) {
        let intent = {
            let model = self.model.borrow();
            GameIntent::ProjectConsent { tick:model.game().next_tick(), subject:model.played(),
                project, owner, worker, grant, process }
        };
        self.status = match self.model.borrow_mut().apply(intent) {
            Ok(events) => events.iter().filter_map(|event| match event {
                GameEvent::ProjectConsented { authorized, .. } => Some(
                    if *authorized == grant {
                        if grant { "Joint permission granted." } else { "Joint permission withdrawn." }
                    } else { "Decision recorded; joint approval pending." }.into()),
                _ => None,
            }).collect(),
            Err(error) => vec![format!("Work permission refused: {error:?}")],
        };
    }
}
