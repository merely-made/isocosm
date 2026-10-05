// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The room's receipts: the capture, its variety, the replay hash and the
//! frame spans.

use super::*;
#[cfg(feature = "r1-proof")]
use eponym_client::gpu::BrickAbi;
use std::path::PathBuf;

/// The receipt: the capture, its variety, the replay hash, and the spans.
pub(super) struct ReportEvidence<'a> {
    pub(super) r1_mode: bool,
    pub(super) frame_us: &'a [u64],
    #[cfg(feature = "r1-proof")]
    pub(super) trace: Option<isometer::lens::BrickDiagnostics>,
    #[cfg(feature = "r1-proof")]
    pub(super) abi: Option<BrickAbi>,
    #[cfg(feature = "r1-proof")]
    pub(super) adapter: &'a str,
    pub(super) opaque_receipt: Option<&'a netrender::OpaqueTenantReceipt>,
}

pub(super) fn report(
    composer: &Composer,
    master: &wgpu::Texture,
    probe: &Probe,
    span: std::time::Duration,
    frames: u64,
    evidence: ReportEvidence<'_>,
) {
    #[cfg(not(feature = "r1-proof"))]
    let _ = (evidence.r1_mode, evidence.frame_us, evidence.opaque_receipt);
    let capture = composer.capture(master);
    #[cfg(feature = "r1-proof")]
    let path = PathBuf::from(if evidence.r1_mode {
        R1_CAPTURE
    } else {
        CAPTURE
    });
    #[cfg(not(feature = "r1-proof"))]
    let path = PathBuf::from(CAPTURE);
    capture.write_png(&path).expect("write the receipt");
    assert!(
        !capture.is_trivial(),
        "the capture is one flat colour ({} distinct); nothing rendered",
        capture.distinct
    );

    println!("--- S0 room probe ---");
    println!("ticks: {} over {frames} frames", probe.tick_count());
    println!("final position: {:?}", probe.at());
    println!("position-log hash: {:#018x}", probe.hash());
    println!("ground hash: {:#018x}", probe.ground_hash());
    println!(
        "capture: {} ({}x{}, {} distinct colours)",
        path.display(),
        capture.size[0],
        capture.size[1],
        capture.distinct
    );
    report_spans(composer, span);
    #[cfg(feature = "r1-proof")]
    if let Some(receipt) = evidence.opaque_receipt {
        let rg3 = Rg3cReceipt {
            gate: "RG3c",
            tenant_name: &receipt.tenant_name,
            producer_path: &receipt.producer_path,
            fallback_count: receipt.fallback_count,
            scene_op_boundary: receipt.scene_op_boundary,
            shared_device: true,
            tenant_format: "Rgba8UnormSrgb",
            master_format: "Rgba8Unorm",
            logical_opaque_producer_boundaries: receipt.logical_opaque_producer_boundaries,
            graph_encoder_batches: receipt.graph_encoder_batches,
            graph_submission_boundaries: receipt.graph_submission_boundaries,
            caller_reported_physical_submission_count: receipt
                .caller_reported_physical_submission_count,
            capture_size: capture.size,
            capture_distinct_colours: capture.distinct,
            capture: path.display().to_string(),
            dependency_provenance: "eponym-client -> netrender RG3a d08f713d8",
            plan_dump: &receipt.logical_plan_dump,
        };
        let json = serde_json::to_string_pretty(&rg3).expect("RG3c receipt JSON");
        std::fs::write(RG3_RECEIPT, &json).expect("write RG3c receipt");
        println!("{json}");
    }
    #[cfg(feature = "r1-proof")]
    if evidence.r1_mode {
        let mut spans = evidence.frame_us.to_vec();
        spans.sort_unstable();
        let diagnostics = evidence.trace.expect("R1 trace diagnostics");
        let receipt = R1Receipt {
            gate: "R1",
            vessel: "paredros",
            camera_profile: "close-perspective-room",
            traversal_implementation: "modulus::BRICK_DDA_WGSL via isometer::lens::BrickTracer",
            adapter: evidence.adapter,
            size: SIZE,
            frames: spans.len(),
            frame_us_min: spans[0],
            frame_us_median: spans[spans.len() / 2],
            frame_us_max: *spans.last().expect("non-empty R1 spans"),
            tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
            steady_brick_upload_bytes: diagnostics.brick_upload_bytes,
            steady_uniform_upload_bytes: diagnostics.uniform_upload_bytes,
            brick_abi: evidence.abi.expect("R1 brick ABI"),
            capture: path.display().to_string(),
            capture_distinct_colours: capture.distinct,
            ground_hash: format!("{:#018x}", probe.ground_hash()),
            position_log_hash: format!("{:#018x}", probe.hash()),
        };
        let json = serde_json::to_string_pretty(&receipt).expect("R1 receipt JSON");
        std::fs::write(R1_RECEIPT, &json).expect("write R1 receipt");
        println!("{json}");
    }
}

#[cfg(feature = "r1-proof")]
#[derive(serde::Serialize)]
pub(super) struct R1Receipt<'a> {
    gate: &'static str,
    vessel: &'static str,
    camera_profile: &'static str,
    traversal_implementation: &'static str,
    adapter: &'a str,
    size: [u32; 2],
    frames: usize,
    frame_us_min: u64,
    frame_us_median: u64,
    frame_us_max: u64,
    tracer_cpu_prepare_us: u64,
    steady_brick_upload_bytes: u64,
    steady_uniform_upload_bytes: u64,
    brick_abi: BrickAbi,
    capture: String,
    capture_distinct_colours: usize,
    ground_hash: String,
    position_log_hash: String,
}

#[cfg(feature = "r1-proof")]
#[derive(serde::Serialize)]
pub(super) struct Rg3cReceipt<'a> {
    gate: &'static str,
    tenant_name: &'a str,
    producer_path: &'a str,
    fallback_count: u64,
    scene_op_boundary: usize,
    shared_device: bool,
    tenant_format: &'static str,
    master_format: &'static str,
    logical_opaque_producer_boundaries: usize,
    graph_encoder_batches: usize,
    graph_submission_boundaries: usize,
    caller_reported_physical_submission_count: Option<u64>,
    capture_size: [u32; 2],
    capture_distinct_colours: usize,
    capture: String,
    dependency_provenance: &'static str,
    plan_dump: &'a str,
}

pub(super) fn report_spans(composer: &Composer, span: std::time::Duration) {
    print!("frame span: probe_frame {:?}", span);
    match composer.net.last_frame_timings() {
        Some(timings) => {
            print!(" | netrender total {:?}", timings.total);
            for named in &timings.spans {
                print!(" | {} {:?}", named.name, named.duration);
            }
        },
        None => print!(" | netrender reported no timings"),
    }
    println!();
}
