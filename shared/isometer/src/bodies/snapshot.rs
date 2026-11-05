// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_render::{
    ClipSlab,
    implicit::Surface,
    live_body::{BodyHit, BodyQueryError},
};
use std::sync::Arc;

/// Only CPU query inputs; retaining these does not duplicate a GPU Scene.
#[derive(Clone)]
pub(crate) struct BodyQueries(Arc<QueryData>);
struct QueryData {
    placed: Vec<PlacedBody>,
    surfaces: BTreeMap<SubjectKey, Surface>,
}
pub(super) struct QueryView<'a> {
    placed: &'a [PlacedBody],
    surfaces: &'a BTreeMap<SubjectKey, Surface>,
}

impl BodyLayer {
    pub(super) fn query_view(&self) -> QueryView<'_> {
        QueryView {
            placed: &self.placed,
            surfaces: &self.surfaces,
        }
    }
    pub(crate) fn query_snapshot(&self) -> BodyQueries {
        BodyQueries(Arc::new(QueryData {
            placed: self.placed.clone(),
            surfaces: self.surfaces.clone(),
        }))
    }
}

impl BodyQueries {
    pub(crate) fn record_counts(&self) -> (usize, usize) {
        (self.0.placed.len(), self.0.surfaces.len())
    }

    fn view(&self) -> QueryView<'_> {
        QueryView {
            placed: &self.0.placed,
            surfaces: &self.0.surfaces,
        }
    }
    pub(crate) fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: ClipSlab,
    ) -> Result<Option<(PartAddress, BodyHit)>, BodyQueryError> {
        self.view().pick(origin, direction, far, clip)
    }
    pub(crate) fn select_part(
        &self,
        subject: SubjectKey,
        current: Option<PartAddress>,
        backwards: bool,
    ) -> Option<PartAddress> {
        self.view().select_part(subject, current, backwards)
    }
    pub(crate) fn contains(&self, address: PartAddress) -> bool {
        self.0.placed.iter().any(|body| {
            body.subject == address.subject
                && body.revision == address.revision
                && body
                    .mesh
                    .placements
                    .iter()
                    .any(|part| part.part == address.part && drawable(&body.mesh, part.part))
        })
    }
}

impl QueryView<'_> {
    pub(super) fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: ClipSlab,
    ) -> Result<Option<(PartAddress, BodyHit)>, BodyQueryError> {
        if self.surfaces.is_empty() {
            let bodies: Vec<_> = self.placed.iter().map(PlacedBody::live).collect();
            return isometer_render::live_body::pick_bodies(
                &bodies,
                origin,
                direction,
                far,
                Some(clip),
            )
            .map(|hit| {
                hit.map(|hit| {
                    let body = &self.placed[hit.body_index];
                    (
                        PartAddress {
                            subject: body.subject,
                            part: hit.part,
                            revision: body.revision,
                        },
                        hit,
                    )
                })
            });
        }
        let mut best: Option<(PartAddress, BodyHit)> = None;
        for (index, body) in self.placed.iter().enumerate() {
            let hit = if let Some(surface) = self.surfaces.get(&body.subject) {
                surface.pick(origin, direction, far, clip)?
            } else {
                isometer_render::live_body::pick_bodies(
                    &[body.live()],
                    origin,
                    direction,
                    far,
                    Some(clip),
                )?
            };
            if let Some(mut hit) = hit {
                hit.body_index = index;
                let address = PartAddress {
                    subject: body.subject,
                    part: hit.part,
                    revision: body.revision,
                };
                if best.as_ref().is_none_or(|(_, b)| hit.distance < b.distance) {
                    best = Some((address, hit));
                }
            }
        }
        Ok(best)
    }
    pub(super) fn select_part(
        &self,
        subject: SubjectKey,
        current: Option<PartAddress>,
        backwards: bool,
    ) -> Option<PartAddress> {
        let body = self.placed.iter().find(|body| body.subject == subject)?;
        let parts: Vec<_> = body
            .mesh
            .placements
            .iter()
            .filter(|part| drawable(&body.mesh, part.part))
            .collect();
        if parts.is_empty() {
            return None;
        }
        let current = current.filter(|address| address.subject == subject);
        let index = current
            .and_then(|address| parts.iter().position(|part| part.part == address.part))
            .map(|index| {
                if backwards {
                    index.checked_sub(1).unwrap_or(parts.len() - 1)
                } else {
                    (index + 1) % parts.len()
                }
            })
            .unwrap_or_else(|| if backwards { parts.len() - 1 } else { 0 });
        Some(PartAddress {
            subject,
            part: parts[index].part,
            revision: body.revision,
        })
    }
}
