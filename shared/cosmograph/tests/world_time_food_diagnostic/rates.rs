// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Protocol guards for inherited numeric life histories. The aligned food
//! declarations may differ; the clocks and founded adult stocks may not.

use super::*;
use cosmograph::rules::{Binding, Query};

#[test]
fn the_aligned_fixture_keeps_the_preceding_world_time_rates() {
    let s = native_world();
    let g = s.sim.genesis();
    // The fixture repeats the five existing founding roles twice. Values are
    // the measured preceding contract, not values derived from current code.
    let adult = [60, 12, 8, 15, 12];
    let upkeep = [1440, 1680, 720, 2160, 2400];
    let feed = [0, 720, 240, 1440, 720];
    let breed_days = [90, 90, 45, 180, 180];
    let maturity_days = [30, 120, 21, 365, 180];
    let yearly_hazard = [100_000, 200_000, 500_000, 120_000, 150_000];
    let doubling_years = [3, 2, 1, 3, 3];
    for i in 0..10usize {
        let role = i % 5;
        let lineage = format!("lineage:{i}");
        let tissue = format!("matter:{i}-0");
        let process = |kind| &g.rules.processes[&format!("ecology:{kind}-{i}")];
        assert_eq!(process("upkeep").period, Some(upkeep[role]));
        let birth = process("birth");
        assert_eq!(birth.period, Some(breed_days[role] * 1440));
        assert!(birth.requires.iter().any(|q| matches!(q,
            Query::Age { at_least } if *at_least == maturity_days[role] * 1440
        )));
        let hazard = process("hazard");
        assert_eq!(hazard.period, Some(1));
        let aging = hazard.risk.as_ref().unwrap().aging.as_ref().unwrap();
        assert_eq!(aging.maturity, maturity_days[role] * 1440);
        assert_eq!(aging.year, 525_960);
        assert_eq!(aging.per_million_per_year, yearly_hazard[role]);
        assert_eq!(aging.doubling_years, doubling_years[role]);
        if feed[role] > 0 {
            assert_eq!(process("feed").period, Some(feed[role] + i as u64));
        } else {
            assert_eq!(process("produce").period, Some(720));
            assert_eq!(process("regrow").period, Some(240));
        }
        assert!(g.population.groups.values().any(|group| {
            let e = &group.entity;
            e.lineage == lineage && cosmograph::anatomy::held(e, &g.rules, &tissue) == adult[role]
        }));
        assert!(birth.requires.iter().any(|q| matches!(q,
            Query::Account { who: Binding::Actor, key, .. } if *key == tissue
        )));
    }
}
