// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One GPU device for every test in this binary.
//!
//! Sixteen tests each making their own instance and device at once stalled
//! one run in three, all sixteen past a minute, the same livelock isometer's
//! `test_gpu` ends (IT2 of
//! `design_docs/archive_docs/2026-11-06/2026-11-06_isometer_tests_plan.md`).
//! The first caller makes the device; the rest share it. `None` when no
//! adapter exists.

use std::sync::OnceLock;

use crate::renderer::Lens;
use crate::tracer::BrickTracer;

/// The shared handles, the instance and adapter kept for callers that
/// compose with another engine on the same device.
#[derive(Clone)]
pub(crate) struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub(crate) fn gpu() -> Option<Gpu> {
    static SHARED: OnceLock<Option<Gpu>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
            let (device, queue) =
                pollster::block_on(adapter.request_device(&Default::default())).ok()?;
            Some(Gpu {
                instance,
                adapter,
                device,
                queue,
            })
        })
        .clone()
}

/// `BrickTracer::headless` on the shared device.
pub(crate) fn tracer(width: u32, height: u32) -> Option<BrickTracer> {
    let gpu = gpu()?;
    Some(BrickTracer::with_device(
        gpu.device, gpu.queue, width, height,
    ))
}

/// `Lens::headless` on the shared device.
pub(crate) fn lens(width: u32, height: u32) -> Option<Lens> {
    let gpu = gpu()?;
    Some(Lens::with_device(gpu.device, gpu.queue, width, height))
}
