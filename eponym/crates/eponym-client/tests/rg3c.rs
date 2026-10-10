// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Opt-in RG3c receipt over the fixed Eponym room tenant.

use eponym_client::gpu::{self, Composer, SIZE, Tenant};
use eponym_client::room::SEED;
use eponym_client::{Probe, TICKS, scene};

fn draw_tenant(
    tenant: &mut Tenant,
    probe: &Probe,
    room: &[isometer::render::geometry::Vertex],
) -> tenant::FrameReport {
    let aspect = SIZE[0] as f32 / SIZE[1] as f32;
    let camera = scene::camera(probe.room(), probe.at(), probe.heading(), aspect);
    tenant.look(&camera);
    tenant.light(&scene::torch(camera.eye, 0.0));
    tenant.set_room(room);
    tenant.set_body(&scene::body_vertices(probe.at()));
    tenant.draw()
}

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y * SIZE[0] + x) * 4) as usize;
    bytes[index..index + 4].try_into().expect("rgba pixel")
}

#[test]
#[ignore = "opt-in physical RG3c room receipt"]
fn rg3c_fixed_room_graph_matches_legacy_composition() {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let handles = gpu::boot(&instance, None);
    let candidate_composer = Composer::new(handles.clone(), SIZE);
    let legacy_composer = Composer::new(handles.clone(), SIZE);
    let mut candidate_tenant = Tenant::new(&handles, SIZE);
    let mut legacy_tenant = Tenant::new(&handles, SIZE);
    let probe = Probe::new(SEED).expect("fixed room probe");
    let room = scene::room_vertices(probe.room());
    let candidate_report = draw_tenant(&mut candidate_tenant, &probe, &room);
    let _ = draw_tenant(&mut legacy_tenant, &probe, &room);
    let chrome = scene::chrome(SIZE, 0, TICKS);

    let (candidate_master, receipt) =
        candidate_composer.compose_opaque_tenant(&chrome, &candidate_tenant, candidate_report);
    let legacy_master = legacy_composer.compose(&chrome, &legacy_tenant.view);
    let candidate = candidate_composer.capture(&candidate_master);
    let legacy = legacy_composer.capture(&legacy_master);

    assert_eq!(candidate.pixels, legacy.pixels);
    assert!(!candidate.is_trivial());
    let chrome_anchor = pixel(&candidate.pixels, 1, 1);
    assert!(
        chrome_anchor[0] > chrome_anchor[2] && chrome_anchor[1] > chrome_anchor[2],
        "boundary-zero chrome anchor is not present: {chrome_anchor:?}"
    );
    assert!(candidate.distinct > 16, "room content is not visible");
    assert_eq!(receipt.tenant_name, "eponym-client");
    assert_eq!(receipt.producer_path, gpu::PRODUCER_PATH);
    assert_eq!(receipt.fallback_count, 0);
    assert_eq!(receipt.scene_op_boundary, 0);
    assert_eq!(receipt.caller_reported_physical_submission_count, Some(1));
    assert_eq!(candidate_report.internal_submissions, 0);
    assert_eq!(receipt.logical_opaque_producer_boundaries, 1);
    assert_eq!(receipt.graph_encoder_batches, 1);
    assert_eq!(receipt.graph_submission_boundaries, 1);
    assert!(
        receipt
            .logical_plan_dump
            .contains("rasterizer=Classic execution_boundary=opaque_submission")
    );
    println!(
        "{{\"shared_device\":true,\"tenant_format\":\"Rgba8UnormSrgb\",\"master_format\":\"Rgba8Unorm\",\"tenant_name\":\"{}\",\"producer_path\":\"{}\",\"fallback_count\":{},\"scene_op_boundary\":{},\"logical_opaque_producer_boundaries\":{},\"graph_encoder_batches\":{},\"graph_submission_boundaries\":{},\"caller_reported_physical_submission_count\":1,\"tenant_internal_submissions\":{},\"capture_distinct_colours\":{},\"plan_dump\":{:?},\"dependency_provenance\":\"mere tenant 4d8bd703 encode -> netrender 9607d16f\"}}",
        receipt.tenant_name,
        receipt.producer_path,
        receipt.fallback_count,
        receipt.scene_op_boundary,
        receipt.logical_opaque_producer_boundaries,
        receipt.graph_encoder_batches,
        receipt.graph_submission_boundaries,
        candidate_report.internal_submissions,
        candidate.distinct,
        receipt.logical_plan_dump,
    );
}
