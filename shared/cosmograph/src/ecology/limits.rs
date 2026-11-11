// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Limits on the consumers of the living. A consumer that eats whatever it
//! finds and bears young wherever it stands outruns its food: the A1 played
//! world's grazers overshoot and the world crashes with them. With the
//! founding's limits set, two requires are laid on each consumer of the
//! living (grazer, predator, omnivore; not producers, not decomposers):
//!
//! - fullness: its `ecology:feed-{i}` holds only while it holds less than
//!   `full` of its own tissue, so a full body does not eat;
//! - a ceiling per site: its `ecology:birth-{i}` holds only where fewer than
//!   `most` of its lineage stand at its site. A number that fits one world
//!   does not fit another (the A1 played world founds 32 grazers on 6
//!   sites), so the ceiling is a ratio of the lineage's founding share per
//!   site, founded members over sites rounded up, at least one:
//!   `most = ceil(share * n / d)`.
//!
//! Unset, nothing here runs and a world founds and hashes as before. The
//! mass law scales the feed and birth processes in place and dormancy copies
//! only upkeep and death, so the gates, laid last, are on the processes the
//! world runs.

use super::{Rates, Role};
use crate::{Founding, Result, rules::*, schema::Key, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The limits on consumers of the living, set on a founding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    /// A consumer feeds only while it holds less than this of its own
    /// tissue. A founded body holds 10 and a grazer breeds at 9, so 12 lets
    /// a body fill past its breeding gate and no further.
    pub full: u64,
    /// The most of its lineage a site holds before its consumers stop
    /// bearing young, as a ratio `[n, d]` of the lineage's founding share per
    /// site.
    pub ceiling: [u64; 2],
}

impl Default for Limits {
    fn default() -> Self {
        Self { full: FULL, ceiling: CEILING }
    }
}

/// The default fullness, in tissue units: above a consumer's breeding gate
/// of 9 and a founded body's 10. Measured on the played A1 world
/// (`tests/limits_measure.rs`, seeds 3, 5, 9, 11, 17, 600 ticks): fullness
/// of 10, 12 or 15 moves the outcome little beside the ceiling; 12 is the
/// middle.
pub const FULL: u64 = 12;
/// The default ceiling, three times the founding share per site (18 grazers
/// where 32 are founded on 6 sites). Measured on the same world: a ceiling of
/// twice the share cuts the grazer peak most, but leaves 18 of 20 draws of
/// the A1 reproduction battery (played) qualifying, seeds 10 and 13 failing;
/// three times keeps all 20 and still cuts the peak by about half.
pub const CEILING: [u64; 2] = [3, 1];

impl Limits {
    /// The ceiling for a lineage founded with `share` members per site.
    pub fn most(&self, share: u64) -> u64 {
        let [n, d] = self.ceiling;
        (u128::from(share) * u128::from(n)).div_ceil(u128::from(d.max(1))).try_into().unwrap_or(u64::MAX)
    }
}

/// Lays the limits on the founded world's consumers. Does nothing where the
/// founding sets none.
///
/// # Errors
/// Fails where the founding is not a bodied stock or A1 ecology, or the
/// limits name no fullness or a ceiling that is nought or has no divisor.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(limits) = &founding.limits else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("the limits are built for a bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("the limits layer on the stock and A1 rates, not world time".into());
    }
    if limits.full == 0 {
        return Err("the limits' fullness is nought".into());
    }
    if limits.ceiling[0] == 0 || limits.ceiling[1] == 0 {
        return Err("the limits' ceiling must be a ratio of two numbers above nought".into());
    }
    let sites = (g.sites.len() as u64).max(1);
    for i in (0..founding.lineages).filter(|&i| Role::of(founding.web, i).consumer()) {
        let lineage: Key = format!("lineage:{i}");
        let founded: u64 = g.population.groups.values()
            .filter(|c| c.entity.alive && c.entity.lineage == lineage)
            .map(|c| c.count)
            .sum();
        let most = limits.most(founded.div_ceil(sites).max(1));
        if let Some(feed) = g.rules.processes.get_mut(&format!("ecology:feed-{i}")) {
            feed.requires.push(Query::Below { who: Binding::Actor, key: format!("matter:{i}-0"), amount: limits.full });
        }
        if let Some(birth) = g.rules.processes.get_mut(&format!("ecology:birth-{i}")) {
            birth.requires.push(Query::MembersBelow { among: BTreeSet::from([lineage]), amount: most });
        }
    }
    Ok(())
}
