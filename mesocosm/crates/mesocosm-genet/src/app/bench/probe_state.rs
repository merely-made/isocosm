// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Read-only receipts of the existing Creator and scene producer.

use super::Context;
use genet_probe::ProbeSnapshot;
pub(super) fn snapshot(ctx: &Context<'_>, captures: usize, opacity: f32) -> ProbeSnapshot {
    let state = ctx.runner.state();
    let model = state.model.borrow();
    let scene = state.scene.borrow();
    let creator_ready =
        !model.creator.pending && model.creator.prepared.is_some() && model.creator.count() > 0;
    let yes = |value| if value { "yes" } else { "no" };
    let mut snapshot = ProbeSnapshot::default()
        .with_field("notice", state.notice.clone())
        .with_field("seed", model.creator.request.seed.to_string())
        .with_field("variation", model.creator.request.variation.to_string())
        .with_field(
            "structure-layout",
            model
                .creator
                .request
                .criteria
                .structure
                .as_ref()
                .map_or("none", |s| s.layout.label()),
        )
        .with_field(
            "structure-organs",
            model
                .creator
                .request
                .criteria
                .structure
                .as_ref()
                .map_or("none", |s| s.organs.label()),
        )
        .with_field(
            "structure-stretches",
            model
                .creator
                .request
                .criteria
                .structure
                .as_ref()
                .map_or(0, |s| s.branch_count)
                .to_string(),
        )
        .with_field(
            "structure-length",
            model
                .creator
                .request
                .criteria
                .structure
                .as_ref()
                .map_or(0, |s| s.segment_length)
                .to_string(),
        )
        .with_field(
            "body-plan",
            model.creator.request.criteria.body_plan.label(),
        )
        .with_field(
            "archetype",
            model
                .creator
                .request
                .criteria
                .archetype
                .map_or("none", |a| a.label()),
        )
        .with_field("size", state.generation.size.to_string())
        .with_field(
            "mass",
            model
                .world()
                .controlled()
                .map_or(0, |o| o.biomass_mg())
                .to_string(),
        )
        .with_field(
            "capacity",
            model
                .world()
                .controlled()
                .map_or(0, |o| o.mass_ceiling_mg())
                .to_string(),
        )
        .with_field(
            "body-bounds",
            model
                .world()
                .controlled()
                .map_or("none".into(), |o| format!("{:?}", o.body().aabb().extent())),
        )
        .with_field(
            "comparison-rendered",
            model
                .comparison
                .as_ref()
                .map_or(0, |c| {
                    c.cards
                        .iter()
                        .enumerate()
                        .filter(|(index, card)| {
                            let scene = state.cards[*index].borrow();
                            card.world.is_some()
                                && scene.section.is_some()
                                && scene.error.is_none()
                                && scene.stats.voxel_bodies == 1
                        })
                        .count()
                })
                .to_string(),
        )
        .with_field("comparison", yes(model.comparison.is_some()))
        .with_field(
            "alternative",
            model
                .comparison
                .as_ref()
                .map_or("none".into(), |c| c.selected.to_string()),
        )
        .with_field(
            "alternatives-admitted",
            model
                .comparison
                .as_ref()
                .map_or(0, |c| {
                    c.cards.iter().skip(1).filter(|c| c.world.is_some()).count()
                })
                .to_string(),
        )
        .with_field(
            "comparison-round",
            model
                .comparison
                .as_ref()
                .map_or(0, |c| c.source.round)
                .to_string(),
        )
        .with_field(
            "comparison-scale",
            model
                .comparison
                .as_ref()
                .map_or("none", |c| if c.shared_scale { "shared" } else { "fit" }),
        )
        .with_field(
            "original-hash",
            model
                .comparison
                .as_ref()
                .and_then(|c| c.cards[0].world.as_deref())
                .map_or("none".into(), |w| {
                    format!("{:016x}", mesocosm_core::state_hash(w))
                }),
        )
        .with_field("opacity", opacity.to_string())
        .with_field("ui-zoom", ctx.ui_zoom.to_string())
        .with_field(
            "output-scale",
            ctx.window.map_or(1.0, |w| w.scale_factor()).to_string(),
        )
        .with_field("logical-width", ctx.logical_size.0.to_string())
        .with_field("logical-height", ctx.logical_size.1.to_string())
        .with_field("creator-ready", yes(creator_ready))
        .with_field("creator-pending", yes(model.creator.pending))
        .with_field(
            "ready",
            yes(creator_ready
                && scene.section.is_some()
                && scene.error.is_none()
                && scene.renders > 0),
        )
        .with_field("epoch", model.epoch.to_string())
        .with_field("revision", model.revision.to_string())
        .with_field("candidate", model.creator.selected.to_string())
        .with_field("candidates", model.creator.count().to_string())
        .with_field("selected", yes(model.selected.is_some()))
        .with_field(
            "selected-part",
            model
                .selected
                .map_or("none".into(), |s| s.part.0.to_string()),
        )
        .with_field(
            "selected-organism",
            model
                .selected
                .map_or("none".into(), |s| s.organism.0.to_string()),
        )
        .with_field("view", if model.isolated { "body" } else { "habitat" })
        .with_field("camera", model.camera.name())
        .with_field(
            "tint",
            match state.tint {
                1 => "warm",
                2 => "cool",
                _ => "natural",
            },
        )
        .with_field("visible", yes(state.visible))
        .with_field("active", yes(scene.section.is_some()))
        .with_field("decorated", yes(state.decorated))
        .with_field("transformed", yes(state.transformed))
        .with_field("renders", scene.renders.to_string())
        .with_field("mesh-bytes", scene.mesh_upload_bytes.to_string())
        .with_field("instance-bytes", scene.instance_upload_bytes.to_string())
        .with_field("drawn-bodies", scene.stats.voxel_bodies.to_string())
        .with_field("drawn-parts", scene.stats.voxel_parts.to_string())
        .with_field("fallback-bodies", scene.stats.fallback_bodies.to_string())
        .with_field("overlay-clicks", state.overlay_clicks.to_string())
        .with_field(
            "hash",
            format!("{:016x}", mesocosm_core::state_hash(model.world())),
        )
        .with_field(
            "error",
            state
                .published_error
                .as_deref()
                .or(scene.error.as_deref())
                .unwrap_or("none"),
        )
        .with_field("captures", captures.to_string());
    {
        let m = state.model.borrow();
        snapshot = snapshot.with_field("spatial-drawn", scene.glyph_count.to_string());
        snapshot = snapshot.with_field("spatial-anchors", scene.anchor_count.to_string());
        snapshot = snapshot.with_field("spatial-enabled", m.spatial.enabled.to_string());
        snapshot = snapshot.with_field("spatial-tick", m.spatial.tick.to_string());
        snapshot = snapshot.with_field("spatial-playing", m.spatial.playing.to_string());
        snapshot = snapshot.with_field("spatial-camera", format!("{:?}", m.camera));
        snapshot = snapshot.with_field("spatial-form", m.spatial.form.label());
        snapshot = snapshot.with_field("spatial-glyph", m.spatial.glyph.label());
        snapshot = snapshot.with_field("spatial-seed", m.spatial.seed.to_string());
        snapshot = snapshot.with_field("spatial-count", m.spatial.count.to_string());
        snapshot = snapshot.with_field(
            "spatial-saved",
            m.spatial
                .saved
                .as_ref()
                .map_or("none".into(), |p| p.display().to_string()),
        );
    }
    for (key, value) in state.effects.probe_fields() {
        snapshot = snapshot.with_field(key, value);
    }
    snapshot
}
