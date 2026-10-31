// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sun (971 to 973): periodic, locked when told, dark at night, and a
//! pure function of its inputs.

use cosmograph::light::{self, DAY_AMBIENT, NIGHT_AMBIENT, SunRules};
use cosmograph::{Founding, preset::Preset};
use cosmograph::rules::{DEFAULT_TICK_MICROSECONDS, Rules, year_ticks};

const DAY: u64 = 1_440;

fn founding(preset: Option<Preset>) -> Founding {
    Founding { seed: 9, sites: 3, population: 24, cohort_size: 8, lineages: 2, preset, ..Default::default() }
}

fn rules(sun: Option<SunRules>) -> Rules {
    let mut r = founding(None).generate().unwrap().rules;
    r.sun = sun;
    r
}

fn up(rules: &Rules, tick: u64) -> f32 {
    light::at(rules, tick, None).sun.map_or(f32::NEG_INFINITY, |s| s.direction[1])
}

#[test]
fn a_day_repeats_and_a_year_repeats() {
    let r = rules(None);
    // 365.25 days: the pattern repeats exactly every four years, 1,461 days.
    let year = 4 * year_ticks(DEFAULT_TICK_MICROSECONDS);
    for tick in [0, 100, 500, 720, 1000, 1439] {
        assert_eq!(light::at(&r, tick, None), light::at(&r, tick + year, None), "year, tick {tick}");
    }
    // Over a day the seasons barely move, so same hour a day later is close.
    let (a, b) = (light::at(&r, 720, None), light::at(&r, 720 + DAY, None));
    let (a, b) = (a.sun.unwrap(), b.sun.unwrap());
    assert!((a.direction[1] - b.direction[1]).abs() < 0.01);
}

#[test]
fn noon_is_the_maximum_and_night_has_no_sun() {
    let r = rules(None);
    let best = (0..DAY).max_by(|a, b| up(&r, *a).total_cmp(&up(&r, *b))).unwrap();
    assert!((best as i64 - 720).abs() <= 1, "peak at {best}");
    let midnight = light::at(&r, 0, None);
    assert!(midnight.sun.is_none());
    assert_eq!(midnight.ambient, NIGHT_AMBIENT);
    let noon = light::at(&r, 720, None);
    assert!(noon.sun.unwrap().intensity > 0.99);
    assert!((noon.ambient - DAY_AMBIENT).abs() < 1e-6);
    // Dawn comes east of noon's south, dusk west.
    assert!(light::at(&r, 6 * 60, None).sun.is_some_and(|s| s.direction[0] > 0.0));
    assert!(light::at(&r, 18 * 60, None).sun.is_some_and(|s| s.direction[0] < 0.0));
}

#[test]
fn a_tidal_lock_never_moves_and_matches_the_fixed_sun() {
    let r = rules(Some(SunRules { tidal_lock: true, ..SunRules::default() }));
    let first = light::at(&r, 0, None);
    for tick in [1, 500, 720, 100_000, 400_000] {
        assert_eq!(light::at(&r, tick, None), first);
    }
    let sun = first.sun.unwrap();
    let len = (0.4f32 * 0.4 + 0.8 * 0.8 + 0.3 * 0.3).sqrt();
    assert!((sun.direction[1] - 0.8 / len).abs() < 1e-6);
    assert_eq!(sun.intensity, 1.0);
    assert!((first.ambient - DAY_AMBIENT).abs() < 1e-6);
}

#[test]
fn the_season_moves_noon_and_a_dim_sun_is_dim() {
    let r = rules(None);
    let year = year_ticks(DEFAULT_TICK_MICROSECONDS);
    let summer = light::at(&r, year / 4 + 720 - (year / 4) % DAY, None).sun.unwrap();
    let winter = light::at(&r, 3 * year / 4 + 720 - (3 * year / 4) % DAY, None).sun.unwrap();
    assert!(summer.direction[1] > winter.direction[1] + 0.2);
    let dim = rules(Some(SunRules { intensity_percent: 35, ..SunRules::default() }));
    assert!((light::at(&dim, 720, None).sun.unwrap().intensity - 0.35).abs() < 1e-6);
}

#[test]
fn two_calls_agree() {
    let r = rules(None);
    for tick in (0..3 * DAY).step_by(7) {
        assert_eq!(light::at(&r, tick, None), light::at(&r.clone(), tick, None));
    }
}

#[test]
fn the_default_sun_is_not_stored_and_presets_state_theirs() {
    let plain = founding(None).generate().unwrap().rules;
    assert!(plain.sun.is_none());
    assert!(!serde_json::to_string(&plain).unwrap().contains("\"sun\""));
    let shelf = founding(Some(Preset::TidalShelf)).generate().unwrap().rules;
    assert!(shelf.sun.as_ref().is_some_and(|s| s.tidal_lock));
    let deep = founding(Some(Preset::HeavyDeep)).generate().unwrap().rules;
    assert!(deep.sun.as_ref().is_some_and(|s| s.intensity_percent < 100));
    let year = founding(Some(Preset::LongYear)).generate().unwrap().rules;
    assert!(year.sun.as_ref().is_some_and(|s| s.tilt > SunRules::default().tilt));
}
