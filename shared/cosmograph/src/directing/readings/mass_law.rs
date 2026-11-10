// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the body-size rule did to each lineage (case 1148): its adult mass
//! and the factors its rates were scaled by, for a sheet or a debug print.
//! A pure reading of the founded world; empty where the founding sets no
//! rule.

use crate::{ecology::mass_law, schema::*, simulation::Simulation};
use std::collections::BTreeMap;

/// One lineage's adult mass and factors, in hundredths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    /// Adult mass, in milligrams.
    pub mass: u64,
    /// Upkeep and meal factor (the three-quarter power).
    pub upkeep: u32,
    /// Maturity, lifespan and birth spacing factor (the one-quarter power).
    pub time: u32,
}

/// Each bodied lineage's row, by lineage key.
pub fn mass_law(sim: &Simulation) -> BTreeMap<Key, Row> {
    let g = sim.genesis();
    let Some(law) = g.founding.as_ref().and_then(|f| f.mass_law.as_ref()) else {
        return BTreeMap::new();
    };
    let Ok(masses) = mass_law::masses(&g.rules, &g.lineages) else {
        return BTreeMap::new();
    };
    let reference = law.reference.or_else(|| mass_law::median(&masses)).unwrap_or(1);
    masses
        .into_iter()
        .map(|(i, mass)| {
            let f = mass_law::factors(mass, reference, law.bounds);
            (format!("lineage:{i}"), Row { mass, upkeep: f.upkeep, time: f.time })
        })
        .collect()
}
