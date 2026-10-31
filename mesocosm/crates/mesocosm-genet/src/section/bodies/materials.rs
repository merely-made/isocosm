// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native bodies into isometer's shared skin (phenotype-contract P5, case
//! 1049): each part's matter and function cells read off the entity, and the
//! skin's forms, colours and marks put back on the placed bodies. The
//! function-to-tissue table that lived here is isometer's now.
use super::super::{DeclaredExtentVolumes, PlacedBody};
use isometer::{PartSkin, SkinBody, SkinRules, SubjectKey};

/// What the skin reads from a native entity: each living part's matter
/// ledger and its cells per function out of its capacity.
pub(crate) fn skin_parts(entity: &cosmograph::schema::Entity) -> Vec<PartSkin> {
    let allocations = cosmograph::attention::spatial::allocations(entity);
    entity
        .parts
        .iter()
        .map(|(&part, held)| {
            let allocation = allocations.get(&part);
            PartSkin {
                matter: held.matter.clone(),
                functions: allocation.map(|a| a.cells.clone()).unwrap_or_default(),
                capacity: allocation.map_or(0, |a| a.capacity),
                ..PartSkin::new(part)
            }
        })
        .collect()
}

/// Skin every placed body: its drawn document becomes the skin's
/// presentation copy (only volume references differ), and its marks and
/// colour table are filled. The returned forms resolve those copies.
pub(crate) fn body_volumes(bodies: &mut [PlacedBody]) -> Result<DeclaredExtentVolumes, String> {
    let mut skin = isometer::skin(
        bodies.iter().map(|body| SkinBody {
            subject: SubjectKey(body.id),
            document: &body.document,
            parts: &body.parts,
            seed: body.id,
        }),
        &SkinRules::default(),
    )
    .map_err(|why| format!("native body appearance: {why:?}"))?;
    for body in bodies.iter_mut() {
        let subject = SubjectKey(body.id);
        if let Some(document) = skin.documents.remove(&subject) {
            body.document = document;
        }
        body.materials = skin.marks.remove(&subject).unwrap_or_default();
        body.palette = skin.palettes.remove(&subject).unwrap_or_default();
    }
    Ok(skin.volumes)
}
