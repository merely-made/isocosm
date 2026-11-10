// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A world-time producer's dormant young (cases 1192 to 1197). With the
//! world's dormancy rule set, a producer's birth makes a propagule holding
//! its lineage's seed share; three processes per lineage then run only for
//! propagules: upkeep at a fraction of the active rate, death when the
//! matter is spent, and waking on the season's draw once the site's light
//! share suits the seed's mass.

use super::{DETRITUS, LIGHT_PER_GROWTH};
use crate::{Result, generate::process, rules::*, schema::{Key, Tick}, simulation::Genesis};

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

/// Lineage `i`'s dormant processes, made from its active ones as they stand:
/// `share` is its seed share and `season` the season in ticks.
pub(super) fn seeds(g: &mut Genesis, i: u32, rule: &Dormancy, share: u64, season: Tick) {
    let dormant = || Query::Trait { who: Binding::Actor, key: rule.key.clone() };
    let soil = |key: &mut Key| if key == DETRITUS { *key = "world:soil".into() };
    let processes = &mut g.rules.processes;
    if let Some(active) = processes.get(&format!("ecology:upkeep-{i}")) {
        // The same payment, to the soil, as often as the share of the rate.
        let mut upkeep = active.clone();
        upkeep.id = format!("ecology:dormant-upkeep-{i}");
        upkeep.requires.push(dormant());
        let [n, d] = rule.upkeep;
        upkeep.period = active.period.map(|t| (t.saturating_mul(d) / n).max(1));
        for effect in &mut upkeep.effects {
            match effect {
                Effect::Transform { give, .. } => if let Some(n) = give.remove(DETRITUS) { give.insert("world:soil".into(), n); },
                Effect::Transfer { account, .. } => soil(account),
                Effect::Spend { into: Some(into), .. } => soil(into),
                _ => {},
            }
        }
        processes.insert(upkeep.id.clone(), upkeep);
    }
    if let Some(active) = processes.get(&format!("ecology:death-{i}")) {
        // A propagule with no matter left dies; nothing remains to go to the site.
        let mut death = active.clone();
        death.id = format!("ecology:dormant-death-{i}");
        death.requires.push(dormant());
        processes.insert(death.id.clone(), death);
    }
    // It wakes by a keyed draw each season, once its site's light share
    // covers what a seedling needs, less as the seed is heavier.
    let mut wake = process(&format!("ecology:wake-{i}"), Causation::Transition, vec![]);
    wake.requires.extend([Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{i}") }, dormant(),
        Query::Sensed { reading: "sense:light-share".into(), at_least: Dormancy::waking(LIGHT_PER_GROWTH, share) }]);
    wake.risk = Some(Risk { per_million: rule.wakes(), aging: None, effects: vec![Effect::Trait {
        who: Binding::Actor, key: rule.key.clone(), present: false }] });
    wake.period = Some(season);
    wake.priority = 12;
    wake.note = true;
    processes.insert(wake.id.clone(), wake);
}

/// The birth processes' ledger entry: seed share by process.
pub(super) fn record(rule: &mut Dormancy, i: u32, share: u64) {
    rule.seeds.insert(format!("ecology:birth-{i}"), share);
}
