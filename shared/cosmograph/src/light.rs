// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sun, the simulator's first light field (rulings 971 to 973): a pure
//! function of the world's rules, the tick and the site. Nothing is stored,
//! so it moves neither replay nor the state hash. Presentation reads it as
//! one light block for terrain and bodies; the ecology scales its sunlight
//! credit by `Sun::intensity` (972, not yet wired).
//!
//! Frame: x east, y up, z south. A day is 24 hours of world time and a year
//! the rules' year, both counted in the world's tick unit, so a minute tick
//! gives 1,440 ticks a day (973). Tick 0 is midnight on the equinox.

use crate::{
    rules::{Rules, year_ticks},
    schema::{Site, Tick},
};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// A day in microseconds of world time.
pub const DAY_MICROSECONDS: u64 = 86_400_000_000;

/// The site condition naming a latitude, in hundredths of a degree north.
/// Sites without it sit at [`DEFAULT_LATITUDE`].
pub const LATITUDE: &str = "light:latitude";
/// Where a site is when its world's grid carries no latitude: 40 degrees north.
pub const DEFAULT_LATITUDE: i64 = 4_000;

/// Ruling 970's fixed sun, toward the sun: the tracer's literal, unnormalised.
pub const FIXED_TOWARD_SUN: [f64; 3] = [0.4, 0.8, 0.3];
/// Ambient light with the sun at full strength, and in full night. Day's
/// value is the tracer's 0.38 (ruling 970); the sun adds up to 0.62 more.
pub const DAY_AMBIENT: f32 = 0.38;
pub const NIGHT_AMBIENT: f32 = 0.10;

/// How a world's sun moves and shines. Integers keep the rules `Eq`.
/// Absent from `Rules` means [`SunRules::default`], and worlds without it
/// serialize and hash as before the field existed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SunRules {
    /// The sun never moves: a fixed terminator, no day and no year.
    pub tidal_lock: bool,
    /// The season's amplitude, the axial tilt in hundredths of a degree.
    pub tilt: u32,
    /// Strength of the sun as a percent of a bright one.
    pub intensity_percent: u32,
    /// The sun's colour in thousandths, linear RGB.
    pub colour: [u32; 3],
    /// The hour of day at tick 0, 0 to 23: a world may begin in the morning
    /// rather than at midnight. Absent reads as midnight.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub start_hour: u32,
}

fn is_zero(hour: &u32) -> bool { *hour == 0 }

impl Default for SunRules {
    fn default() -> Self {
        Self {
            tidal_lock: false,
            tilt: 2_350,
            intensity_percent: 100,
            colour: [1_000; 3],
            start_hour: 0,
        }
    }
}

/// The sun at one moment. Direction points toward the sun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sun {
    pub direction: [f32; 3],
    /// 0 to 1: the sun's strength, rising through twilight; geometry (N.L)
    /// is the consumer's.
    pub intensity: f32,
    pub colour: [f32; 3],
}

/// What the sky gives a site at a tick. `sun` is `None` below the horizon;
/// `ambient` (0 to 1) carries the night.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub sun: Option<Sun>,
    pub ambient: f32,
}

fn smoothstep(low: f64, high: f64, x: f64) -> f64 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The light at `site` on `tick`. `site` is `None` for a world-scale read,
/// which takes the default latitude.
pub fn at(rules: &Rules, tick: Tick, site: Option<&Site>) -> Light {
    let sun_rules = rules.sun.clone().unwrap_or_default();
    let strength = f64::from(sun_rules.intensity_percent) / 100.0;
    let ambient_by_day = NIGHT_AMBIENT + (DAY_AMBIENT - NIGHT_AMBIENT) * strength as f32;
    let colour = sun_rules.colour.map(|c| c as f32 / 1_000.0);

    let toward = if sun_rules.tidal_lock {
        let [x, y, z] = FIXED_TOWARD_SUN;
        let len = (x * x + y * y + z * z).sqrt();
        [x / len, y / len, z / len]
    } else {
        let unit = rules.tick_microseconds().max(1);
        let day = (DAY_MICROSECONDS / unit).max(1);
        let year = year_ticks(unit);
        // Hour angle: 0 at noon, negative in the morning.
        let tick = tick + u64::from(sun_rules.start_hour % 24) * day / 24;
        let hour = TAU * ((tick % day) as f64 / day as f64 - 0.5);
        let season = TAU * ((tick % year) as f64 / year as f64);
        let declination = (f64::from(sun_rules.tilt) / 100.0).to_radians() * season.sin();
        let latitude = site
            .and_then(|s| s.conditions.get(LATITUDE).copied())
            .unwrap_or(DEFAULT_LATITUDE) as f64
            / 100.0;
        let lat = latitude.to_radians();
        let up = lat.sin() * declination.sin() + lat.cos() * declination.cos() * hour.cos();
        let east = -declination.cos() * hour.sin();
        let north = declination.sin() * lat.cos() - declination.cos() * lat.sin() * hour.cos();
        [east, up, -north]
    };

    let up = toward[1];
    if up <= 0.0 {
        return Light { sun: None, ambient: NIGHT_AMBIENT };
    }
    let rise = if sun_rules.tidal_lock { 1.0 } else { smoothstep(0.0, 0.2, up) };
    let intensity = (rise * strength).clamp(0.0, 1.0) as f32;
    let ambient = if sun_rules.tidal_lock {
        ambient_by_day
    } else {
        let day_share = smoothstep(-0.1, 0.25, up) as f32;
        NIGHT_AMBIENT + (ambient_by_day - NIGHT_AMBIENT) * day_share
    };
    Light {
        sun: Some(Sun {
            direction: toward.map(|c| c as f32),
            intensity,
            colour,
        }),
        ambient,
    }
}
