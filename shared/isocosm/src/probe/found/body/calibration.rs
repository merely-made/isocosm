// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 505's calibration: income and the mouthful read their measurements
//! alone, at rates per unit set so the domain's median body earns what
//! checkpoint 6's tissue rates gave it. Over a thousand founded worlds, each
//! body's old rate is worked out as checkpoint 6 wrote it, and the new one
//! from the constants, and the medians must agree to a milligram. Each
//! rate is the middle of the range of rates that give the median, as the
//! nearest fraction with a denominator of at most 400; calibrated again
//! for checkpoint 8's recipe bodies, which give reproduce a cell and vary
//! (rulings 513 and 531), the medians 5 and 12 mg.

use super::{
    BodyFounding,
    physiology::{FIXES_PER_FACE, GRAZES_PER_VOXEL},
};
use crate::{anatomy, rules::Measure, schema::*};

/// Mass to the three-quarter power as checkpoint 6's expressions took it.
fn m34(m: u64) -> u64 {
    let m = m.max(1);
    (m * m.isqrt()).isqrt()
}

/// What each founded member earned under checkpoint 6, and what it earns
/// now, by lineage: 0 the producers' income, 1 the grazers' mouthful.
fn rates() -> [Vec<(u64, u64, u64)>; 2] {
    let mut out: [Vec<(u64, u64, u64)>; 2] = [vec![], vec![]];
    for seed in 1..=1000 {
        let world = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .expect("the domain founds");
        let rules = &world.genesis.rules;
        let b = rules.body();
        for cohort in world.genesis.population.groups.values() {
            let e = &cohort.entity;
            let i = match e.lineage.as_str() {
                "lineage:0" => 0,
                "lineage:1" => 1,
                _ => continue,
            };
            let living: Vec<&Part> = e.parts.values().filter(|p| !p.severed).collect();
            let tissue = anatomy::held(e, rules, &format!("tissue:{i}"));
            let sum = |f: &dyn Fn(&Part) -> u128| living.iter().map(|p| f(p)).sum::<u128>();
            let ceiling = sum(&|p| u128::from(anatomy::ceiling(p, b))).max(1);
            let (old, new) = match i {
                0 => {
                    let old = (5 * m34(tissue) / 31).max(1);
                    let area = sum(&|p| anatomy::share_of(p, "function:fix", Measure::Area));
                    let (n, d) = FIXES_PER_FACE;
                    let new = (area * n as u128 / d as u128).max(1);
                    (old, new as u64)
                },
                _ => {
                    let span = sum(&|p| match p.functions.contains("function:contract") {
                        true => u128::from(
                            p.half_extent
                                .iter()
                                .map(|h| h.unsigned_abs())
                                .max()
                                .unwrap_or(0),
                        ),
                        false => 0,
                    });
                    let priced = ceiling + span * 100;
                    let old = (3 * u128::from(m34(tissue)) * priced / (31 * ceiling)).max(1);
                    let volume = sum(&|p| anatomy::share_of(p, "function:intake", Measure::Volume));
                    let (n, d) = GRAZES_PER_VOXEL;
                    let new = (n as u128 * volume * priced / (d as u128 * ceiling)).max(1);
                    (old as u64, new as u64)
                },
            };
            out[i].push((old, new, cohort.count));
        }
    }
    out
}

/// The member-weighted median of one column.
fn median(rows: &[(u64, u64, u64)], column: fn(&(u64, u64, u64)) -> u64) -> u64 {
    let mut values: Vec<(u64, u64)> = rows.iter().map(|r| (column(r), r.2)).collect();
    values.sort_unstable();
    let total: u64 = values.iter().map(|v| v.1).sum();
    let mut seen = 0;
    for (value, n) in values {
        seen += n;
        if seen * 2 >= total {
            return value;
        }
    }
    0
}

#[test]
fn the_median_body_earns_what_it_did() {
    let [producers, grazers] = rates();
    let medians: Vec<(&str, u64, u64)> = [("income", &producers), ("mouthful", &grazers)]
        .into_iter()
        .map(|(name, rows)| (name, median(rows, |r| r.0), median(rows, |r| r.1)))
        .collect();
    for (name, old, new) in &medians {
        eprintln!("{name}: median old {old}, new {new}");
    }
    // Exactly: the medians are a few milligrams, so a tolerance of one
    // would pass a rate a quarter off.
    for (name, old, new) in medians {
        assert_eq!(old, new, "{name}");
    }
}
