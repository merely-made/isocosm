// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each panel reads out of the one `Session` inside `SceneModel`.
//!
//! Projections only: nothing here applies an intent or mutates the model, so a
//! panel can be rebuilt any number of times per frame without touching the
//! world. The subject sheet is `SubjectSheet::from_input` over the played
//! subject's admitted anatomy, exactly as `body_sheet::equipment_session`
//! builds one, but read from this session rather than that private fixture.

use mesocosm_core::PartId;
use paredros_identity::SubjectId;
use paredros_world::{
    AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment, ItemKind, ItemLocation, MOTION_SCALE,
    SubjectSheet, SubjectSheetInput, TechniqueInputs, TechniqueKnowledge,
};

use super::SessionApp;

/// One carried or worn item, as the equipment panel lists it.
pub(super) struct Carried {
    pub item: paredros_world::ItemId,
    pub kind: ItemKind,
    /// The part it is attached to, when it is worn rather than carried.
    pub attached: Option<PartId>,
}

impl SessionApp {
    /// The played subject's sheet: admitted anatomy, no invented abilities.
    ///
    /// `actions` and `resources` are cleared for the same reason
    /// `equipment_session::sheet` clears them: this fixture teaches no
    /// techniques, so an action row here would be a claim the world does not
    /// make. Staleness and death stay as blockers.
    pub(super) fn sheet(&self) -> Option<SubjectSheet> {
        let model = self.model.borrow();
        let game = model.game();
        let played = model.played();
        let record = game.anatomies().get(played)?;
        let body = game.bodies().get(played)?;
        let knowledge = TechniqueKnowledge {
            subject: played,
            learned: Vec::new(),
        };
        let inputs = TechniqueInputs {
            occupied_parts: Vec::new(),
            part_capabilities: Vec::new(),
            equipment: Vec::new(),
            resources: Vec::new(),
            environment: ArrestFallEnvironment {
                support_present: false,
                support_distance_voxels: 0,
                arrest_load_mg: 0,
                support_load_capacity_mg: 0,
                adhesive_surface: AdhesiveSurface::Unsuitable,
                adhesive_resource: AdhesiveResource::Exhausted,
            },
        };
        let mut sheet = SubjectSheet::from_input(SubjectSheetInput {
            subject: played,
            revision: record.revision,
            current_revision: body.revision,
            body: &record.document,
            knowledge: &knowledge,
            inputs: &inputs,
            part_names: paredros_world::fixtures::three_lives::PART_NAMES,
            selected_part: self.selected.and_then(|(subject, part)| {
                (subject == played).then_some(part)
            }),
        });
        sheet.actions.clear();
        sheet.resources.clear();
        sheet.global_blockers.clear();
        if record.revision != body.revision {
            sheet.global_blockers.push(format!(
                "Detailed anatomy revision {} is stale; current body revision is {}. Reconcile before attaching equipment.",
                record.revision.0, body.revision.0
            ));
        }
        if !body.alive() {
            sheet
                .global_blockers
                .push("The played subject is dead; this sheet is read-only.".to_owned());
        }
        Some(sheet)
    }

    /// Everything the played subject carries or wears.
    pub(super) fn carried(&self) -> Vec<Carried> {
        let model = self.model.borrow();
        let played = model.played();
        model
            .game()
            .items()
            .carried_by(played)
            .map(|item| Carried {
                item: item.id,
                kind: item.kind,
                attached: match item.location {
                    ItemLocation::Attached { part, .. } => Some(part),
                    _ => None,
                },
            })
            .collect()
    }

    /// Parts an item may be attached to: intact, and not the root.
    pub(super) fn attachable_parts(&self) -> Vec<PartId> {
        let model = self.model.borrow();
        let played = model.played();
        model
            .game()
            .anatomies()
            .get(played)
            .map(|record| {
                record
                    .document
                    .parts
                    .iter()
                    .filter(|part| !part.severed && part.attachment.is_some())
                    .map(|part| part.id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The same lines the timed-action host's text view prints.
    pub(super) fn status_lines(&self) -> Vec<String> {
        let model = self.model.borrow();
        let game = model.game();
        let played = model.played();
        let mut lines: Vec<String> = self.status.clone();
        match model.action().and_then(|action| action.action()) {
            Some(action) => {
                lines.push(format!(
                    "Action: {:?}  started={}  last_tick={}",
                    action.direction, action.started_at.0, action.last_tick.0
                ));
                for (part, contribution) in &action.contributors {
                    lines.push(format!(
                        "Part {}: {:?}, charge={}, node={}",
                        part.0, contribution.state, contribution.charge, contribution.node.0
                    ));
                }
            },
            None => lines.push("Action: idle".into()),
        }
        if let Some(pose) = game.movement().pose(played) {
            let at = pose.position.map(|v| v as f64 / MOTION_SCALE as f64);
            lines.push(format!(
                "Position {:.3}, {:.3}, {:.3}; step={} grounded={}",
                at[0], at[1], at[2], pose.step, pose.grounded
            ));
        }
        match game.movement_projection(played) {
            Ok(Some(projection)) => lines.push(format!(
                "Movement supports {}/{}; speed {:.2} voxels/s; clearance {:.2} wide x {:.2} high",
                projection.active_supports.len(),
                projection.declared_supports,
                projection.speed as f64 / MOTION_SCALE as f64,
                2.0 * projection.envelope.half_width as f64 / MOTION_SCALE as f64,
                projection.envelope.height as f64 / MOTION_SCALE as f64,
            )),
            Ok(None) => lines.push("Movement: legacy stance profile".into()),
            Err(error) => lines.push(format!("Movement support unavailable: {error:?}")),
        }
        drop(model);
        lines.extend(self.target_lines());
        lines
    }

    fn target_lines(&self) -> Vec<String> {
        let model = self.model.borrow();
        let game = model.game();
        let played = model.played();
        let Some(body) = game.bodies().get(self.target) else {
            return vec!["Target unavailable".into()];
        };
        let anatomy = game
            .current_anatomy(self.target)
            .ok()
            .or_else(|| game.anatomies().get(self.target));
        let parts = anatomy
            .map(|record| {
                record
                    .document
                    .parts
                    .iter()
                    .map(|part| {
                        format!(
                            "{}:{}",
                            part.id.0,
                            if part.severed { "severed" } else { "intact" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "stale anatomy".into());
        let item = game
            .items()
            .get(self.target_item)
            .map(|item| format!("{:?}", item.location))
            .unwrap_or_else(|| "missing".into());
        vec![
            format!(
                "Player at {:?}; vitality={} wound={} dressings={} lost parts={}",
                game.movement().position(played),
                game.bodies().get(played).map_or(0, |body| body.vitality),
                game.bodies().get(played).map_or(0, |body| body.wound),
                game.items()
                    .carried_by(played)
                    .filter(|item| item.kind == ItemKind::Dressing)
                    .count(),
                game.anatomies().get(played).map_or(0, |record| record
                    .document
                    .parts
                    .iter()
                    .filter(|part| part.severed)
                    .count())
            ),
            format!(
                "Target {} at {:?}; vitality={} {}",
                body.name.as_ref().map_or("unnamed", |name| name.as_str()),
                game.movement().position(self.target),
                body.vitality,
                body.died_at.map_or("alive", |_| "dead")
            ),
            format!("Target parts: {parts}"),
            format!("Target item {}: {item}", self.target_item.0),
        ]
    }

    /// The label under the viewport: who is framed, and what is selected.
    pub(super) fn selection_line(&self) -> String {
        match self.selected {
            Some((subject, part)) => format!("Selected subject {} part {}", subject.0, part.0),
            None => "Nothing selected. Click a body in the scene, or a part button.".to_owned(),
        }
    }

    pub(super) fn played(&self) -> SubjectId {
        self.model.borrow().played()
    }
}
