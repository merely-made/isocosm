// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A world-time producer's dormant young (cases 1192 to 1197): the shared
//! processes of `ecology::dormant`, with waking gated on the site's light
//! share suiting the seed's mass.

use super::LIGHT_PER_GROWTH;
use crate::{Result, rules::*, schema::Tick, simulation::Genesis};

/// The rule as the founding states it, with the dormant trait declared, or
/// nothing where the founding states none. Waking reads a plant's light
/// share, which only sensing bodies have.
pub(super) fn declare(founding: &crate::Founding, g: &mut Genesis) -> Result<Option<Dormancy>> {
    let Some(rule) = founding.dormancy.clone() else { return Ok(None) };
    if !founding.bodies.as_ref().is_some_and(|b| b.sensing) {
        return Err("dormancy wakes on the plant light-share sense, which needs sensing bodies".into());
    }
    g.rules.traits.insert(rule.key.clone());
    Ok(Some(rule))
}

/// Lineage `i`'s dormant processes: `share` is its seed share and `season`
/// the season in ticks.
pub(super) fn seeds(g: &mut Genesis, i: u32, rule: &Dormancy, share: u64, season: Tick) {
    // It wakes once its site's light share covers what a seedling needs,
    // less as the seed is heavier.
    let light = Query::Sensed { reading: "sense:light-share".into(), at_least: Dormancy::waking(LIGHT_PER_GROWTH, share) };
    crate::ecology::dormant::seeds(g, i, rule, season, vec![light]);
}

/// The birth processes' ledger entry: seed share by process.
pub(super) fn record(rule: &mut Dormancy, i: u32, share: u64) {
    crate::ecology::dormant::record(rule, i, share);
}
