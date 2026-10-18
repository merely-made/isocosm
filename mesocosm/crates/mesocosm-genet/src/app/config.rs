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
    /// H6's explicit dev world, played through the core and Assembly (897).
    pub bench_world: bool,
    pub bench_epoch: u64,
    pub seed: u64,
    /// Select the existing native clock-sourced founding seed instead of this explicit seed.
    pub unchosen_seed: bool,
    /// Maximum deterministic seed candidates for the ordinary A1 default.
    pub candidate_attempts: u32,
    pub start: Option<isocosm::directing::interim::Start>,
    pub mode: Option<isocosm::directing::readings::Mode>,
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
    /// Presentation-only scale of the body tenant's sun and ambient (1 matches the terrain).
    pub body_light: f32,
    /// Where the sky's light comes from; a bench reads the fixed sun unless told otherwise.
    pub sun: section::SunSource,
    /// Off by default (DT1); recorded in the receipt either way.
    pub dev: bool,
    /// Which body the camera starts on (DT2); presentation only.
    pub follow: Option<u64>,
    /// A recorded save to take up and play on from (786).
    pub watch: Option<PathBuf>,
    /// Authored physical inputs for already admitted native bodies and patches.
    pub spatial: Option<PathBuf>,
    pub graft: Option<PathBuf>,
    /// Explicit new founding and logged author spread startup; never a player packet.
    pub spread: Option<PathBuf>,
    /// Complete authored joint witness over an exact generated candidate.
    pub ecology_witness: Option<PathBuf>,
    /// Complete explicit generated candidate; never stripped to regenerated rules.
    pub ecology_candidate: Option<PathBuf>,
    /// Optional native ecology profile. New ordinary worlds default to A1;
    /// an existing authored witness retains its historical Stock declaration.
    pub ecology_rates: Option<isocosm::ecology::Rates>,
}

impl HostConfig {
    pub fn ordinary_ecology_rates(&self) -> isocosm::ecology::Rates {
        self.ecology_rates.unwrap_or_else(|| {
            if self.ecology_witness.is_some() { isocosm::ecology::Rates::Stock }
            else { isocosm::ecology::Rates::A1 }
        })
    }

    pub fn parse_ecology_rates(value: &str) -> Option<isocosm::ecology::Rates> {
        match value {
            "stock" => Some(isocosm::ecology::Rates::Stock),
            "a1" => Some(isocosm::ecology::Rates::A1),
            _ => None,
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            bench_world: false,
            bench_epoch: 60,
            seed: 0x00A7_7AC4,
            unchosen_seed: false,
            candidate_attempts: 8,
            start: None,
            mode: None,
            population: isocosm::Founding::default().population,
            // The default tempo follows the measured tick (wing ruling 978,
            // 2026-10-17): about 0.45 s per tick after the core tick lane, so 2
            // ticks per second keeps the clock from deferring without limit.
            // TD2's 10 (2026-08-29) stays the aspiration.
            ticks_per_second: 2,
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
            body_light: 1.0,
            sun: section::SunSource::Sim,
            dev: false,
            follow: None,
            watch: None,
            spatial: None,
            graft: None,
            spread: None,
            ecology_witness: None,
            ecology_candidate: None,
            ecology_rates: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_a1_default_preserves_explicit_stock_and_authored_witnesses() {
        let mut config = HostConfig::default();
        assert_eq!(config.ecology_rates, None);
        assert_eq!(config.ordinary_ecology_rates(), isocosm::ecology::Rates::A1);
        assert_eq!(isocosm::Founding::default().ecology_rates, isocosm::ecology::Rates::Stock);
        config.ecology_witness = Some("authored-875.json".into());
        assert_eq!(config.ordinary_ecology_rates(), isocosm::ecology::Rates::Stock);
        config.ecology_rates = Some(isocosm::ecology::Rates::A1);
        assert_eq!(config.ordinary_ecology_rates(), isocosm::ecology::Rates::A1);
        config.ecology_rates = Some(isocosm::ecology::Rates::Stock);
        assert_eq!(config.ordinary_ecology_rates(), isocosm::ecology::Rates::Stock);
        assert_eq!(Some(HostConfig::parse_ecology_rates("stock").unwrap()),
            Some(isocosm::ecology::Rates::Stock));
        assert_eq!(Some(HostConfig::parse_ecology_rates("a1").unwrap()),
            Some(isocosm::ecology::Rates::A1));
        assert_eq!(HostConfig::parse_ecology_rates("A1"), None);
    }
}
