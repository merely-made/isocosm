// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{
    DAY_MICROSECONDS, DEFAULT_LATITUDE, LATITUDE, Rules, Site, SunRules, Tick, at, year_ticks,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The authoritative sunlight calculation recorded in a world's rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditRule {
    #[default]
    Legacy,
    FixedV1,
}

impl CreditRule {
    pub(crate) fn is_legacy(&self) -> bool {
        *self == Self::Legacy
    }
}

#[derive(Clone, Copy, Debug)]
enum Intensity {
    Legacy(f64),
    Fixed(u64),
}

impl Intensity {
    fn at(rules: &Rules, tick: Tick, site: Option<&Site>) -> Self {
        match rules.sunlight_credit {
            CreditRule::Legacy => Self::Legacy(
                at(rules, tick, site)
                    .sun
                    .map_or(0.0, |sun| f64::from(sun.intensity)),
            ),
            CreditRule::FixedV1 => Self::Fixed(fixed_intensity(rules, tick, site)),
        }
    }

    fn credit(self, full: u64) -> u64 {
        match self {
            Self::Legacy(intensity) => (full as f64 * intensity).round() as u64,
            Self::Fixed(intensity) => {
                let unit = FULL_INTENSITY as u128;
                ((u128::from(full) * u128::from(intensity) + unit / 2) / unit) as u64
            },
        }
    }
}

pub(crate) fn sunlight_credit(rules: &Rules, tick: Tick, site: Option<&Site>, full: u64) -> u64 {
    Intensity::at(rules, tick, site).credit(full)
}

/// Ecology uses the default latitude and shares its calculation across a tick.
#[derive(Debug, Default)]
pub(crate) struct SunMemo(Mutex<Option<Remembered>>);

type Remembered = (Tick, u64, CreditRule, Option<SunRules>, Intensity);

impl Clone for SunMemo {
    fn clone(&self) -> Self {
        Self(Mutex::new(
            self.0.lock().unwrap_or_else(|p| p.into_inner()).clone(),
        ))
    }
}

impl SunMemo {
    pub(crate) fn credit(&self, rules: &Rules, tick: Tick, full: u64) -> u64 {
        let unit = rules.tick_microseconds();
        let rule = rules.sunlight_credit;
        let mut memo = self.0.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((t, u, r, sun, value)) = &*memo
            && (*t, *u, *r, sun) == (tick, unit, rule, &rules.sun)
        {
            return value.credit(full);
        }
        let value = Intensity::at(rules, tick, None);
        *memo = Some((tick, unit, rule, rules.sun.clone(), value));
        value.credit(full)
    }
}

// Angles are scaled turns. Trig outputs and intensity use the same scale.
const SCALE: i64 = 1 << 30;
const QUARTER: i64 = SCALE / 4;
// Include the percentage denominator so a full-rise sun at 1% is exact.
const FULL_INTENSITY: i64 = 100 * SCALE;

fn mul(a: i64, b: i64) -> i64 {
    a * b / SCALE
}

fn phase(tick: Tick, offset: Tick, period: Tick) -> i64 {
    let elapsed = (tick % period + offset % period) % period;
    (u128::from(elapsed) * SCALE as u128 / u128::from(period)) as i64
}

fn angle(hundredths: i64) -> i64 {
    (hundredths % 36_000) * SCALE / 36_000
}

fn cos(turns: i64) -> i64 {
    let arc = turns.rem_euclid(SCALE);
    let arc = arc.min(SCALE - arc);
    let (arc, sign) = if arc > QUARTER {
        (SCALE / 2 - arc, -1)
    } else {
        (arc, 1)
    };
    if arc == QUARTER {
        return 0;
    }
    // Taylor polynomial for cos(π/2 × x) over 0 ≤ x ≤ 1. Coefficients
    // are rounded to Q30, in descending powers of x², then evaluated by Horner's rule.
    let x = arc * 4;
    let square = mul(x, x);
    let value = [
        -7,
        506,
        -27_060,
        987_048,
        -22_401_992,
        272_375_560,
        -1_324_675_879,
        SCALE,
    ]
    .into_iter()
    .fold(0, |sum, term| mul(sum, square) + term);
    sign * value.clamp(0, SCALE)
}

fn sin(turns: i64) -> i64 {
    cos(QUARTER - turns)
}

fn fixed_intensity(rules: &Rules, tick: Tick, site: Option<&Site>) -> u64 {
    let sun = rules.sun.clone().unwrap_or_default();
    let rise = if sun.tidal_lock {
        SCALE
    } else {
        let unit = rules.tick_microseconds().max(1);
        let day = (DAY_MICROSECONDS / unit).max(1);
        let year = year_ticks(unit);
        let offset = u64::from(sun.start_hour % 24) * day / 24;
        let hour = phase(tick, offset, day) - SCALE / 2;
        let season = phase(tick, offset, year);
        // Apply the tilt before reducing modulo a turn, since the season
        // scales the entire tilt and not just its remainder.
        let declination = i64::from(sun.tilt) * sin(season) / 36_000;
        let latitude = angle(
            site.and_then(|s| s.conditions.get(LATITUDE).copied())
                .unwrap_or(DEFAULT_LATITUDE),
        );
        let up = mul(sin(latitude), sin(declination))
            + mul(mul(cos(latitude), cos(declination)), cos(hour));
        // Smoothstep from the horizon to up = 0.2.
        let t = (up * 5).clamp(0, SCALE);
        mul(mul(t, t), 3 * SCALE - 2 * t)
    };
    (rise * i64::from(sun.intensity_percent)).min(FULL_INTENSITY) as u64
}

pub(super) fn rendered_intensity(rules: &Rules, tick: Tick, site: Option<&Site>) -> f32 {
    (fixed_intensity(rules, tick, site) as f64 / FULL_INTENSITY as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Founding;

    #[test]
    fn fixed_twilight_tracks_the_rendered_sun() {
        let mut world = Founding::default().generate().unwrap();
        let site = world.sites.values_mut().next().unwrap();
        let mut rules = world.rules;
        rules.sunlight_credit = CreditRule::Legacy;
        let day = DAY_MICROSECONDS / rules.tick_microseconds();
        let year = year_ticks(rules.tick_microseconds());
        for latitude in [-9_000, -6_700, -4_000, 0, 4_000, 6_700, 9_000] {
            site.conditions.insert(LATITUDE.into(), latitude);
            for season in [0, year / 4, year / 2, 3 * year / 4] {
                for tick in season..season + day {
                    let fixed = rendered_intensity(&rules, tick, Some(site));
                    let rendered = at(&rules, tick, Some(site))
                        .sun
                        .map_or(0.0, |s| s.intensity);
                    assert!(
                        (fixed - rendered).abs() < 0.000_01,
                        "latitude={latitude}, tick={tick}"
                    );
                }
            }
        }
    }

    #[test]
    fn credits_round_up_at_half_and_are_bounded_at_full_sun() {
        let mut rules = Founding::default().generate().unwrap().rules;
        rules.sun = Some(SunRules {
            tidal_lock: true,
            intensity_percent: 50,
            ..SunRules::default()
        });
        let memo = SunMemo::default();
        assert_eq!(memo.credit(&rules, u64::MAX, 3), 2);
        assert_eq!(memo.credit(&rules, u64::MAX, u64::MAX), 1 << 63);
        rules.sun.as_mut().unwrap().intensity_percent = 1;
        assert_eq!(memo.credit(&rules, u64::MAX, 50), 1);
        rules.sun.as_mut().unwrap().intensity_percent = u32::MAX;
        assert_eq!(memo.credit(&rules, u64::MAX, u64::MAX), u64::MAX);
        rules.sun = Some(SunRules {
            start_hour: 7,
            ..SunRules::default()
        });
        assert_eq!(
            memo.credit(&rules, u64::MAX, 500),
            sunlight_credit(&rules, u64::MAX, None, 500)
        );
    }
}
