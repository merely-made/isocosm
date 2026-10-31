// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native bodies into isometer's shared skin (phenotype-contract P5, case
//! 1049): Cosmograph reads each part's matter and function cells off the
//! entity, and the skin's forms, colours and marks go back on the placed
//! bodies. The
//! function-to-tissue table that lived here is isometer's now.
use super::super::{DeclaredExtentVolumes, PlacedBody};
use isometer::{SkinBody, SkinRules, SubjectKey};

pub(crate) use cosmograph::attention::spatial::skin_parts;

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
