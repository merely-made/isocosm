// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a body sees from (case 1143): the top centre of a named part's
//! posed box, falling back to the root's, posed exactly as the next draw
//! poses it so first person sits on the body the frame shows.

use super::*;

impl BodyLayer {
    /// One part's posed world box under the next draw's pose, or `None` when
    /// the part draws nothing (severed, absent or without faces).
    pub fn posed_part_bounds(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
        part: PartId,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        let (mesh, _) = self
            .project(body, volumes)
            .map_err(|error| format!("body projection: {error:?}"))?;
        Ok(part_box(posed(body, &mesh), &mesh, part))
    }

    /// The eye: the top centre of `part`'s posed box, or of the root's when
    /// no part is named or it draws nothing. The sim names the part: its
    /// most sensing (cosmograph `anatomy::sensing_part`).
    pub fn eye(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
        part: Option<PartId>,
    ) -> Result<Option<[f32; 3]>, String> {
        for part in part.into_iter().chain([body.document.root]) {
            if let Some((min, max)) = self.posed_part_bounds(body, volumes, part)? {
                return Ok(Some([(min[0] + max[0]) * 0.5, max[1], (min[2] + max[2]) * 0.5]));
            }
        }
        Ok(None)
    }
}

/// [`BodyLayer::eye`] without a scene: a host that sets its camera before
/// any device exists projects the body once here.
pub fn eye_of(
    body: &SceneBody<'_>,
    volumes: SceneVolumes<'_>,
    part: Option<PartId>,
) -> Result<Option<[f32; 3]>, String> {
    let mut projector = LiveBodyProjector::default();
    let (mesh, _) = match volumes {
        SceneVolumes::Voxels(map) => projector.project_body(body.document, map),
        SceneVolumes::DeclaredSolid(solid) => projector.project_body(body.document, solid),
    }
    .map_err(|error| format!("body projection: {error:?}"))?;
    for part in part.into_iter().chain([body.document.root]) {
        if let Some((min, max)) = part_box(posed(body, &mesh), &mesh, part) {
            return Ok(Some([(min[0] + max[0]) * 0.5, max[1], (min[2] + max[2]) * 0.5]));
        }
    }
    Ok(None)
}

/// The posed box of one part's faces under `live`.
pub(super) fn part_box(live: LiveBody<'_>, mesh: &BodyMesh, part: PartId) -> Option<([f32; 3], [f32; 3])> {
    let placement = mesh.placements.iter().find(|p| p.part == part)?;
    let quads = mesh.mesh_for(placement.volume)?.quads.len();
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for index in 0..quads {
        for corner in isometer_render::live_body::posed_quad(live, part, index).ok()? {
            for axis in 0..3 {
                min[axis] = min[axis].min(corner[axis]);
                max[axis] = max[axis].max(corner[axis]);
            }
        }
    }
    (min[0] <= max[0]).then_some((min, max))
}

/// First person's body (case 1143): `document` with every part not wholly
/// below the eye severed, so the body does not fill its own view and its
/// lower parts show at the frame's edge. The eye is the top of `sensing`
/// (the part the sim names), else of the root; the part it sits on is not
/// drawn. Parts are severed in the copy only, never in the world.
pub fn below_the_eye(document: &BodyDocument, sensing: Option<PartId>) -> BodyDocument {
    let part = sensing.filter(|part| document.part(*part).is_some_and(|p| !p.severed)).unwrap_or(document.root);
    let top = |id: PartId| Some(document.world_pivot(id)?[1] + document.part(id)?.half_extent[1]);
    let mut drawn = document.clone();
    let Some(eye) = top(part) else { return drawn };
    for p in &mut drawn.parts {
        if top(p.id).is_none_or(|t| t >= eye) {
            p.severed = true;
        }
    }
    drawn
}
