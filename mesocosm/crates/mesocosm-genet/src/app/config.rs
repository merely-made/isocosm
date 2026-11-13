// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's configuration: every flag `main.rs` can set.

use std::path::PathBuf;

use mesocosm_runtime::continuation::Rounds;

use crate::section::{self, CameraMode};

#[derive(Clone, Debug)]
pub struct HostConfig {
    /// H6's explicit dev world, played through the core and Assembly (897).
    pub bench_world: bool,
    /// Developer comparison of derived native component stores; no world rules change.
    pub component_scene: bool,
    pub bench_epoch: u64,
    /// A playable A1 world: the cheap structural check, no full witness, and
    /// an epoch short enough to reach the board in a sitting.
    pub play: bool,
    pub body_roster: bool,
    pub body_founding: Option<String>,
    /// None uses varied proportions for ordinary play, authored for explicit presets.
    pub body_proportions: Option<bool>,
    /// New-play territory; false retains the historical bounded-patch instrument.
    pub habitat_site: bool,
    pub play_epoch: u64,
    /// The hour of day a play world begins at.
    pub play_hour: u32,
    /// The share of founding soil a play world starts with, in percent: soil
    /// is most of the matter, and matter bounds how far the world booms.
    pub play_soil: u64,
    pub seed: u64,
    /// Select the existing native clock-sourced founding seed instead of this explicit seed.
    pub unchosen_seed: bool,
    /// Maximum deterministic seed candidates for the ordinary A1 default.
    pub candidate_attempts: u32,
    pub start: Option<cosmograph::directing::interim::Start>,
    pub mode: Option<cosmograph::directing::readings::Mode>,
    /// The generated world's founding population.
    pub population: u64,
    pub ticks_per_second: u32,
    pub render_scale: u32,
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
    pub view_depth: Option<f32>,
    pub terrain_style: section::TerrainStyle,
    /// Generated voxels, declared boxes or capsules, presentation only.
    pub body_mode: section::BodyMode,
    pub body_budget: usize,
    /// Presentation-only scale of the body tenant's sun and ambient (1 matches the terrain).
    pub body_light: f32,
    /// The hybrid rung's sun quantum (`section::DEFAULT_LIGHT_STEP`).
    pub light_step: f32,
    /// The shared body pass's display encoding (case 1176).
    pub body_display: isometer::BodyDisplay,
    /// Where the sky's light comes from; a bench reads the fixed sun unless told otherwise.
    pub sun: section::SunSource,
    /// Off by default (DT1); recorded in the receipt either way.
    pub dev: bool,
    /// Dev clock starts held before window/renderer warmup.
    pub start_paused: bool,
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
    pub ecology_rates: Option<cosmograph::ecology::Rates>,
    /// How a round runs past its span (lineage-rounds D7); capped by default.
    pub rounds: Rounds,
    /// Each epoch's par is what the played line left alone would hold (D8).
    pub condition: bool,
    /// The world rules for a new generated world: all off but the crowding
    /// brake, the consumers' limits and rot, which are on unless `--no-dormancy`,
    /// `--no-limits` or `--no-rot` (D2, D9, D12, D27; 2026-11-11, 2026-11-12).
    pub world_rules: WorldRules,
}

pub use mesocosm_runtime::world_rules::WorldRules;

impl HostConfig {
    /// Why the round flags cannot apply to this startup, if they cannot: like
    /// `--start`, they are for an ordinary new generated world.
    pub fn round_flags_refused(&self, replay: bool, bench: bool) -> Option<&'static str> {
        let set = self.rounds != Rounds::Capped || self.condition || self.world_rules != WorldRules::generated();
        (set && (self.watch.is_some() || self.spread.is_some() || self.ecology_candidate.is_some()
            || self.ecology_witness.is_some() || replay || bench)).then_some(
            "--rounds, --condition, --drift, --draft, --sheet, --climate, --regard, --exertion, --hosting, --learning, --mass-law, --dormancy, --no-dormancy, --limits, --no-limits, --rot, --no-rot, --spoilage and --turn require an ordinary new generated world; \
a resumed save, authored spread, candidate, witness, replay or bench declares its own")
    }

    pub fn parse_rounds(value: &str) -> Option<Rounds> {
        match value {
            "capped" => Some(Rounds::Capped),
            "waiting" => Some(Rounds::Waiting),
            "continuous" => Some(Rounds::Continuous),
            _ => None,
        }
    }

    pub fn ordinary_ecology_rates(&self) -> cosmograph::ecology::Rates {
        self.ecology_rates.unwrap_or_else(|| {
            if self.ecology_witness.is_some() {
                cosmograph::ecology::Rates::Stock
            } else {
                cosmograph::ecology::Rates::A1
            }
        })
    }

    pub fn parse_ecology_rates(value: &str) -> Option<cosmograph::ecology::Rates> {
        match value {
            "stock" => Some(cosmograph::ecology::Rates::Stock),
            "a1" => Some(cosmograph::ecology::Rates::A1),
            _ => None,
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            bench_world: false,
            component_scene: false,
            bench_epoch: 60,
            play: false,
            body_roster: false,
            body_founding: None,
            body_proportions: None,
            habitat_site: true,
            play_epoch: 60,
            play_hour: 7,
            play_soil: 100,
            seed: 0x00A7_7AC4,
            unchosen_seed: false,
            candidate_attempts: 128,
            start: None,
            mode: None,
            population: cosmograph::Founding::default().population,
            // TD2's 10 a second (2026-08-29), raised from case 978's 2 once
            // decision periods per kind brought a played tick near 20 ms
            // (the execution plan's D21).
            ticks_per_second: 10,
            render_scale: 1,
            width: 960,
            height: 540,
            frames: None,
            capture: None,
            trace: None,
            receipt: None,
            scenario: None,
            slab_half_height: section::SLAB_HALF_HEIGHT,
            camera: CameraMode::default(),
            view_depth: None,
            terrain_style: section::TerrainStyle::Auto,
            // Hybrid stays opt-in until the zoom dither stops re-rendering the
            // whole scene and the contact gait's loss is settled (case 1175).
            body_mode: section::BodyMode::default(),
            body_budget: section::DEFAULT_BODY_BUDGET,
            body_light: 1.0,
            light_step: section::DEFAULT_LIGHT_STEP,
            body_display: isometer::BodyDisplay::Srgb,
            sun: section::SunSource::Sim,
            dev: false,
            start_paused: false,
            follow: None,
            watch: None,
            spatial: None,
            graft: None,
            spread: None,
            ecology_witness: None,
            ecology_candidate: None,
            ecology_rates: None,
            rounds: Rounds::Capped,
            condition: false,
            world_rules: WorldRules::generated(),
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
        assert_eq!(
            config.ordinary_ecology_rates(),
            cosmograph::ecology::Rates::A1
        );
        assert_eq!(
            cosmograph::Founding::default().ecology_rates,
            cosmograph::ecology::Rates::Stock
        );
        config.ecology_witness = Some("authored-875.json".into());
        assert_eq!(
            config.ordinary_ecology_rates(),
            cosmograph::ecology::Rates::Stock
        );
        config.ecology_rates = Some(cosmograph::ecology::Rates::A1);
        assert_eq!(
            config.ordinary_ecology_rates(),
            cosmograph::ecology::Rates::A1
        );
        config.ecology_rates = Some(cosmograph::ecology::Rates::Stock);
        assert_eq!(
            config.ordinary_ecology_rates(),
            cosmograph::ecology::Rates::Stock
        );
        assert_eq!(
            Some(HostConfig::parse_ecology_rates("stock").unwrap()),
            Some(cosmograph::ecology::Rates::Stock)
        );
        assert_eq!(
            Some(HostConfig::parse_ecology_rates("a1").unwrap()),
            Some(cosmograph::ecology::Rates::A1)
        );
        assert_eq!(HostConfig::parse_ecology_rates("A1"), None);
    }

    #[test]
    fn round_flags_default_off_and_parse_their_words() {
        let config = HostConfig::default();
        assert_eq!((config.rounds, config.condition, config.world_rules),
            (Rounds::Capped, false, WorldRules::generated()));
        assert_eq!(HostConfig::parse_rounds("waiting"), Some(Rounds::Waiting));
        assert_eq!(HostConfig::parse_rounds("continuous"), Some(Rounds::Continuous));
        assert_eq!(HostConfig::parse_rounds("capped"), Some(Rounds::Capped));
        assert_eq!(HostConfig::parse_rounds("Waiting"), None);
        assert_eq!(WorldRules::all(), WorldRules { drift: true, draft: true, sheet: true, ..Default::default() });
    }

    #[test]
    fn world_rules_set_only_what_is_asked() {
        let mut genesis = mesocosm_runtime::Founded::generated(9, 256).unwrap().genesis;
        let before = genesis.rules.directing().clone();
        WorldRules::default().apply(&mut genesis).unwrap();
        assert_eq!(genesis.rules.directing(), before);
        WorldRules { drift: true, ..Default::default() }.apply(&mut genesis).unwrap();
        let d = genesis.rules.directing();
        assert!(d.drift && d.draft.is_none() && !d.sheet);
        WorldRules::all().apply(&mut genesis).unwrap();
        let d = genesis.rules.directing();
        assert!(d.drift && d.sheet);
        assert_eq!(d.draft, Some(cosmograph::rules::Draft::default()));
        assert!(genesis.rules.climate.is_none(), "the rounds' rules leave the climate alone");
        WorldRules { climate: true, ..Default::default() }.apply(&mut genesis).unwrap();
        assert_eq!(genesis.rules.climate, Some(cosmograph::rules::Climate::default()));
        assert!(genesis.rules.directing().regard.is_none(), "regard is its own flag");
        WorldRules { regard: true, ..Default::default() }.apply(&mut genesis).unwrap();
        assert_eq!(genesis.rules.directing().regard, Some(cosmograph::rules::Regard::default()));
    }

    #[test]
    fn hosting_learning_and_turn_are_off_by_default_and_each_checked() {
        let found = || mesocosm_runtime::Founded::generated(9, 256).unwrap().genesis;
        let plain = found();
        let mut genesis = found();
        WorldRules::default().apply(&mut genesis).unwrap();
        assert!(genesis == plain, "no flag, no change");
        WorldRules { hosting: true, ..Default::default() }.apply(&mut genesis).unwrap();
        let rules = &genesis.rules;
        assert!(rules.hosting.is_some() && ["gene:rot", "gene:feed", "gene:leak", "gene:enter"].iter()
            .all(|gene| rules.traits.contains(*gene)));
        assert!(genesis.lineages.contains_key(mesocosm_runtime::microbe::KEY));
        genesis.validate().unwrap();
        let mut genesis = found();
        assert!(WorldRules { turn: Some(3), ..Default::default() }.apply(&mut genesis).unwrap_err().contains("--climate"));
        WorldRules { climate: true, turn: Some(3), ..Default::default() }.apply(&mut genesis).unwrap();
        assert_eq!(genesis.rules.climate.as_ref().and_then(|c| c.turn), Some(3));
        let learning = WorldRules { learning: true, ..Default::default() };
        assert!(plain.rules.directing().policy == Default::default() && plain.lineages.values().all(|l| l.policy.is_none()));
        let mut learned = found();
        learning.apply(&mut learned).unwrap();
        let d = learned.rules.directing();
        assert_eq!(d.learning, Some(cosmograph::rules::Learning::default()));
        assert_eq!(d.policy, cosmograph::rules::PolicyVariation { founding: 0, expression: 0, candidates: 1 });
        let needs = learned.rules.mind.as_ref().unwrap().needs.len();
        let deciding: Vec<_> = learned.lineages.values().filter(|l| l.decides.is_some()).collect();
        assert!(!deciding.is_empty());
        assert!(deciding.iter().all(|l| l.policy == Some(cosmograph::directing::policy::LineagePolicy::founding(needs))));
        assert!(learned.lineages.values().filter(|l| l.decides.is_none()).all(|l| l.policy.is_none()));
        learned.validate().unwrap();
        // A world that already sets its gate keeps it.
        let mut gated = found();
        gated.rules.directing.get_or_insert_with(Default::default).policy =
            cosmograph::rules::PolicyVariation { founding: 3, expression: 2, candidates: 4 };
        learning.apply(&mut gated).unwrap();
        assert_eq!(gated.rules.directing().policy.founding, 3);
    }

    #[test]
    fn round_flags_are_refused_where_start_is() {
        let mut config = HostConfig::default();
        assert_eq!(config.round_flags_refused(true, true), None, "nothing set, nothing refused");
        config.rounds = Rounds::Waiting;
        assert_eq!(config.round_flags_refused(false, false), None);
        assert!(config.round_flags_refused(true, false).is_some());
        assert!(config.round_flags_refused(false, true).is_some());
        config.watch = Some("save.json".into());
        assert!(config.round_flags_refused(false, false).is_some());
        let mut config = HostConfig { condition: true, ..Default::default() };
        config.spread = Some("spread.json".into());
        assert!(config.round_flags_refused(false, false).is_some());
        let config = HostConfig { world_rules: WorldRules::all(), ecology_witness: Some("w.json".into()), ..Default::default() };
        assert!(config.round_flags_refused(false, false).is_some());
    }
}
