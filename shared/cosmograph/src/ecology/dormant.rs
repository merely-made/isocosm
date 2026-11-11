// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Producers' dormant young (cases 1192 to 1197), as both rate families
//! make them. With the world's dormancy rule set, a producer's birth makes a
//! propagule holding its lineage's seed share; three processes per lineage
//! then run only for propagules: upkeep at a fraction of the active rate,
//! death when the matter is spent, and waking on the season's draw once the
//! family's gate holds. World time gates on the site's light share
//! (`world_time::dormant`); the flat rate sets, stock and A1, on the
//! site's crowding of plants, in `flat` here.

use super::{Role, world_time::DETRITUS};
use crate::{Founding, Result, generate::process, rules::*, schema::{Key, Tick}, simulation::Genesis};
use std::collections::BTreeSet;

/// Lineage `i`'s dormant processes, made from its active ones as they stand:
/// `season` is the season in ticks and `gate` what must hold for a seed to
/// wake, besides its being a seed of the lineage.
pub(super) fn seeds(g: &mut Genesis, i: u32, rule: &Dormancy, season: Tick, gate: Vec<Query>) {
    let dormant = || Query::Trait { who: Binding::Actor, key: rule.key.clone() };
    let soil = |key: &mut Key| if key == DETRITUS { *key = "world:soil".into() };
    let processes = &mut g.rules.processes;
    if let Some(active) = processes.get(&format!("ecology:upkeep-{i}")) {
        // The same payment, to the soil, as often as the share of the rate.
        // It is a clone of the active upkeep, so a transition on the body's
        // own clock as that is (D16, case 1076).
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
    // It wakes by a keyed draw each season, once the gate holds.
    let mut wake = process(&format!("ecology:wake-{i}"), Causation::Transition, vec![]);
    wake.requires.extend([Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{i}") }, dormant()]);
    wake.requires.extend(gate);
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

/// The flat rate sets' dormancy, laid after the ecology, its bodies and the
/// mass law are (so the dormant rates are those of the scaled active ones).
/// The producers are the lineages whose role is `Role::Producer`, the ones
/// `configure` gives a `produce` process. A producer's birth gives its seed
/// share where it gave the founding's child mass, so the parent's gate
/// rises or falls with it and the matter it keeps after a birth is as it
/// was. A propagule's matter is in its parts like any young body's, where
/// the dormant upkeep takes it as the active upkeep does. A seed wakes by
/// its season's draw where its site is habitable, as growth requires, and
/// holds fewer plants than `Dormancy::crowding` of its share allows: the
/// flat sets have no sensed light share to read, and a sensing body's is
/// one a growth act already needs at least one of.
pub(super) fn flat(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(mut rule) = founding.dormancy.clone() else { return Ok(()) };
    if !founding.ecology {
        return Err("dormancy needs an ecology to seed".into());
    }
    if founding.bodies.is_none() {
        return Err("dormancy in the flat rate sets needs bodied lineages".into());
    }
    g.rules.traits.insert(rule.key.clone());
    let unit = g.rules.tick_microseconds();
    let season = rule.season_ticks.unwrap_or_else(|| minutes_ticks(rule.season, unit));
    let producers: Vec<u32> = (0..founding.lineages).filter(|&i| Role::of(founding.web, i) == Role::Producer).collect();
    let among: BTreeSet<Key> = producers.iter().map(|i| format!("lineage:{i}")).collect();
    for i in producers {
        let share = rule.drawn(founding.seed, i);
        let body = format!("matter:{i}-0");
        let Some(birth) = g.rules.processes.get_mut(&format!("ecology:birth-{i}")) else { continue };
        let mut was = 0;
        for effect in birth.effects.iter_mut().chain(birth.risk.iter_mut().flat_map(|r| r.effects.iter_mut())) {
            if let Effect::Birth { provision, .. } = effect && let Some(n) = provision.get_mut(&body) {
                was = *n;
                *n = share;
            }
        }
        for q in &mut birth.requires {
            if let Query::Account { key, at_least, .. } = q && *key == body {
                *at_least = at_least.saturating_sub(was).saturating_add(share);
            }
        }
        record(&mut rule, i, share);
        let gate = vec![Query::Condition { key: "world:habitable".into(), at_least: 1 },
            Query::MembersBelow { among: among.clone(), amount: rule.crowding(share) }];
        seeds(g, i, &rule, season, gate);
    }
    g.rules.dormancy = Some(rule);
    Ok(())
}
