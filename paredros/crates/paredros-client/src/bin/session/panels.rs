// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each panel reads out of the one `Session` inside `SceneModel`.
//!
//! Projections only: nothing here applies an intent or mutates the model, so a
//! panel can be rebuilt any number of times per frame without touching the
//! world. The subject sheet is `SubjectSheet::from_input` over the played
//! subject's admitted anatomy.
//!
//! [`subject_sheet`] is the surviving copy of that projection. The body
//! sheet's private `EquipmentSession` built the same one over its own Keeper
//! fixture, and was retired with it (M5 of the isomere plan); its four
//! projection claims — a geometry-preserving sheet with no invented
//! abilities, a stale anatomy that stays inspectable and says so, a
//! reconciled severance that shows, and a dead subject that says so — moved
//! into this module's tests, against this session's own world. The world
//! rules underneath them are proved where they are owned, in
//! `paredros-world/tests/equipment.rs`.

use isometer::core::PartId;
use paredros_identity::SubjectId;
use paredros_world::CanonRevisionCause;
use paredros_world::glyphs::ProvenanceKind;
use paredros_world::{
    AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment, GameState, ItemKind, ItemLocation,
    MOTION_SCALE, SubjectSheet, SubjectSheetInput, TechniqueInputs, TechniqueKnowledge,
};

use super::SessionApp;

/// One subject's sheet: admitted anatomy, no invented abilities.
///
/// `actions` and `resources` are cleared because this fixture teaches no
/// techniques, so an action row here would be a claim the world does not make.
/// Staleness and death stay as blockers. `selected_part` is what the sheet
/// highlights, and nothing else reaches the projection.
pub(super) fn subject_sheet(
    game: &GameState,
    subject: SubjectId,
    selected_part: Option<PartId>,
) -> Option<SubjectSheet> {
    let record = game.anatomies().get(subject)?;
    let body = game.bodies().get(subject)?;
    let knowledge = TechniqueKnowledge {
        subject,
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
        subject,
        revision: record.revision,
        current_revision: body.revision,
        body: &record.document,
        knowledge: &knowledge,
        inputs: &inputs,
        part_names: paredros_world::fixtures::three_lives::PART_NAMES,
        selected_part,
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

/// One carried or worn item, as the equipment panel lists it.
pub(super) struct Carried {
    pub item: paredros_world::ItemId,
    pub kind: ItemKind,
    /// The part it is attached to, when it is worn rather than carried.
    pub attached: Option<PartId>,
}

impl SessionApp {
    /// The played subject's sheet, with this session's own selection.
    pub(super) fn sheet(&self) -> Option<SubjectSheet> {
        let model = self.model.borrow();
        let played = model.played();
        subject_sheet(
            model.game(),
            played,
            self.selected
                .and_then(|(subject, part)| (subject == played).then_some(part)),
        )
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

/// One acquired glyph, as the acquisition journal lists it.
pub(super) struct JournalRow {
    pub glyph: String,
    /// The canon's display mark. Opaque text, never an identity.
    pub display: String,
    /// What this glyph meant when it was acquired. This never moves.
    pub effect: String,
    /// What it means under the live revision, when that is not the same, with
    /// the cause the world published the revision for.
    pub live_effect: Option<String>,
    pub cause: Option<String>,
    /// The accepted event kind that was the evidence.
    pub kind: String,
    pub tick: u64,
}

impl SessionApp {
    /// The journal in first-acquisition order. A projection of the reading:
    /// drawing it grants nothing and reads no world fact.
    pub(super) fn journal(&self) -> Vec<JournalRow> {
        let Some(reading) = self.glyphs.as_ref() else {
            return Vec::new();
        };
        let canon = reading.canon();
        let cause = reading.revision().map(|live| describe_cause(&live.cause));
        reading
            .journey()
            .acquisitions()
            .iter()
            .map(|acquisition| JournalRow {
                glyph: acquisition.glyph.clone(),
                display: canon
                    .spec()
                    .glyphs
                    .iter()
                    .find(|glyph| glyph.id == acquisition.glyph)
                    .map(|glyph| glyph.display.clone())
                    .unwrap_or_default(),
                effect: canon
                    .effect(&acquisition.glyph)
                    .unwrap_or("unknown")
                    .to_owned(),
                live_effect: reading
                    .live_effect(&acquisition.glyph)
                    .filter(|live| Some(*live) != canon.effect(&acquisition.glyph))
                    .map(str::to_owned),
                cause: cause.clone(),
                kind: match &acquisition.provenance.kind {
                    ProvenanceKind::Custom(kind) => kind.clone(),
                    other => format!("{other:?}").to_lowercase(),
                },
                tick: acquisition.tick,
            })
            .collect()
    }

    /// How many of the canon's glyphs are held, and what that makes eligible.
    pub(super) fn journal_summary(&self) -> String {
        let Some(reading) = self.glyphs.as_ref() else {
            return "Acquisition journal unavailable.".to_owned();
        };
        let eligibility = reading.eligibility();
        format!(
            "{} of {} acquired · {} evidence records · {}{}",
            reading.journey().acquisitions().len(),
            reading.canon().spec().glyphs.len(),
            reading.records().len(),
            if eligibility.complete {
                "canon complete"
            } else {
                "canon incomplete"
            },
            if reading.ended() { " · ended" } else { "" }
        )
    }

    /// The live canon revision the reading answers from.
    pub(super) fn canon_revision(&self) -> Option<u64> {
        Some(self.glyphs.as_ref()?.live_canon().spec().revision)
    }

    /// What the last acquired glyph means now, for `glyph-last-live-effect`.
    pub(super) fn last_live_effect(&self) -> Option<String> {
        let reading = self.glyphs.as_ref()?;
        reading.live_effect(&self.last_glyph()?).map(str::to_owned)
    }

    /// The last glyph acquired, for the scenario's `glyph-last` reading.
    pub(super) fn last_glyph(&self) -> Option<String> {
        self.glyphs
            .as_ref()?
            .journey()
            .acquisitions()
            .last()
            .map(|acquisition| acquisition.glyph.clone())
    }
}

/// One published revision's cause, in the panel's own words.
fn describe_cause(cause: &CanonRevisionCause) -> String {
    match cause {
        CanonRevisionCause::Period {
            metric_id,
            end_tick,
        } => format!("period {metric_id} ended tick {}", end_tick.0),
        CanonRevisionCause::Promotion { promotion_id, .. } => format!("promotion {promotion_id}"),
        CanonRevisionCause::Authored { label } => format!("authored {label}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paredros_identity::BodyRevisionId;
    use paredros_world::GameIntent;
    use paredros_world::fixtures::session as session_fixture;

    /// The session's own world, detached from its `Session` so a test can
    /// apply the world facts the projection has to survive.
    fn played_game() -> (GameState, SubjectId) {
        let fixture = session_fixture::timed_action_world();
        let played = fixture.keeper;
        (fixture.action.session().game().clone(), played)
    }

    fn fall(game: &mut GameState, subject: SubjectId, distance: i32) {
        let tick = game.next_tick();
        game.apply(GameIntent::Fall {
            tick,
            subject,
            distance,
        })
        .expect("a fall is an accepted world fact");
    }

    #[test]
    fn the_sheet_keeps_admitted_geometry_without_inventing_abilities() {
        let (game, played) = played_game();
        let admitted = game.anatomies().get(played).expect("admitted anatomy");
        let sheet = subject_sheet(&game, played, None).expect("sheet for the played subject");

        assert_eq!(sheet.subject, played);
        assert_eq!(sheet.revision, admitted.revision);
        assert_eq!(sheet.parts.len(), admitted.document.parts.len());
        assert!(sheet.parts.iter().all(|part| part.bounds.is_some()));
        assert!(sheet.parts.iter().all(|part| part.capabilities.is_empty()));
        assert!(sheet.learned.is_empty());
        assert!(sheet.actions.is_empty());
        assert!(sheet.resources.is_empty());
        assert!(sheet.global_blockers.is_empty());
    }

    #[test]
    fn a_stale_anatomy_stays_inspectable_and_says_so() {
        let (mut game, played) = played_game();
        fall(&mut game, played, 5);
        let body = game.bodies().get(played).expect("body");
        assert!(body.alive());
        assert_ne!(
            body.revision,
            game.anatomies().get(played).expect("anatomy").revision
        );

        let before = game.state_hash().expect("hash");
        let sheet = subject_sheet(&game, played, None).expect("sheet");
        assert_eq!(game.state_hash().expect("hash"), before);
        assert!(!sheet.parts.is_empty());
        assert!(sheet.parts.iter().all(|part| part.bounds.is_some()));
        assert!(
            sheet
                .global_blockers
                .iter()
                .any(|line| line.contains("stale"))
        );
    }

    #[test]
    fn a_reconciled_severance_shows_on_the_sheet_and_clears_the_blocker() {
        let (mut game, played) = played_game();
        let old = game.bodies().get(played).expect("body").revision;
        fall(&mut game, played, 5);
        let tick = game.next_tick();
        game.apply(GameIntent::ReconcileAnatomy {
            tick,
            subject: played,
            from_revision: old,
            revision: BodyRevisionId(old.0 + 1),
            severed_parts: vec![PartId(1)],
        })
        .expect("the injury cut's reconciliation is accepted");

        let sheet = subject_sheet(&game, played, None).expect("sheet");
        assert!(
            sheet
                .parts
                .iter()
                .any(|part| part.id == PartId(1) && part.severed)
        );
        assert!(sheet.global_blockers.is_empty());
    }

    #[test]
    fn a_dead_subject_stays_inspectable_and_says_so() {
        let (mut game, played) = played_game();
        fall(&mut game, played, 200);
        assert!(!game.bodies().get(played).expect("body").alive());

        let sheet = subject_sheet(&game, played, None).expect("sheet");
        assert!(!sheet.parts.is_empty());
        assert!(
            sheet
                .global_blockers
                .iter()
                .any(|line| line.contains("dead"))
        );
    }
}
