// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One GPU device for every test in this binary.
//!
//! Sixteen tests each making their own instance and device at once stalled
//! about half of parallel runs, the GPU idle and the CPU spinning (IT2 of
//! `design_docs/archive_docs/2026-11-06/2026-11-06_isometer_tests_plan.md`).
//! The first caller makes the device; the rest share it. `None` when no
//! adapter exists.

use std::sync::OnceLock;

pub(crate) fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    static SHARED: OnceLock<Option<(wgpu::Device, wgpu::Queue)>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
            pollster::block_on(adapter.request_device(&Default::default())).ok()
        })
        .clone()
}
