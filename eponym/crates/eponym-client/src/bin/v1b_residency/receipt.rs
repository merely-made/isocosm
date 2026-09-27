// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1b receipt serialization and recovery measurements.

use super::*;
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Serialize)]
pub(super) struct FrameSample {
    pub(super) frame: u64,
    pub(super) projection_revision: u64,
    pub(super) camera_distance: f32,
    pub(super) visible_range: i32,
    pub(super) resident_range: i32,
    pub(super) page_transition: bool,
    pub(super) page_prepare_us: u64,
    pub(super) loaded_bricks: usize,
    pub(super) evicted_bricks: usize,
    pub(super) resident_bricks: usize,
    pub(super) brick_upload_bytes: u64,
    pub(super) tracer_cpu_prepare_us: u64,
    pub(super) resource_creations: u32,
    pub(super) bind_group_rebuilds: u32,
    pub(super) projection_replaced: bool,
    pub(super) frame_us: u64,
}

#[derive(Serialize)]
struct Recovery {
    event_frame: u64,
    resident_range: i32,
    steady_median_us: u64,
    threshold_us: u64,
    event_frame_us: u64,
    recovered_at_frame: Option<u64>,
    recovery_frames: Option<u64>,
}

#[derive(Serialize)]
struct Receipt<'a> {
    gate: &'static str,
    vessel: &'static str,
    camera_profile: &'static str,
    traversal_implementation: &'static str,
    resident_measure: &'static str,
    publication_mode: &'static str,
    adapter: &'a str,
    size: [u32; 2],
    frames: usize,
    world_bricks: usize,
    ground_revision: u64,
    resident_budget_bytes: u64,
    device_max_texture_dimension_3d: u32,
    cache_capacity_bricks: usize,
    fixed_resident_bytes: u64,
    fixed_pointer_extent: [u32; 3],
    fixed_atlas_extent: [u32; 3],
    allocator_bytes_after_first_frame: Option<u64>,
    allocator_bytes_after_last_frame: Option<u64>,
    allocator_growth_bytes: Option<i64>,
    retargets: usize,
    travel_frame: u64,
    total_brick_upload_bytes: u64,
    frame_us_min: u64,
    frame_us_median: u64,
    frame_us_max: u64,
    steady_frame_us_median: u64,
    rapid_close_recovery: Recovery,
    rapid_far_recovery: Recovery,
    capture: &'a str,
    capture_distinct_colours: usize,
    samples: &'a [FrameSample],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn report(
    composer: &Composer,
    master: &wgpu::Texture,
    adapter: &str,
    scene: &ResidencyScene,
    stable: &StableResidency,
    samples: &[FrameSample],
    allocator_after_first: Option<u64>,
    allocator_after_last: Option<u64>,
    device_max_texture_dimension_3d: u32,
) {
    let capture = composer.capture(master);
    capture.write_png(Path::new(CAPTURE)).expect("V1b capture");
    assert!(
        !capture.is_trivial(),
        "V1b capture has only {} distinct colours",
        capture.distinct
    );
    let frame_spans: Vec<_> = samples.iter().map(|sample| sample.frame_us).collect();
    let steady_spans: Vec<_> = samples
        .iter()
        .filter(|sample| !sample.page_transition)
        .map(|sample| sample.frame_us)
        .collect();
    let rapid_close_recovery = recovery(samples, 48);
    let rapid_far_recovery = recovery(samples, 60);
    assert!(
        rapid_close_recovery.recovered_at_frame.is_some()
            && rapid_far_recovery.recovered_at_frame.is_some(),
        "both rapid zooms must recover within the V1b trace"
    );
    let allocator_growth_bytes = allocator_after_first
        .zip(allocator_after_last)
        .map(|(first, last)| last as i64 - first as i64);
    if let Some(growth) = allocator_growth_bytes {
        // Driver-internal staging metadata moves by a few bytes between
        // samples; a leaked page or texture would be half a megabyte. The
        // tolerance is far below one brick slot.
        assert!(
            growth.abs() <= 4096,
            "the allocator moved {growth} bytes after the first frame; the cache is not stable"
        );
    }
    let receipt = Receipt {
        gate: "V1b",
        vessel: "paredros",
        camera_profile: "third-person continuous zoom: near acts, mid leads, far plans",
        traversal_implementation: "modulus::BRICK_DDA_WGSL via isometer::lens::BrickTracer",
        resident_measure: "fixed pointer plus atlas allocation, with wgpu allocator-report bytes",
        publication_mode: "one capacity-fixed cache; retargets publish the pointer volume plus \
                           loaded slots only, retained slots never re-upload",
        adapter,
        size: SIZE,
        frames: samples.len(),
        world_bricks: scene.ground.brick_count(),
        ground_revision: scene.ground.revision(),
        resident_budget_bytes: RESIDENT_BUDGET_BYTES,
        device_max_texture_dimension_3d,
        cache_capacity_bricks: stable.capacity(),
        fixed_resident_bytes: stable.resident_bytes(),
        fixed_pointer_extent: stable.map().pointer_extent(),
        fixed_atlas_extent: stable.map().atlas_extent(),
        allocator_bytes_after_first_frame: allocator_after_first,
        allocator_bytes_after_last_frame: allocator_after_last,
        allocator_growth_bytes,
        retargets: samples
            .iter()
            .filter(|sample| sample.page_transition)
            .count(),
        travel_frame: TRAVEL_FRAME,
        total_brick_upload_bytes: samples.iter().map(|sample| sample.brick_upload_bytes).sum(),
        frame_us_min: *frame_spans.iter().min().expect("V1b frames"),
        frame_us_median: median(&frame_spans),
        frame_us_max: *frame_spans.iter().max().expect("V1b frames"),
        steady_frame_us_median: median(&steady_spans),
        rapid_close_recovery,
        rapid_far_recovery,
        capture: CAPTURE,
        capture_distinct_colours: capture.distinct,
        samples,
    };
    let json = serde_json::to_string_pretty(&receipt).expect("V1b receipt JSON");
    std::fs::write(RECEIPT, &json).expect("write V1b receipt");
    println!("{json}");
}

fn recovery(samples: &[FrameSample], event_frame: u64) -> Recovery {
    let event = samples
        .iter()
        .find(|sample| sample.frame == event_frame)
        .expect("rapid zoom frame");
    let steady: Vec<_> = samples
        .iter()
        .filter(|sample| sample.resident_range == event.resident_range && !sample.page_transition)
        .map(|sample| sample.frame_us)
        .collect();
    let steady_median_us = median(&steady);
    let threshold_us = steady_median_us + steady_median_us / 4;
    let recovered_at_frame = samples
        .iter()
        .find(|sample| {
            sample.frame > event_frame
                && sample.resident_range == event.resident_range
                && sample.frame_us <= threshold_us
        })
        .map(|sample| sample.frame);
    Recovery {
        event_frame,
        resident_range: event.resident_range,
        steady_median_us,
        threshold_us,
        event_frame_us: event.frame_us,
        recovered_at_frame,
        recovery_frames: recovered_at_frame.map(|frame| frame - event_frame),
    }
}

fn median(values: &[u64]) -> u64 {
    assert!(!values.is_empty(), "a median needs samples");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}
