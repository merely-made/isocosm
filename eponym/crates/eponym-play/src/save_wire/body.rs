// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Generator-1 anatomy wire, before geometry and lineal matter separated.
//! Decoding preserves every old part fact; it does not authenticate a save.

use isocosm::lineage::{LineageBody, Provenance, SpeciesId};
use isometer_core::{Attachment, PartId, VolumeRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Part {
    id: PartId,
    volume: VolumeRef,
    mass_mg: u64,
    half_extent: [i32; 3],
    pivot: [i32; 3],
    attachment: Option<Attachment>,
    provenance: Provenance,
    #[serde(default)]
    severed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Body {
    species: SpeciesId,
    root: PartId,
    plan: isometer_core::plan::BodyPlan,
    parts: Vec<Part>,
}

impl Body {
    pub(super) fn native(self) -> Result<LineageBody, String> {
        if self.parts.is_empty() || self.parts.len() > crate::anatomy::MAX_ANATOMY_PARTS {
            return Err("legacy anatomy is empty or exceeds its admission limit".into());
        }
        if self.parts.iter().enumerate().any(|(index, part)| part.id != PartId(index as u32)) {
            return Err("legacy anatomy part IDs are not contiguous in wire order".into());
        }
        let first = &self.parts[0];
        let mut body = LineageBody::new(self.species, first.volume, first.mass_mg, first.half_extent);
        // Allocate the lineal rows through their owner, then replace the complete
        // geometry with the original facts. Admission validates the actual tree.
        for part in self.parts.iter().skip(1) {
            body.attach(part.volume, part.mass_mg, part.half_extent,
                Attachment { parent:PartId(0), offset:[0;3], yaw:isometer_core::Yaw::Zero },
                part.provenance.clone()).map_err(|_| "legacy lineal row allocation failed")?;
        }
        for (index, part) in self.parts.iter().enumerate() {
            body.set_provenance(PartId(index as u32), part.provenance.clone());
        }
        *body.doc_mut() = isometer_core::BodyDocument {
            root:self.root, plan:self.plan,
            parts:self.parts.into_iter().map(|p| isometer_core::Part {
                id:p.id, volume:p.volume, half_extent:p.half_extent, pivot:p.pivot,
                attachment:p.attachment, origin:p.provenance.tag(), severed:p.severed,
                situs:None, shape:String::new(),
            }).collect(),
        };
        Ok(body)
    }

    pub(super) fn old(body: &LineageBody) -> Result<Self, String> {
        if body.parts.iter().enumerate().any(|(index, part)| part.id != PartId(index as u32)) {
            return Err("anatomy part IDs do not match their lineal rows".into());
        }
        let parts = body.parts.iter().map(|p| {
            if p.situs.is_some() || !p.shape.is_empty() {
                return Err("new geometry cannot be written as a generator-1 anatomy".into());
            }
            let lineal = body.lineal(p.id).ok_or("anatomy has no matching lineal row")?;
            if p.origin != lineal.provenance.tag() {
                return Err("geometry and lineal provenance disagree".into());
            }
            Ok(Part { id:p.id, volume:p.volume, mass_mg:lineal.mass_mg,
                half_extent:p.half_extent, pivot:p.pivot, attachment:p.attachment,
                provenance:lineal.provenance.clone(), severed:p.severed })
        }).collect::<Result<_, String>>()?;
        Ok(Self { species:body.species(), root:body.root, plan:body.plan.clone(), parts })
    }
}
