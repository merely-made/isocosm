// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The one light block terrain and bodies read (ruling 472): made once from
//! the sim's sun (971), then projected to the tracer's slot and the tenant's
//! `LightBlock`. Both projections come from the same three numbers, so they
//! cannot disagree.

use isometer::lens::TraceLight;
use isocosm::light::Light;
use tenant::{Ambient, LightBlock, Sun};

/// The sun's weight in the terrain's Lambert term at full strength: the
/// tracer's literal 0.62 (ruling 970); ambient is the light's own.
const SUN_WEIGHT: f32 = 0.62;
/// Ruling 970's fixed sun, toward the sun, for a bench with no sim.
const FIXED_TOWARD_SUN: [f32; 3] = [0.4, 0.8, 0.3];

/// Direction toward the sun, ambient, and the sun's per-channel weight
/// (zero at night, where `toward` is straight up and unused).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub toward: [f32; 3],
    pub ambient: f32,
    pub sun: [f32; 3],
}

impl Block {
    /// Ruling 970's fixed sun: the bench default.
    pub fn fixed() -> Self {
        Self { toward: FIXED_TOWARD_SUN, ambient: 0.38, sun: [SUN_WEIGHT; 3] }
    }

    pub fn from_sim(light: Light) -> Self {
        match light.sun {
            Some(sun) => Self {
                toward: sun.direction,
                ambient: light.ambient,
                sun: sun.colour.map(|c| c * sun.intensity * SUN_WEIGHT),
            },
            None => Self { toward: [0.0, 1.0, 0.0], ambient: light.ambient, sun: [0.0; 3] },
        }
    }

    pub fn trace(self) -> TraceLight {
        TraceLight { toward_sun: self.toward, ambient: self.ambient, sun: self.sun }
    }

    /// For the tenant, both terms scaled by `--body-light` (1 matches the
    /// terrain, 0 is dark). kiss3d's diffuse is albedo / pi * N.L * intensity,
    /// so the sun's intensity carries the pi that makes it the Lambert term.
    pub fn tenant(self, scale: f32) -> LightBlock {
        let strongest = self.sun.into_iter().fold(0.0, f32::max);
        let colour = if strongest > 0.0 { self.sun.map(|c| c / strongest) } else { [1.0; 3] };
        LightBlock {
            sun: Some(Sun {
                direction: self.toward.map(|c| -c),
                color: colour,
                intensity: strongest * std::f32::consts::PI * scale,
                casts_shadows: false,
            }),
            ambient: Ambient { color: [1.0; 3], intensity: self.ambient * scale },
            ..Default::default()
        }
    }
}

/// Where the sky comes from: the sim at the presented tick, the sim at a
/// named tick (a bench's dawn, noon, dusk or night), or ruling 970's fixed sun.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SunSource {
    Sim,
    Fixed,
    At(u64),
}

impl SunSource {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "sim" => Some(Self::Sim),
            "fixed" => Some(Self::Fixed),
            tick => tick.parse().ok().map(Self::At),
        }
    }

    pub fn block(self, rules: &isocosm::rules::Rules, tick: u64) -> Block {
        match self {
            Self::Fixed => Block::fixed(),
            Self::Sim => Block::from_sim(isocosm::light::at(rules, tick, None)),
            Self::At(at) => Block::from_sim(isocosm::light::at(rules, at, None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isocosm::{Founding, light, preset::Preset};

    fn rules(preset: Option<Preset>) -> isocosm::rules::Rules {
        Founding { seed: 9, sites: 3, population: 24, cohort_size: 8, lineages: 2, preset,
            ..Default::default() }.generate().unwrap().rules
    }

    #[test]
    fn the_fixed_block_is_the_tracers_default() {
        let fixed = Block::fixed().trace();
        let tracer = TraceLight::default();
        assert_eq!((fixed.toward_sun, fixed.ambient, fixed.sun),
            (tracer.toward_sun, tracer.ambient, tracer.sun));
        // And the sim's tidal lock reproduces it.
        let locked = rules(Some(Preset::TidalShelf));
        let block = Block::from_sim(light::at(&locked, 12_345, None));
        let len = FIXED_TOWARD_SUN.iter().map(|c| c * c).sum::<f32>().sqrt();
        assert!(block.toward.iter().zip(FIXED_TOWARD_SUN).all(|(a, b)| (a - b / len).abs() < 1e-6));
        assert!((block.ambient - 0.38).abs() < 1e-6);
        assert!(block.sun.iter().all(|c| (c - SUN_WEIGHT).abs() < 1e-6));
    }

    /// The CPU control (972): across a day and a season, tracer and tenant
    /// receive the same direction, ambient and sun weight from the block.
    #[test]
    fn tracer_and_tenant_receive_identical_values() {
        for preset in [None, Some(Preset::HeavyDeep), Some(Preset::LongYear)] {
            let rules = rules(preset);
            for tick in (0..3 * 1_440).step_by(37).chain([131_490, 394_470]) {
                let block = Block::from_sim(light::at(&rules, tick, None));
                let (trace, lit) = (block.trace(), block.tenant(1.0));
                assert!(lit.is_valid(), "tick {tick}");
                let sun = lit.sun.expect("sun");
                assert!(sun.direction.iter().zip(trace.toward_sun).all(|(d, t)| *d == -t));
                assert_eq!(lit.ambient.intensity, trace.ambient);
                for channel in 0..3 {
                    let weight = sun.color[channel] * sun.intensity / std::f32::consts::PI;
                    assert!((weight - trace.sun[channel]).abs() < 1e-6, "tick {tick}");
                }
            }
        }
    }

    #[test]
    fn night_is_ambient_only_and_body_light_scales_both() {
        let night = Block::from_sim(light::at(&rules(None), 0, None));
        assert_eq!(night.sun, [0.0; 3]);
        assert!(night.ambient < 0.38);
        assert_eq!(night.tenant(1.0).sun.expect("sun").intensity, 0.0);
        let dark = Block::fixed().tenant(0.0);
        assert!(dark.is_valid());
        assert_eq!(dark.ambient.intensity, 0.0);
        let peak = Block::fixed().tenant(1.0);
        let sun = peak.sun.expect("sun");
        assert!((peak.ambient.intensity + sun.intensity / std::f32::consts::PI - 1.0).abs() < 1e-6);
    }
}
