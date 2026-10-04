// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The receipt's shape, and the summaries drawn from the raw arms.

use isocosm::{
    probe::{
        BodyFounding, ProbeFounding, ProbeWorld,
        check::{Comparison, Settings},
        readings::{Probe, Reading},
    },
    rules::{Causation, Effect, Process, Query},
    schema::Key,
};
use serde::Serialize;
use std::collections::BTreeSet;

pub const ARMS: [&str; 6] = [
    "exact",
    "exact-control",
    "crowd",
    "crowd-averaged",
    "crowd-approximate",
    "crowd-unweighted",
];

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
    /// Crowds only: hunting passes where the prey ran out part way through
    /// hunters sharing one state, so that which of them ate changed nothing.
    #[serde(skip_serializing_if = "is_zero")]
    pub shortfalls: u64,
    /// Why a crowd refused the draw, which then has no readings on this arm
    /// and is left out of the comparisons the arm takes part in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
    pub readings: Vec<u64>,
}

impl Arm {
    pub fn refused(arm: &'static str, dynamics: u64, why: String) -> Self {
        Self {
            arm,
            dynamics,
            micros: 0,
            evaluations: 0,
            represented: 0,
            accepted: 0,
            blocked: 0,
            stored: 0,
            alive: 0,
            alive_states: 0,
            shortfalls: 0,
            refused: Some(why),
            readings: Vec::new(),
        }
    }
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

/// One contested thing as a drawn world holds it.
#[derive(Serialize)]
pub struct Contested {
    pub key: Key,
    pub ration: u64,
    /// The first lineage's threshold of wanting it.
    pub want: u64,
    pub regrowth: u64,
}

#[derive(Serialize)]
pub struct Draw {
    pub k: u64,
    pub seed: u64,
    pub sites: usize,
    pub members: u64,
    pub leanings: Vec<Key>,
    pub contested: Vec<Contested>,
    pub margin: u64,
    pub cost: u64,
    pub upset: u32,
    pub advantage: u64,
    /// With predators: the hunters as drawn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predation: Option<Predation>,
    /// A world of bodies: each lineage's founders, by lineage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodies: Option<std::collections::BTreeMap<Key, u64>>,
    pub arms: Vec<Arm>,
}

/// A world's hunters: how many in all, what each eats at a time, and the
/// body below which it hunts.
#[derive(Serialize)]
pub struct Predation {
    pub hunters: u64,
    pub bite: u64,
    pub appetite: u64,
}

fn predation(world: &ProbeWorld) -> Option<Predation> {
    let g = &world.genesis;
    let p = g
        .rules
        .processes
        .values()
        .find(|p| p.target.as_ref().is_some_and(|t| t.weighted))?;
    let bite = p.effects.iter().find_map(|e| match e {
        Effect::Eat { amount, .. } => amount.resolved().ok(),
        _ => None,
    });
    let appetite = p.requires.iter().find_map(|q| match q {
        Query::Below { amount, .. } => Some(*amount),
        _ => None,
    });
    let own = p.requires.iter().find_map(|q| match q {
        Query::Trait { key, .. } => Some(key),
        _ => None,
    })?;
    let groups = g.population.groups.values();
    let hunters = groups
        .filter(|c| c.entity.traits.contains(own))
        .map(|c| c.count);
    Some(Predation {
        hunters: hunters.sum(),
        bite: bite?,
        appetite: appetite?,
    })
}

impl Draw {
    pub fn new(k: u64, seed: u64, world: &ProbeWorld) -> Result<Self, String> {
        let g = &world.genesis;
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
        // What regrows a thing is the agentless process that makes it.
        let regrowth = |key: &Key| {
            let makes = |p: &&Process| {
                p.causation == Causation::Agentless
                    && p.effects.iter().any(
                        |e| matches!(e, Effect::Transform { give, .. } if give.contains_key(key)),
                    )
            };
            let p = g.rules.processes.values().find(makes);
            p.and_then(|p| p.requires.last()).map_or(0, threshold)
        };
        let contested = world
            .competitions()
            .iter()
            .map(|(key, c)| Contested {
                key: key.clone(),
                ration: c.ration,
                want: c.kinds.first().map_or(0, |k| threshold(&k.hungry)),
                regrowth: regrowth(key),
            })
            .collect();
        // A world of bodies contests nothing; it is summed by lineage.
        let first = world.competitions().values().next();
        let bodies = first.is_none().then(|| {
            let mut by = std::collections::BTreeMap::new();
            for c in g.population.groups.values() {
                if c.entity.kingdom != "kingdom:world" {
                    *by.entry(c.entity.lineage.clone()).or_default() += c.count;
                }
            }
            by
        });
        Ok(Self {
            k,
            seed,
            sites: g.sites.len(),
            members: g.population.count() - g.sites.len() as u64,
            leanings: world.kinds().iter().map(|k| leaning(&k.identity)).collect(),
            contested,
            margin: first.map_or(0, |c| c.margin),
            cost: first.map_or(0, |c| c.cost),
            upset: first.map_or(0, |c| c.upset),
            advantage: first.map_or(0, |c| c.advantage),
            predation: predation(world),
            bodies,
            arms: vec![],
        })
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

/// An arm a run left out reads as nothing done.
static MISSING: Arm = Arm {
    arm: "missing",
    dynamics: 0,
    micros: 0,
    evaluations: 0,
    represented: 0,
    accepted: 0,
    blocked: 0,
    stored: 0,
    alive: 0,
    alive_states: 0,
    shortfalls: 0,
    refused: None,
    readings: Vec::new(),
};

fn find<'a>(d: &'a Draw, name: &str) -> &'a Arm {
    d.arms.iter().find(|a| a.arm == name).unwrap_or(&MISSING)
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
    /// With the approximate pairing draw: its crowd's evaluations and time,
    /// and how much faster it ran than the exact runner and the exact crowd.
    pub approximate: Option<Approximate>,
}

#[derive(Serialize)]
pub struct Approximate {
    pub evaluations: u64,
    pub micros: u64,
    pub wall_ratio_to_exact: f64,
    pub wall_ratio_to_crowd: f64,
    pub per_draw_wall_ratio_to_crowd: Spread,
}

impl Savings {
    pub fn new(draws: &[&Draw]) -> Self {
        let total =
            |name: &str, f: fn(&Arm) -> u64| draws.iter().map(|d| f(find(d, name))).sum::<u64>();
        let ran = |name: &str| draws.iter().all(|d| d.arms.iter().any(|a| a.arm == name));
        let approximate = ran("crowd-approximate").then(|| {
            let micros = total("crowd-approximate", |a| a.micros);
            Approximate {
                evaluations: total("crowd-approximate", |a| a.evaluations),
                micros,
                wall_ratio_to_exact: total("exact", |a| a.micros) as f64 / micros.max(1) as f64,
                wall_ratio_to_crowd: total("crowd", |a| a.micros) as f64 / micros.max(1) as f64,
                per_draw_wall_ratio_to_crowd: Spread::of(
                    draws
                        .iter()
                        .map(|d| {
                            find(d, "crowd").micros as f64
                                / find(d, "crowd-approximate").micros.max(1) as f64
                        })
                        .collect(),
                ),
            }
        });
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
            approximate,
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
    pub fn new(draws: &[&Draw]) -> Self {
        let spread = |f: &dyn Fn(&Draw) -> f64| Spread::of(draws.iter().map(|d| f(d)).collect());
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
    /// With predators: whether the check told the draw's control, prey
    /// drawn by members alone, from the exact runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draw_control_detected: Option<bool>,
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
            draw_control_detected: comparisons
                .iter()
                .find(|c| c.name.ends_with("(draw control)"))
                .map(|c| c.different),
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
        ) + &self
            .draw_control_detected
            .map_or(String::new(), |d| format!(", draw control detected {d}"))
    }
}

#[derive(Serialize)]
pub struct Checks {
    pub exact_rerun_identical: bool,
    pub collect_changes_no_outcome: bool,
}

pub const NOTE: &str = "Worlds are drawn per k; each arm runs the same world under its own dynamics seed. The exact arms are the core's individual runner: every competition resolves member by member against the tick's start, fights on copies of the two states, and each member's takes settle through the interpreter at the tick's end. The crowd is the exact-state histogram with exact count draws; the approximate crowd, when run, takes each count in one step near its mean and variance; the averaged crowd replaces each lineage's reserves at a site by their average after every round. Readings are taken after the last tick. Evaluations count interpreter applications, and the act applications fights make on copies: per member in the exact arms, per state, and per distinct act on a state within a round, in the crowds. With predators, the exact arms draw each hunter's prey by the core's weighted draw from the pass's start (ruling 454); the crowds draw a prey state per hunting member, weighted by its members times what each holds, and the member it lands on uniformly, and the unweighted crowd, the draw's control, by its members alone. A world of bodies (checkpoint 6) contests nothing: its producers fix and grow glands and its grazers graze them by the same weighted draw, and the averaged crowd replaces each lineage's reserves at a site by their average after every tick.";

/// The domain a run drew its worlds from.
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Domain {
    Probe(Box<ProbeFounding>),
    Bodies(Box<BodyFounding>),
}

impl Domain {
    pub fn generate(&self, seed: u64) -> Result<ProbeWorld, String> {
        match self {
            Self::Probe(d) => ProbeFounding { seed, ..*d.clone() }.generate(),
            Self::Bodies(d) => BodyFounding {
                seed,
                ..(**d).clone()
            }
            .generate(),
        }
    }
}

#[derive(Serialize)]
pub struct Receipt {
    pub version: u32,
    pub kind: &'static str,
    pub master_seed: u64,
    pub arms: Vec<&'static str>,
    pub domain: Domain,
    pub settings: Settings,
    pub read_set: BTreeSet<String>,
    pub readings: Vec<ReadingInfo>,
    pub verdicts: Option<Verdicts>,
    pub comparisons: Vec<Comparison>,
    pub savings: Savings,
    pub density: Density,
    pub checks: Checks,
    /// The draws each crowd refused, by arm, when there were any.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub refused: std::collections::BTreeMap<&'static str, Vec<u64>>,
    pub note: &'static str,
    pub draws: Vec<Draw>,
}
