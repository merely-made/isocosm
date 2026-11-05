// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn shared_totals_survive_source_replacement_and_reading_adds_no_work() {
    let owner = Costs::default();
    let source = owner.clone();
    let body = BodyFrameStats {
        draw_parts: 2,
        mesh_builds: 1,
        mesh_upload_bytes: 48,
        instance_upload_bytes: 16,
        frame_upload_bytes: 8,
        ..Default::default()
    };
    source.drawn(&body, None, 0, ResidencyStats::default(), true);
    drop(source);
    let first = owner.snapshot();
    assert_eq!(first.successful_draws, 1);
    assert_eq!((first.mesh_bytes(), first.instance_bytes()), (48, 16));
    assert_eq!(
        owner.snapshot().successful_draws,
        1,
        "a retained observation adds no draw"
    );
    let replacement = owner.clone();
    replacement.drawn(&body, None, 0, ResidencyStats::default(), true);
    let second = owner.snapshot();
    assert_eq!(second.successful_draws, 2);
    assert_eq!((second.mesh_bytes(), second.instance_bytes()), (96, 32));
    assert_eq!(second.mesh_builds, 2);
}

#[test]
fn qualified_bytes_do_not_turn_unexposed_components_into_measured_zero() {
    let costs = Costs::default();
    let body = BodyFrameStats {
        draw_parts: 1,
        mesh_upload_bytes: 80,
        body_upload_measurement: Some("mesh estimates; tenant frame uploads not exposed"),
        ..Default::default()
    };
    costs.drawn(&body, None, 0, ResidencyStats::default(), true);
    let report = costs.snapshot();
    let bytes = &report.body_bytes_by_measurement[body.body_upload_measurement.unwrap()];
    assert_eq!(bytes.mesh_bytes, Some(80));
    assert_eq!((bytes.instance_bytes, bytes.frame_bytes), (None, None));
    assert!(report.terrain.is_none());
    let json = serde_json::to_value(report).unwrap();
    assert!(
        json["body_bytes_by_measurement"][body.body_upload_measurement.unwrap()]["instance_bytes"]
            .is_null()
    );
    assert!(
        json["unavailable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "input_to_present_latency")
    );
}

#[test]
fn terrain_diagnostics_add_once_and_retained_last_change_is_not_recounted() {
    let costs = Costs::default();
    let body = BodyFrameStats::default();
    let residency = ResidencyStats {
        loaded: 3,
        refreshed: 2,
        ..Default::default()
    };
    let first = BrickDiagnostics {
        brick_upload_bytes: 64,
        uniform_upload_bytes: 8,
        atlas_write_calls: 1,
        trace_passes: 1,
        ..Default::default()
    };
    costs.drawn(&body, Some(first), 1, residency, false);
    costs.drawn(
        &body,
        Some(BrickDiagnostics {
            uniform_upload_bytes: 8,
            trace_passes: 1,
            ..Default::default()
        }),
        0,
        residency,
        false,
    );
    let totals = costs.snapshot().terrain.unwrap();
    assert_eq!((totals.draws, totals.trace_passes), (2, 2));
    assert_eq!(
        (totals.brick_upload_bytes, totals.uniform_upload_bytes),
        (64, 16)
    );
    assert_eq!(
        (
            totals.residency_changes,
            totals.loaded_bricks,
            totals.refreshed_bricks
        ),
        (1, 3, 2)
    );
}
