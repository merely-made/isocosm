// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The receipt's shape, and the summaries drawn from the raw arms.

use isocosm::{
    probe::{
        ProbeFounding, ProbeWorld,
        check::{Comparison, Settings},
        readings::{Probe, Reading},
    },
    rules::Query,
    schema::Key,
};
use serde::Serialize;
use std::collections::BTreeSet;

pub const ARMS: [&str; 4] = ["exact", "exact-control", "crowd", "crowd-averaged"];

#[derive(Serialize)]
pub struct Arm {
    pub arm: &'static str,
    pub dynamics: u64,
    pub micros: u64,
    pub evaluations: u64,
    pub represented: u64,
    pub accepted: u64,
    pub blocked: u64,
    /// Exact: groups stored after the last collection. Crowd: bins.
    pub stored: usize,
    pub alive: u64,
    pub alive_states: usize,
    pub readings: Vec<u64>,
}

#[derive(Serialize)]
pub struct Draw {
    pub k: u64,
    pub seed: u64,
    pub sites: usize,
    pub members: u64,
    pub leanings: Vec<Key>,
    pub ration: u64,
    pub hunger: u64,
    pub margin: u64,
    pub cost: u64,
    pub regrowth: u64,
    pub arms: Vec<Arm>,
}

impl Draw {
    pub fn new(k: u64, seed: u64, world: &ProbeWorld) -> Self {
        let g = &world.genesis;
        let c = &world.competition;
        let leaning = |identity: &str| {
            g.lineages
                .values()
                .find(|l| l.traits.contains(identity))
                .and_then(|l| l.traits.iter().find(|t| t.starts_with("leaning:")))
                .cloned()
                .unwrap_or_default()
        };
        let threshold = |q: &Query| match q {
            Query::Below { amount, .. } => *amount,
            Query::Account { at_least, .. } => *at_least,
            _ => 0,
        };
        Self {
            k,
            seed,
            sites: g.sites.len(),
            members: g.population.count() - g.sites.len() as u64,
            leanings: c.kinds.iter().map(|k| leaning(&k.identity)).collect(),
            ration: c.ration,
            hunger: c.kinds.first().map_or(0, |k| threshold(&k.hungry)),
            margin: c.margin,
            cost: c.cost,
            regrowth: g.rules.processes["probe:regrow"]
                .requires
                .last()
                .map_or(0, threshold),
            arms: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct ReadingInfo {
    pub key: Key,
    pub source: Key,
    pub probe: Probe,
    pub starvation: bool,
    pub bound_per_mille: u32,
}

impl ReadingInfo {
    pub fn new(r: &Reading, bound_per_mille: u32) -> Self {
        Self {
            key: r.key.clone(),
            source: r.source.clone(),
            probe: r.probe.clone(),
            starvation: r.starvation,
            bound_per_mille,
        }
    }
}

#[derive(Serialize)]
pub struct Spread {
    pub min: f64,
    pub median: f64,
    pub max: f64,
}

impl Spread {
    pub fn of(mut values: Vec<f64>) -> Self {
        values.sort_by(f64::total_cmp);
        let at = |i: usize| values.get(i).copied().unwrap_or(0.0);
        Self {
            min: at(0),
            median: at(values.len() / 2),
            max: at(values.len().saturating_sub(1)),
        }
    }
}

fn find<'a>(d: &'a Draw, name: &str) -> &'a Arm {
    d.arms.iter().find(|a| a.arm == name).expect("arm ran")
}

#[derive(Serialize)]
pub struct Savings {
    pub exact_evaluations: u64,
    pub crowd_evaluations: u64,
    pub evaluation_ratio: f64,
    pub per_draw_evaluation_ratio: Spread,
    pub exact_represented: u64,
    pub crowd_represented: u64,
    pub exact_micros: u64,
    pub crowd_micros: u64,
    pub wall_ratio: f64,
}

impl Savings {
    pub fn new(draws: &[Draw]) -> Self {
        let total =
            |name: &str, f: fn(&Arm) -> u64| draws.iter().map(|d| f(find(d, name))).sum::<u64>();
        let (exact, crowd) = (
            total("exact", |a| a.evaluations),
            total("crowd", |a| a.evaluations),
        );
        let (exact_micros, crowd_micros) =
            (total("exact", |a| a.micros), total("crowd", |a| a.micros));
        Self {
            exact_evaluations: exact,
            crowd_evaluations: crowd,
            evaluation_ratio: exact as f64 / crowd.max(1) as f64,
            per_draw_evaluation_ratio: Spread::of(
                draws
                    .iter()
                    .map(|d| {
                        find(d, "exact").evaluations as f64
                            / find(d, "crowd").evaluations.max(1) as f64
                    })
                    .collect(),
            ),
            exact_represented: total("exact", |a| a.represented),
            crowd_represented: total("crowd", |a| a.represented),
            exact_micros,
            crowd_micros,
            wall_ratio: exact_micros as f64 / crowd_micros.max(1) as f64,
        }
    }
}

#[derive(Serialize)]
pub struct Density {
    pub founders: Spread,
    pub alive_end: Spread,
    pub alive_per_state_end: Spread,
    pub exact_groups_stored_end: Spread,
    pub crowd_bins_end: Spread,
}

impl Density {
    pub fn new(draws: &[Draw]) -> Self {
        let spread = |f: &dyn Fn(&Draw) -> f64| Spread::of(draws.iter().map(f).collect());
        Self {
            founders: spread(&|d| d.members as f64),
            alive_end: spread(&|d| find(d, "crowd").alive as f64),
            alive_per_state_end: spread(&|d| {
                let c = find(d, "crowd");
                c.alive as f64 / c.alive_states.max(1) as f64
            }),
            exact_groups_stored_end: spread(&|d| find(d, "exact").stored as f64),
            crowd_bins_end: spread(&|d| find(d, "crowd").stored as f64),
        }
    }
}

#[derive(Serialize)]
pub struct Verdicts {
    pub main_pass: bool,
    pub main_equivalent: bool,
    pub main_different: bool,
    pub positive_control_pass: bool,
    /// Failing means a starvation reading was detected different, or could
    /// not be certified within its bound. Detection is the stronger sign.
    pub negative_control_failed_starvation_check: bool,
    pub negative_control_detected_starvation_difference: bool,
    pub starvation_readings: Vec<Key>,
}

impl Verdicts {
    pub fn new(comparisons: &[Comparison], starvation: Vec<Key>) -> Self {
        let starving = |c: &Comparison| {
            c.readings
                .iter()
                .filter(|r| starvation.contains(&r.key))
                .map(|r| (r.detected, r.certified))
                .collect::<Vec<_>>()
        };
        let negative = starving(&comparisons[2]);
        Self {
            main_pass: comparisons[0].pass,
            main_equivalent: comparisons[0].equivalent,
            main_different: comparisons[0].different,
            positive_control_pass: comparisons[1].pass,
            negative_control_failed_starvation_check: negative.iter().any(|(d, c)| *d || !*c),
            negative_control_detected_starvation_difference: negative.iter().any(|(d, _)| *d),
            starvation_readings: starvation,
        }
    }
    pub fn summary(&self) -> String {
        format!(
            "main pass {} (equivalent {}, different {}), positive control pass {}, negative control starvation detected {}",
            self.main_pass,
            self.main_equivalent,
            self.main_different,
            self.positive_control_pass,
            self.negative_control_detected_starvation_difference
        )
    }
}

#[derive(Serialize)]
pub struct Checks {
    pub exact_rerun_identical: bool,
    pub collect_changes_no_outcome: bool,
}

pub const NOTE: &str = "Worlds are drawn per k; each arm runs the same world under its own dynamics seed. The exact arms are the core's individual runner with the probe's competition round executed through the same interpreter; the crowd is the exact-state histogram with count draws; the averaged crowd replaces each lineage's reserves at a site by their average after every round. Readings are taken after the last tick. Evaluations count interpreter applications: one per member in the exact arms, one per state in the crowd.";

#[derive(Serialize)]
pub struct Receipt {
    pub version: u32,
    pub kind: &'static str,
    pub master_seed: u64,
    pub arms: Vec<&'static str>,
    pub domain: ProbeFounding,
    pub settings: Settings,
    pub read_set: BTreeSet<String>,
    pub readings: Vec<ReadingInfo>,
    pub verdicts: Option<Verdicts>,
    pub comparisons: Vec<Comparison>,
    pub savings: Savings,
    pub density: Density,
    pub checks: Checks,
    pub note: &'static str,
    pub draws: Vec<Draw>,
}
