// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's configuration: every flag `main.rs` can set.

use std::path::PathBuf;

use crate::section::{self, CameraMode};

#[derive(Clone, Debug)]
pub struct HostConfig {
    pub seed: u64,
    /// The generated world's founding population.
    pub population: u64,
    pub ticks_per_second: u32,
    pub width: u32,
    pub height: u32,
    /// Run this many frames and exit.
    pub frames: Option<u32>,
    /// Write the last frame here before exiting.
    pub capture: Option<PathBuf>,
    /// Write the session's save here before exiting.
    pub trace: Option<PathBuf>,
    /// Write the run's receipt here before exiting.
    pub receipt: Option<PathBuf>,
    /// The scenario text driving this run, when `--scenario` gave it one (DT4).
    pub scenario: Option<String>,
    /// Half the height of the section's slab, in voxels. Presentation only.
    pub slab_half_height: f32,
    /// Which way the section looks (DC4). Presentation only.
    pub camera: CameraMode,
    pub terrain_style: section::TerrainStyle,
    /// Voxel anatomy or the capsule comparison, presentation only.
    pub body_mode: section::BodyMode,
    pub body_budget: usize,
    /// Off by default (DT1); recorded in the receipt either way.
    pub dev: bool,
    /// Which body the camera starts on (DT2); presentation only.
    pub follow: Option<u64>,
    /// A recorded save to take up and play on from (786).
    pub watch: Option<PathBuf>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            seed: 0x00A7_7AC4,
            population: isocosm::Founding::default().population,
            // The canonical played tempo (TD2, ruled 2026-08-29).
            ticks_per_second: 10,
            width: 960,
            height: 540,
            frames: None,
            capture: None,
            trace: None,
            receipt: None,
            scenario: None,
            slab_half_height: section::SLAB_HALF_HEIGHT,
            camera: CameraMode::default(),
            terrain_style: section::TerrainStyle::Auto,
            body_mode: section::BodyMode::default(),
            body_budget: section::DEFAULT_BODY_BUDGET,
            dev: false,
            follow: None,
            watch: None,
        }
    }
}
