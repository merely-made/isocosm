// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{HostConfig, PlayedTrace, played::SceneMode};

fn host() -> Host {
    Host::new(HostConfig {
        seed: 7,
        scene: SceneMode::GraftPractice,
        ..HostConfig::default()
    })
}

#[test]
fn whole_body_preview_fits_deep_bodies_after_every_turn() {
    use crate::section::{CameraMode, camera_basis};
    use mesocosm_core::world::generation::{BodyPlan, Request};
    let mut exceeds_world_slice = false;
    for plan in [BodyPlan::Axial, BodyPlan::Branched] {
        let mut request = Request::default();
        request.criteria.body_plan = plan;
        let prepared = request
            .prepare(mesocosm_core::Founding::Drawn.palette())
            .unwrap();
        for index in 0..prepared.draft().candidates.len() {
            let world = prepared.enter(index).unwrap();
            let organism = world.controlled().unwrap();
            let bounds = organism.body().aabb();
            for mode in CameraMode::ALL {
                for pitch in [None, Some(12.0)] {
                    for frame in [(960, 600), (1280, 720)] {
                        let origin = [0, 1, 2].map(|i| {
                            organism.position[i] as f32
                                - if i == 1 && pitch.is_some() {
                                    bounds.min[1] as f32
                                } else {
                                    0.0
                                }
                        });
                        let presented = (
                            [0, 1, 2].map(|i| origin[i] + bounds.min[i] as f32),
                            [0, 1, 2].map(|i| origin[i] + bounds.max[i] as f32),
                        );
                        let (centre, half, depth) = framing(presented, frame, mode, pitch).unwrap();
                        exceeds_world_slice |= depth > crate::section::SLAB_DEPTH;
                        let [right, up, _] = camera_basis(mode, pitch);
                        let normal = [right[2], 0.0, -right[0]];
                        for mask in 0..8 {
                            let delta: [f32; 3] = [0, 1, 2].map(|i| {
                                let corner = if mask & (1 << i) == 0 {
                                    bounds.min[i]
                                } else {
                                    bounds.max[i]
                                };
                                organism.position[i] as f32 + corner as f32
                                    - centre[i]
                                    - if i == 1 && pitch.is_some() {
                                        bounds.min[1] as f32
                                    } else {
                                        0.0
                                    }
                            });
                            let dot =
                                |axis: [f32; 3]| (0..3).map(|i| delta[i] * axis[i]).sum::<f32>();
                            assert!(
                                dot(normal).abs() < depth * 0.5,
                                "{plan:?}/{mode:?}: whole body must survive cutaway clipping"
                            );
                            assert!(dot(up).abs() < half);
                        }
                    }
                }
            }
        }
    }
    assert!(
        exceeds_world_slice,
        "fixture must exercise the old thin-slice failure"
    );
}

#[test]
fn rotating_presentation_bounds_fit_beside_the_panel_and_inside_the_slab() {
    use isometer::mesh::{BodyMesh, Volume};
    use isometer::render::live_body::{LiveBody, body_bounds};
    let mesh = BodyMesh::single(
        mesocosm_core::VolumeRef::from_tag(251),
        &Volume::solid([4, 6, 40], 3),
    );
    for degrees in [0.0_f32, 45.0, 90.0, 180.0, 270.0, 360.0] {
        let mut body = LiveBody::new(&mesh, [13.0, 4.0, -17.0]);
        body.yaw_radians = degrees.to_radians();
        let bounds = body_bounds(body).unwrap().unwrap();
        for mode in crate::section::CameraMode::ALL {
            let frame = (960, 600);
            let pitch = Some(12.0);
            let (centre, half, depth) = framing(bounds, frame, mode, pitch).unwrap();
            let [right, up, forward] = crate::section::camera_basis(mode, pitch);
            let length = forward[0].hypot(forward[2]);
            let normal = [forward[0] / length, 0.0, forward[2] / length];
            let available =
                (frame.0 as f32 - mesocosm_views::BODY_MENU_WIDTH as f32 - 24.0).max(80.0);
            let left = -half * frame.0 as f32 / frame.1 as f32;
            let right_edge = left + 2.0 * half * available / frame.1 as f32;
            for mask in 0..8 {
                let delta = [0, 1, 2].map(|i| {
                    if mask & (1 << i) == 0 {
                        bounds.0[i] - centre[i]
                    } else {
                        bounds.1[i] - centre[i]
                    }
                });
                let dot = |axis: [f32; 3]| (0..3).map(|i| delta[i] * axis[i]).sum::<f32>();
                assert!(
                    dot(right) > left && dot(right) < right_edge,
                    "{degrees}/{mode:?}: panel fit"
                );
                assert!(dot(up).abs() < half, "{degrees}/{mode:?}: vertical fit");
                assert!(
                    dot(normal).abs() < depth * 0.5,
                    "{degrees}/{mode:?}: cutaway fit"
                );
            }
        }
    }
}

#[test]
fn browsing_and_cancelling_a_graft_never_change_the_world_or_trace() {
    let mut host = host();
    let hash = host.runtime.state_hash();
    assert!(host.run_action("h"));
    assert!(host.grafting.open);
    assert!(host.grafting.admissible);
    assert!(host.grafting.preview.is_some());
    for action in ["down", "up", "tab", "tab", "w", "e", ".", "g", "f"] {
        assert!(host.run_action(action));
        host.advance();
        assert_eq!(host.runtime.state_hash(), hash);
        assert!(host.runtime.trace().is_empty());
        assert_eq!(host.runtime.queued_len(), 0);
    }
    assert!(host.run_action("escape"));
    assert!(!host.grafting.open);
    assert!(host.grafting.preview.is_none());
    assert_eq!(host.runtime.state_hash(), hash);
}

#[test]
fn confirmation_records_one_graft_and_replays_from_the_named_scene() {
    let mut host = host();
    host.run_action("h");
    let candidate = host
        .grafting
        .preview
        .as_ref()
        .unwrap()
        .controlled()
        .unwrap()
        .phenotype
        .clone();
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    assert!(matches!(host.runtime.trace()[0], Intent::Graft { .. }));
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Grafted { parts: 2, .. }
    ));
    assert_eq!(
        host.runtime.world().body().unwrap().parts.len(),
        candidate.body().parts.len()
    );
    assert!(host.grafting.preview.is_none());
    assert!(host.grafting.reading.status.starts_with("Grafted"));
    // A second Enter is not a second transaction without another selection.
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    let trace = PlayedTrace {
        start: None,
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: SceneMode::GraftPractice,
        body_layout: host.config.effective_body_layout(),
        seed: 7,
        organisms: host.runtime.receipt().organisms,
        steps: 1,
        state_hash: host.runtime.state_hash(),
        intents: host.runtime.trace().to_vec(),
        content: host.content.clone(),
    };
    let mut replay = Host::new(HostConfig {
        seed: 7,
        replay: Some(trace),
        ..HostConfig::default()
    });
    assert!(replay.advance());
    assert_eq!(replay.runtime.state_hash(), host.runtime.state_hash());
}

#[test]
fn a_stale_preview_requires_review_before_it_can_commit() {
    let mut host = host();
    host.run_action("h");
    host.runtime.queue(Intent::Idle);
    host.runtime.step(1);
    let after = host.runtime.state_hash();
    let count = host.runtime.trace().len();
    host.run_action("enter");
    assert_eq!(host.runtime.state_hash(), after);
    assert_eq!(host.runtime.trace().len(), count);
    assert!(
        host.grafting
            .reading
            .status
            .starts_with("The world changed")
    );
}

#[test]
fn an_empty_menu_is_safe_and_does_not_require_dev_mode() {
    let mut host = Host::new(HostConfig {
        organisms: 0,
        ..HostConfig::default()
    });
    let hash = host.runtime.state_hash();
    host.run_action("h");
    assert!(host.grafting.open);
    assert!(host.grafting.sources.is_empty());
    assert!(host.grafting.reading.detail.starts_with("No reachable"));
    for key in ["up", "down", "tab", "enter", "escape"] {
        host.run_action(key);
    }
    assert_eq!(host.runtime.state_hash(), hash);
    assert!(host.runtime.trace().is_empty());
}
