// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a held surface lineage lies, for the minimap and the scene (cases
//! 1223, 1224, 1226), and the seeds a world banks (case 1193). Pure readings
//! of a world: a surface's matter lies in a site's ledger or in the ledger of
//! each body that hosts it, so where it is is counted, never kept.

use cosmograph::{directing::readings, schema::{Entity, Id, Key}, simulation::Simulation};
use std::collections::BTreeMap;

/// How much of the held surfaces lies at one site.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SiteHold {
    /// Matter on the site itself, over every held surface.
    pub land: u64,
    /// Bodies at the site that carry some of it.
    pub hosts: u64,
}

/// The surface lineages `participant` holds that still hold matter, in key order.
pub fn held_surfaces(sim: &Simulation, participant: Id) -> Vec<Key> {
    sim.holds(participant).into_iter()
        .filter(|key| sim.is_surface(key) && readings::held(sim, key).is_some_and(|held| !held.ended))
        .collect()
}

/// Whether `entity` carries matter of any of `surfaces` in its accounts.
pub fn carries_held(entity: &Entity, surfaces: &[Key]) -> bool {
    surfaces.iter().any(|key| entity.accounts.get(key).is_some_and(|amount| *amount > 0))
}

/// Each site that holds some of `surfaces`, on its own ground or in bodies
/// living there; sites with none are left out.
pub fn site_holds(sim: &Simulation, surfaces: &[Key]) -> BTreeMap<Id, SiteHold> {
    let state = sim.state();
    let mut out: BTreeMap<Id, SiteHold> = BTreeMap::new();
    for (id, site) in &state.sites {
        let land: u64 = surfaces.iter().map(|key| site.accounts.get(key).copied().unwrap_or(0)).sum();
        if land > 0 { out.entry(*id).or_default().land = land; }
    }
    for group in state.population.groups.values().filter(|g| g.entity.alive && carries_held(&g.entity, surfaces)) {
        if let Some(site) = group.entity.resident_site() {
            out.entry(site).or_default().hosts += group.count;
        }
    }
    out
}

/// How strongly a mark draws for `amount` against the most any site holds: a
/// faint mark for the least, full for the most, none for nothing.
pub fn mark_strength(amount: u64, peak: u64) -> f32 {
    if amount == 0 || peak == 0 { return 0.0; }
    0.3 + 0.7 * (amount.min(peak) as f32 / peak as f32)
}

/// The minimap's legend for what it marks, in words; none when nothing is held.
pub fn hold_legend_words(surfaces: &[Key], kingdom: &dyn Fn(&str) -> String) -> Option<String> {
    let names: Vec<String> = surfaces.iter()
        .map(|key| format!("{} {}", crate::held::kingdom_word(&kingdom(key)), crate::held::short_key(key))).collect();
    (!names.is_empty()).then(|| format!(
        "ring: where {} lies on the ground, stronger for more; diamond: bodies carrying it", names.join(" and ")))
}

/// The seeds a world has banked, in words, for one lineage or for all of
/// them; none when it banks none (case 1193).
pub fn banked_words(sim: &Simulation, lineage: Option<&str>) -> Option<String> {
    let banked = readings::propagules(sim);
    let seeds: u64 = banked.iter().filter(|(key, _)| lineage.is_none_or(|l| key.as_str() == l))
        .map(|(_, tally)| tally.count).sum();
    (seeds > 0).then(|| format!("{seeds} {} banked", if seeds == 1 { "seed" } else { "seeds" }))
}

#[cfg(test)]
mod tests;
