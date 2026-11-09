// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1173's recovery criterion, read beside the original year bar from
//! the same run. A run passes when each of the five diets has a living
//! resident in the footprint at every monthly sample. A patch (one site)
//! losing a diet is allowed where a native return arrives within six months
//! or the diet is held elsewhere in the footprint meanwhile. A founding
//! lineage's loss is reported, never failed. On the 2x2 laid map the
//! footprint is the whole world, so losses are read at the patch and the
//! world only.

use cosmograph::{
    Session,
    directing::readings::scopes::{Scope, reading},
    schema::{Id, Key, Provenance},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// The diets of a web of diets, by founding lineage index modulo five
/// (`ecology::web::Role::of`).
pub const DIETS: [&str; 5] = ["producer", "grazer", "decomposer", "predator", "omnivore"];
/// The months a lost patch has for a native return.
pub const RETURN_MONTHS: u64 = 6;

/// Living residents by site and diet at one monthly sample.
pub type Residents = BTreeMap<Id, BTreeMap<&'static str, u64>>;

/// How a returning resident came: born of a scattered birth at `from`, or
/// walked in from `from`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Route {
    pub lineage: Key,
    pub means: &'static str,
    pub from: Option<Id>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    /// A native return within six months.
    Returned,
    /// No return in time, the diet held elsewhere in the footprint when
    /// the patch lost it. A later global loss overtaking it is reported on
    /// its own (1191).
    HeldElsewhere,
    /// The run ended inside the six months, the diet held elsewhere.
    Open,
    /// Lost with the whole footprint at the same sample: that global loss.
    Global,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PatchLoss {
    pub diet: &'static str,
    pub site: Id,
    pub lost_month: u64,
    pub returned_month: Option<u64>,
    /// The month a global loss overtook a patch held elsewhere (1191).
    pub overtaken_month: Option<u64>,
    pub routes: Vec<Route>,
    pub resolution: Resolution,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GlobalLoss {
    pub diet: &'static str,
    pub month: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LineageLoss {
    pub lineage: Key,
    pub month: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Recovery {
    pub criterion: &'static str,
    pub months: u64,
    pub patch_losses: Vec<PatchLoss>,
    pub global_losses: Vec<GlobalLoss>,
    /// Reported, never failed (1173).
    pub lineage_losses: Vec<LineageLoss>,
    pub outcome: Option<super::Outcome>,
    #[serde(skip)]
    residents: Vec<Residents>,
    #[serde(skip)]
    routes: BTreeMap<(u64, Id, &'static str), Vec<Route>>,
    #[serde(skip)]
    living: Vec<BTreeMap<Key, u64>>,
}

/// The founding lineage index `lineage` descends from.
fn founder(s: &Session, lineage: &str) -> Option<u32> {
    let lineages = &s.sim.state().lineages;
    let mut at = lineage;
    for _ in 0..lineages.len() + 1 {
        if let Some(i) = at.strip_prefix("lineage:").and_then(|i| i.parse().ok()) {
            return Some(i);
        }
        at = lineages.get(at)?.parent.as_deref()?;
    }
    None
}

pub fn diet(s: &Session, lineage: &str) -> Option<&'static str> {
    founder(s, lineage).map(|i| DIETS[i as usize % DIETS.len()])
}

impl Recovery {
    /// Reads the run at a monthly sample, then judges the months so far.
    pub fn observe(&mut self, s: &Session, expected: &BTreeSet<Key>, month: u64) {
        let sites: Vec<Id> = s.sim.state().sites.keys().copied().collect();
        let mut residents = Residents::new();
        for &site in &sites {
            let r = reading(&s.sim, &Scope::Patch(site)).expect("a patch reading");
            let by = residents.entry(site).or_default();
            for d in DIETS {
                by.insert(d, 0);
            }
            for (lineage, n) in &r.living {
                if let Some(d) = diet(s, lineage) {
                    *by.entry(d).or_default() += n;
                }
            }
        }
        let world = reading(&s.sim, &Scope::World).expect("a world reading");
        self.living.push(expected.iter().map(|l| (l.clone(), world.living.get(l).copied().unwrap_or(0))).collect());
        if let Some(before) = self.residents.last() {
            for (&site, by) in &residents {
                for (&d, &n) in by {
                    if n > 0 && before[&site][d] == 0 {
                        self.routes.insert((month, site, d), routes(s, site, d));
                    }
                }
            }
        }
        self.residents.push(residents);
        self.judge();
    }

    fn judge(&mut self) {
        let (patch, global) = judge(&self.residents);
        self.patch_losses = patch
            .into_iter()
            .map(|mut l| {
                if let Some(m) = l.returned_month {
                    l.routes = self.routes.get(&(m, l.site, l.diet)).cloned().unwrap_or_default();
                }
                l
            })
            .collect();
        self.global_losses = global;
        self.lineage_losses = lineage_losses(&self.living);
        self.criterion = "1173: each diet resident in the footprint at every monthly sample; a patch loss returns natively within six months or the diet is held elsewhere; lineage losses reported";
        self.months = self.residents.len().saturating_sub(1) as u64;
        self.outcome = Some(outcome(&self.patch_losses, &self.global_losses, self.months));
    }
}

/// The patch and global losses over monthly residents (month 0 first).
pub fn judge(months: &[Residents]) -> (Vec<PatchLoss>, Vec<GlobalLoss>) {
    let held = |m: usize, d: &str| months[m].values().any(|by| by.get(d).is_some_and(|&n| n > 0));
    let at = |m: usize, site: Id, d: &str| months[m].get(&site).and_then(|by| by.get(d)).is_some_and(|&n| n > 0);
    let mut global = vec![];
    for (m, _) in months.iter().enumerate() {
        for d in DIETS {
            if !held(m, d) {
                global.push(GlobalLoss { diet: d, month: m as u64 });
            }
        }
    }
    let mut patch = vec![];
    let sites: BTreeSet<Id> = months.iter().flat_map(|r| r.keys().copied()).collect();
    for site in sites {
        for d in DIETS {
            for m in 1..months.len() {
                if !(at(m - 1, site, d) && !at(m, site, d)) {
                    continue;
                }
                let back = (m + 1..months.len()).find(|&r| at(r, site, d));
                let six = RETURN_MONTHS as usize;
                let overtaken = (m..back.unwrap_or(months.len())).find(|&k| !held(k, d));
                let resolution = match back {
                    Some(r) if r - m <= six => Resolution::Returned,
                    _ if overtaken == Some(m) => Resolution::Global,
                    None if months.len() - 1 - m < six => Resolution::Open,
                    _ => Resolution::HeldElsewhere,
                };
                patch.push(PatchLoss {
                    diet: d,
                    site,
                    lost_month: m as u64,
                    returned_month: back.map(|r| r as u64),
                    overtaken_month: overtaken.filter(|&k| k > m).map(|k| k as u64),
                    routes: vec![],
                    resolution,
                });
            }
        }
    }
    (patch, global)
}

/// The first month each founding lineage had no living member, reported.
pub fn lineage_losses(living: &[BTreeMap<Key, u64>]) -> Vec<LineageLoss> {
    let names: BTreeSet<&Key> = living.iter().flat_map(|m| m.keys()).collect();
    names
        .into_iter()
        .filter_map(|l| {
            let m = living.iter().position(|by| by.get(l).copied().unwrap_or(0) == 0)?;
            Some(LineageLoss { lineage: l.clone(), month: m as u64 })
        })
        .collect()
}

/// Passed over a full two years with no global loss; failed on any global
/// loss however short the run. A patch loss fails only as part of one: it
/// was returned, held elsewhere or lost with the footprint (1191).
pub fn outcome(_patch: &[PatchLoss], global: &[GlobalLoss], months: u64) -> super::Outcome {
    if !global.is_empty() {
        super::Outcome::Failed
    } else if months >= 24 {
        super::Outcome::Passed
    } else {
        super::Outcome::Inconclusive
    }
}

/// How the residents of `diet` now at `site` came there.
fn routes(s: &Session, site: Id, diet_key: &str) -> Vec<Route> {
    let state = s.sim.state();
    let mut out = vec![];
    for g in state.population.groups.values() {
        let e = &g.entity;
        if !e.alive || e.resident_site() != Some(site) || diet(s, &e.lineage) != Some(diet_key) {
            continue;
        }
        let walked = e.visits.last().filter(|v| v.place != site).map(|v| v.place);
        let route = match (&e.provenance, walked) {
            (_, Some(from)) => Route { lineage: e.lineage.clone(), means: "walk", from: Some(from) },
            (Provenance::Born(event), None) => {
                let from = state.events.get(event).and_then(|ev| match ev.place {
                    cosmograph::schema::EventPlace::Site(p) => Some(p),
                    _ => None,
                });
                Route { lineage: e.lineage.clone(), means: "scattered birth", from }
            },
            _ => Route { lineage: e.lineage.clone(), means: "unknown", from: None },
        };
        if !out.contains(&route) {
            out.push(route);
        }
    }
    out
}
