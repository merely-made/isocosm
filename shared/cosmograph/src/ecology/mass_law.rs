// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Body-size rates for a generated, bodied ecology (case 1148; the ecology
//! brief, 3.1). Upkeep, meals and a producer's growth scale with adult mass
//! to the three-quarter power, maturity, lifespan and birth spacing to the
//! one-quarter power; walking speed keeps leg length and is not touched.
//! The flat rates hold at the rule's reference mass, so a lineage of exactly
//! that mass keeps them to the byte. Each factor is held within the rule's
//! bounds so no lineage's rate runs away. Unset, nothing here runs and a
//! world founds and hashes as before.
//!
//! Integer arithmetic only. Masses are in milligrams and are scaled by
//! `10^16` before rooting, so the fourth root of a milligram still has four
//! digits (`10^4 * m^(1/4)`) and the three-quarter power six (`10^6 *
//! m^(3/4)`); a mass is held at or under `10^9` mg for the products to fit a
//! `u128`. Upkeep is a period, never a fraction: a body that pays one unit
//! every `p` ticks at the reference pays every `p * tq(ref) / tq(mass)`
//! ticks, and where that falls below one tick it pays more each tick so the
//! rate is kept (S10.2).
//!
//! The reference temperature is whatever the flat rates assumed; no
//! temperature factor is applied. That factor is owed (S10.2).

use super::Rates;
use crate::{Founding, Result, rules::*, schema::{Key, Lineage, Tick}, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The fewest and greatest factor, in hundredths, a rate may be scaled by.
pub const BOUNDS: [u32; 2] = [25, 400];

/// The most mass the roots are taken of, in milligrams.
const MOST_MG: u128 = 1_000_000_000;

/// The scale a mass takes before it is rooted.
const SCALE: u128 = 10_000_000_000_000_000;

/// The body-size rule (case 1148), set on a founding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MassLaw {
    /// The adult mass, in milligrams, at which the flat rates hold. Unset it
    /// is the median adult mass of the world's embodied lineages, and
    /// founding records it here so the world states it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<u64>,
    /// The least and greatest factor, in hundredths, any rate is scaled by.
    pub bounds: [u32; 2],
}

impl Default for MassLaw {
    fn default() -> Self {
        Self { reference: None, bounds: BOUNDS }
    }
}

/// The factors a lineage's rates are scaled by, in hundredths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Factors {
    /// Upkeep and meals: the three-quarter power of the mass ratio.
    pub upkeep: u32,
    /// Maturity, lifespan and birth spacing: the one-quarter power.
    pub time: u32,
}

/// `m^(3/4)` scaled by `10^6`, by two integer roots.
pub fn tq(mass: u64) -> u128 {
    let m = u128::from(mass).clamp(1, MOST_MG) * SCALE;
    (m * m.isqrt()).isqrt()
}

/// `m^(1/4)` scaled by `10^4`.
pub fn q(mass: u64) -> u128 {
    (u128::from(mass).clamp(1, MOST_MG) * SCALE).isqrt().isqrt()
}

/// The factors for a body of `mass` against `reference`, each rounded to a
/// hundredth and held within `bounds`.
pub fn factors(mass: u64, reference: u64, bounds: [u32; 2]) -> Factors {
    let held = |n: u128, d: u128| {
        u32::try_from((n * 100 + d / 2) / d).unwrap_or(u32::MAX).clamp(bounds[0], bounds[1])
    };
    Factors {
        upkeep: held(tq(mass), tq(reference)),
        time: held(q(mass), q(reference)),
    }
}

/// `n` scaled by `factor` hundredths, rounded, at least one.
fn scaled(n: u64, factor: u32) -> u64 {
    let n = (u128::from(n) * u128::from(factor) + 50) / 100;
    u64::try_from(n).unwrap_or(u64::MAX).max(1)
}

/// A period run `factor` hundredths as often, and the multiple of the usual
/// amount to move each time. The rate is `mult / period`, kept to within a
/// hundredth where a multiple of at most eight can do it: a body pays two
/// units every seven ticks sooner than one unit every three, and where the
/// rate wants a period under a tick the amount grows and the period is one.
fn shifted(period: Tick, factor: u32) -> (Tick, u64) {
    let want = u128::from(period) * 100;
    let mut best = (u128::MAX, period, 1);
    for mult in 1..=8u128 {
        let p = (mult * want + u128::from(factor) / 2) / u128::from(factor);
        if p == 0 {
            continue;
        }
        let err = (p * u128::from(factor)).abs_diff(mult * want) * 1_000_000 / (mult * want);
        if err < best.0 {
            best = (err, u64::try_from(p).unwrap_or(u64::MAX), mult as u64);
        }
        if err <= 10_000 {
            break;
        }
    }
    (best.1, best.2)
}

/// The adult mass of a lineage's fully grown body, in milligrams: the sheet's
/// reading of the body its recipe develops (case 1215), the parts' ceilings
/// summed. A lineage with no recipe has none.
///
/// # Errors
/// Fails if the recipe cannot develop a body.
pub fn adult_mass(rules: &Rules, lineages: &BTreeMap<Key, Lineage>, key: &str) -> Result<Option<u64>> {
    let Some(d) = lineages.get(key).and_then(|l| l.development.as_ref()) else {
        return Ok(None);
    };
    let sheet = crate::lineage::sheet::of(rules, lineages, key, d)?;
    Ok(Some(u64::try_from(sheet.mass).unwrap_or(u64::MAX).max(1)))
}

/// The lineages of the ecology that have a body, by index, with adult mass.
///
/// # Errors
/// Fails if a recipe cannot develop a body.
pub fn masses(rules: &Rules, lineages: &BTreeMap<Key, Lineage>) -> Result<BTreeMap<u32, u64>> {
    let mut out = BTreeMap::new();
    for key in lineages.keys() {
        let Some(i) = key.strip_prefix("lineage:").and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        if !rules.processes.contains_key(&format!("ecology:upkeep-{i}")) {
            continue;
        }
        if let Some(mass) = adult_mass(rules, lineages, key)? {
            out.insert(i, mass);
        }
    }
    Ok(out)
}

/// The median of the masses, the mean of the middle two where there are an
/// even number.
pub fn median(masses: &BTreeMap<u32, u64>) -> Option<u64> {
    let mut all: Vec<u64> = masses.values().copied().collect();
    all.sort_unstable();
    let n = all.len();
    let mid = n.checked_sub(1)? / 2;
    let high = all[n / 2];
    Some(((u128::from(all[mid]) + u128::from(high)) / 2) as u64)
}

/// Scales the founded world's rates (case 1148), after the flat processes
/// are configured and the bodies embodied, and records the reference the
/// world's rule states. Does nothing where the founding sets no rule.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(law) = &founding.mass_law else {
        return Ok(());
    };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("the mass law is built for a bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("the mass law layers on the stock and A1 rates, not world time".into());
    }
    if law.bounds[0] == 0 || law.bounds[0] > 100 || law.bounds[1] < 100 {
        return Err("the mass law's bounds must hold the reference factor of 100".into());
    }
    let masses = masses(&g.rules, &g.lineages)?;
    let reference = match law.reference {
        Some(r) if r > 0 => r,
        Some(_) => return Err("the mass law's reference mass is nought".into()),
        None => median(&masses).ok_or("the mass law has no bodied lineage to set a reference")?,
    };
    if let Some(f) = g.founding.as_mut() {
        f.mass_law = Some(MassLaw { reference: Some(reference), bounds: law.bounds });
    }
    for (i, mass) in masses {
        let f = factors(mass, reference, law.bounds);
        scale(&mut g.rules, i, f);
    }
    Ok(())
}

/// Scales lineage `i`'s processes by `f`.
fn scale(rules: &mut Rules, i: u32, f: Factors) {
    let id = |kind: &str| format!("ecology:{kind}-{i}");
    if f.upkeep != 100 {
        if let Some(p) = rules.processes.get_mut(&id("upkeep")) {
            let (period, mult) = shifted(p.period.unwrap_or(1), f.upkeep);
            p.period = Some(period);
            multiply(p, mult);
        }
        // A producer's growth is its meal: the same factor, a period too.
        if let Some(p) = rules.processes.get_mut(&id("produce")) {
            let (period, mult) = shifted(p.period.unwrap_or(1), f.upkeep);
            p.period = Some(period);
            multiply(p, mult);
        }
        if let Some(p) = rules.processes.get_mut(&id("feed")) {
            for e in &mut p.effects {
                if let Effect::Eat { amount: Amount::Fixed(n), .. } = e {
                    *n = scaled(*n, f.upkeep);
                }
            }
        }
    }
    if f.time != 100 {
        for kind in ["birth", "hazard", "age"] {
            let Some(p) = rules.processes.get_mut(&id(kind)) else { continue };
            for q in &mut p.requires {
                if let Query::Age { at_least } = q {
                    *at_least = scaled(*at_least, f.time);
                }
            }
            if kind == "birth" {
                p.period = p.period.map(|t| scaled(t, f.time));
            }
            // A body that lives longer runs its hazard less often.
            if let Some(r) = &mut p.risk {
                r.per_million = u32::try_from(
                    (u64::from(r.per_million) * 100 + u64::from(f.time) / 2) / u64::from(f.time),
                )
                .unwrap_or(u32::MAX)
                .max(1);
            }
        }
    }
}

/// Multiplies every unit a process moves, and the shares it senses, by `n`.
fn multiply(p: &mut Process, n: u64) {
    if n == 1 {
        return;
    }
    for e in &mut p.effects {
        match e {
            Effect::Transform { take, give, .. } => {
                take.values_mut().chain(give.values_mut()).for_each(|v| *v *= n);
            },
            Effect::Transfer { amount: Amount::Fixed(a), .. } => *a *= n,
            _ => {},
        }
    }
    for q in &mut p.requires {
        if let Query::Sensed { at_least, .. } = q {
            *at_least *= n;
        }
    }
}
