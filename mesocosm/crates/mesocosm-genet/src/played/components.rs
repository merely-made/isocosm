// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A derived component-store backend over an already admitted [`SiteScene`].
//!
//! Capture and reconstruction are measured separately from shared native scene
//! extraction. This transports the supplied body representation; it does not
//! perform admission, expand cohorts, infer poses or replace the native record.
//! Source cuts, terrain and disclosure remain with the host's scene snapshot.

use cosmograph::store::{SparseStore, intersection};
use isometer::core::BodyDocument;

use crate::section::{PlacedBody, SiteScene, Walk};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
struct Placement {
    at: [i32; 3],
    physical: Option<mesocosm_runtime::spatial::Pose>,
    walks: Vec<Walk>,
}

#[derive(Clone, Debug)]
struct Appearance {
    tint: [f32; 3],
    parts: Vec<isometer::PartSkin>,
    materials: Vec<isometer::render::PartMaterial>,
    palette: Vec<isometer::PaletteColour>,
    alive: bool,
}

/// Stored component rows, for the comparison's workload receipt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct ComponentCounts {
    pub documents: usize,
    pub placements: usize,
    pub appearances: usize,
}

/// Three typed, native-identity-ordered stores for one supplied body roster.
/// Clones share immutable component payloads until a store is changed.
#[derive(Clone, Debug, Default)]
pub struct ComponentScene {
    documents: SparseStore<BodyDocument>,
    placements: SparseStore<Placement>,
    appearances: SparseStore<Appearance>,
    supplied: usize,
}

impl ComponentScene {
    /// Captures only the supplied body vector. A repeated identity or incomplete
    /// component membership is refused by [`Self::bodies`] before publication.
    pub fn capture(scene: &SiteScene) -> Self {
        let mut result = Self {
            supplied: scene.bodies.len(),
            ..Self::default()
        };
        for body in &scene.bodies {
            result.insert(body);
        }
        result
    }

    /// The supplied roster's size; cohort multiplicity is never expanded.
    pub fn len(&self) -> usize {
        self.supplied
    }

    pub fn is_empty(&self) -> bool {
        self.supplied == 0
    }

    pub fn counts(&self) -> ComponentCounts {
        ComponentCounts {
            documents: self.documents.len(),
            placements: self.placements.len(),
            appearances: self.appearances.len(),
        }
    }

    fn insert(&mut self, body: &PlacedBody) {
        self.documents.set(body.id, body.document.clone());
        self.placements.set(
            body.id,
            Placement {
                at: body.at,
                physical: body.physical.clone(),
                walks: body.walks.clone(),
            },
        );
        self.appearances.set(
            body.id,
            Appearance {
                tint: body.tint,
                parts: body.parts.clone(),
                materials: body.materials.clone(),
                palette: body.palette.clone(),
                alive: body.alive,
            },
        );
    }

    /// Reconstructs the complete roster in ascending native identity order.
    /// Missing or differently keyed components refuse the whole reconstruction;
    /// an intersection must never silently omit an admitted subject.
    pub fn bodies(&self) -> Result<Vec<PlacedBody>, String> {
        let expected = ComponentCounts {
            documents: self.supplied,
            placements: self.supplied,
            appearances: self.supplied,
        };
        if self.counts() != expected {
            return Err(format!(
                "component scene membership differs: expected {expected:?}, found {:?}",
                self.counts()
            ));
        }
        let mut appearances = intersection(&self.documents, &self.appearances);
        let mut bodies = Vec::with_capacity(self.supplied);
        for (id, document, placement) in intersection(&self.documents, &self.placements) {
            let Some((appearance_id, _, appearance)) = appearances.next() else {
                return Err(format!(
                    "component scene has no common appearance for subject {id}"
                ));
            };
            if id != appearance_id {
                return Err(format!(
                    "component scene memberships disagree at subject {id}"
                ));
            }
            bodies.push(PlacedBody {
                id,
                document: document.clone(),
                at: placement.at,
                physical: placement.physical.clone(),
                walks: placement.walks.clone(),
                tint: appearance.tint,
                parts: appearance.parts.clone(),
                materials: appearance.materials.clone(),
                palette: appearance.palette.clone(),
                alive: appearance.alive,
            });
        }
        if bodies.len() != self.supplied || appearances.next().is_some() {
            return Err("component scene join omitted a supplied subject".into());
        }
        Ok(bodies)
    }
}
