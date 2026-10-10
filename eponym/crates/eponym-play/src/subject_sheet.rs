// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Read-only subject/body and action presentation projection.
//!
//! This module owns labels and row shape only. `arrest_fall` evaluates
//! availability; callers remain responsible for admitting the supplied facts.

use crate::identity::{BodyRevisionId, SubjectId};
use isocosm::lineage::{LineageBody as BodyDocument, Provenance};
use isometer_core::{Aabb, PartId};

use crate::part_bounds;
use crate::{
    ActionBlocker, BindingBlocker, BindingKind, EquipmentFunction, EquipmentProjection,
    PartFunction, ResourceCost, ResourceKind, SourceQuery, SubjectBody, TechniqueId,
    TechniqueInputs, TechniqueKnowledge, arrest_fall,
};

/// Caller-owned facts used to build a sheet. No field is changed by projection.
pub struct SubjectSheetInput<'a> {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub current_revision: BodyRevisionId,
    pub body: &'a BodyDocument,
    pub knowledge: &'a TechniqueKnowledge,
    pub inputs: &'a TechniqueInputs,
    /// Stable presentation names. Missing entries fall back to `part N`.
    pub part_names: &'a [(PartId, &'a str)],
    /// Part selected by the host; selection is not world state.
    pub selected_part: Option<PartId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartRow {
    pub id: PartId,
    pub name: String,
    pub severed: bool,
    /// Parent in the body graph. This remains present for severed parts so a
    /// read-only inspection can explain where the part was attached.
    pub parent: Option<PartId>,
    /// World/body-space bounds derived through `BodyDocument::place`.
    pub bounds: Option<Aabb>,
    pub provenance: Provenance,
    pub capabilities: Vec<CapabilityRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityRow {
    pub function: PartFunction,
    pub reach_voxels: u32,
    pub load_capacity_mg: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionRow {
    pub kind: BindingKind,
    pub available: bool,
    pub uses_selected_part: bool,
    pub sources: Vec<SourceQuery>,
    pub costs: Vec<ResourceCost>,
    pub equipment: Vec<EquipmentRow>,
    pub blockers: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentRow {
    pub item: crate::ItemId,
    pub function: EquipmentFunction,
    pub carried_by: Option<SubjectId>,
    pub attached_to: Option<PartId>,
    pub reach_voxels: u32,
    pub load_capacity_mg: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceRow {
    pub kind: ResourceKind,
    pub available_units: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectSheet {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub learned: Vec<TechniqueId>,
    pub parts: Vec<PartRow>,
    pub actions: Vec<ActionRow>,
    pub global_blockers: Vec<String>,
    pub resources: Vec<ResourceRow>,
}

impl SubjectSheet {
    /// Build a stable, owned projection without mutating any source.
    pub fn from_input(input: SubjectSheetInput<'_>) -> Self {
        let query = arrest_fall(
            input.knowledge,
            SubjectBody {
                subject: input.subject,
                revision: input.revision,
                body: input.body,
            },
            input.current_revision,
            input.inputs,
        );
        let parts = input
            .body
            .parts
            .iter()
            .map(|part| PartRow {
                id: part.id,
                name: input
                    .part_names
                    .iter()
                    .find(|(id, _)| *id == part.id)
                    .map(|(_, name)| (*name).to_owned())
                    .unwrap_or_else(|| format!("part {}", part.id.0)),
                severed: part.severed,
                parent: part.attachment.map(|attachment| attachment.parent),
                bounds: part_bounds(input.body, part.id),
                provenance: input
                    .body
                    .provenance(part.id)
                    .cloned()
                    .unwrap_or_else(Provenance::founding),
                capabilities: input
                    .inputs
                    .part_capabilities
                    .iter()
                    .filter(|cap| cap.part == part.id)
                    .map(|cap| CapabilityRow {
                        function: cap.function,
                        reach_voxels: cap.reach_voxels,
                        load_capacity_mg: cap.load_capacity_mg,
                    })
                    .collect(),
            })
            .collect();
        let actions = query
            .bindings
            .iter()
            .map(|binding| ActionRow {
                kind: binding.kind,
                available: binding.available(),
                uses_selected_part: input.selected_part.is_some()
                    && binding
                        .sources
                        .iter()
                        .any(|source| *source == SourceQuery::Part(input.selected_part.unwrap())),
                sources: binding.sources.clone(),
                costs: binding.costs.clone(),
                equipment: equipment_for(
                    binding.sources.as_slice(),
                    input.inputs.equipment.as_slice(),
                ),
                blockers: binding.blockers.iter().map(binding_label).collect(),
            })
            .collect();
        Self {
            subject: query.subject,
            revision: input.revision,
            learned: input.knowledge.learned.clone(),
            parts,
            actions,
            global_blockers: query.blockers.iter().map(action_label).collect(),
            resources: input
                .inputs
                .resources
                .iter()
                .map(|r| ResourceRow {
                    kind: r.kind,
                    available_units: r.available_units,
                })
                .collect(),
        }
    }
}

fn equipment_for(sources: &[SourceQuery], equipment: &[EquipmentProjection]) -> Vec<EquipmentRow> {
    equipment
        .iter()
        .filter(|item| sources.contains(&SourceQuery::Equipment(item.item)))
        .map(|item| EquipmentRow {
            item: item.item,
            function: item.function,
            carried_by: item.carried_by,
            attached_to: item.attached_to,
            reach_voxels: item.reach_voxels,
            load_capacity_mg: item.load_capacity_mg,
        })
        .collect()
}

fn action_label(blocker: &ActionBlocker) -> String {
    match blocker {
        ActionBlocker::SubjectMismatch { body, knowledge } => format!(
            "knowledge belongs to subject {}, not {}",
            knowledge.0, body.0
        ),
        ActionBlocker::StaleRevision { known, current } => format!(
            "body revision {} is stale; current is {}",
            known.0, current.0
        ),
        ActionBlocker::NotLearned(_) => "arrest a fall has not been learned".into(),
    }
}

fn binding_label(blocker: &BindingBlocker) -> String {
    match blocker {
        BindingBlocker::ActionBlocked => "action is globally blocked".into(),
        BindingBlocker::MissingCapability(PartFunction::Grip) => "no gripping part".into(),
        BindingBlocker::MissingCapability(PartFunction::Adhesion) => "no adhesive part".into(),
        BindingBlocker::MissingPart(id) => format!("part {} is missing", id.0),
        BindingBlocker::SeveredPart(id) => format!("part {} is severed", id.0),
        BindingBlocker::OccupiedPart(id) => format!("part {} is occupied", id.0),
        BindingBlocker::MissingEquipment(EquipmentFunction::Line) => "no line equipment".into(),
        BindingBlocker::MissingEquipment(EquipmentFunction::Harness) => {
            "no harness equipment".into()
        },
        BindingBlocker::NotCarriedEquipment(id) => format!("equipment {} is not carried", id.0),
        BindingBlocker::HarnessUnattached(id) => format!("harness {} is unattached", id.0),
        BindingBlocker::NoReachableSupport {
            reach_voxels,
            distance_voxels,
        } => format!(
            "support is {} voxels away, beyond reach {}",
            distance_voxels, reach_voxels
        ),
        BindingBlocker::NoSupportPresent => "no support is present".into(),
        BindingBlocker::InsufficientLoadCapacity {
            capacity_mg,
            load_mg,
        } => format!("capacity {} mg is below {} mg load", capacity_mg, load_mg),
        BindingBlocker::UnsuitableSurface => "surface is unsuitable".into(),
        BindingBlocker::ResourceExhausted => "required resource is exhausted".into(),
    }
}
