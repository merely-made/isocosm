// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Limits on consumers. A consumer that eats whatever it finds and bears
//! young wherever it stands outruns its food: the A1 played world's grazers
//! overshoot and the world crashes with them. With the founding's limits set,
//! two requires are laid on each consumer (grazer, predator, omnivore and,
//! since ruling D27, the decomposer, whose feed and birth are named like the
//! rest and whose body is the same `matter:{i}-0`; never a producer, which
//! does not feed or breed on others). A founding with limits set before D27
//! now also limits its decomposers (with a ceiling of their own, see
//! [`DECOMPOSER_CEILING`]): set `spare_decomposers` to found as it did.
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
    /// When set to `k`, a consumer also bears young only where the edible
    /// matter its feed could draw from at its site (`EdiblePerHead`) covers
    /// `k` feeding periods of every member's demand (`k` times one meal at
    /// the lineage's pace), so breeding stops before the food is gone, not
    /// after the holders are. Unset, the birth carries no such requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stocked: Option<u64>,
    /// A ceiling for decomposers alone, as `ceiling` is for the rest; the
    /// default is [`DECOMPOSER_CEILING`]. Unset, decomposers take `ceiling`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decomposer_ceiling: Option<[u64; 2]>,
    /// Leaves decomposers without either gate, as limits stood before D27.
    #[serde(default, skip_serializing_if = "is_false")]
    pub spare_decomposers: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Default for Limits {
    fn default() -> Self {
        Self { full: FULL, ceiling: CEILING, stocked: None, decomposer_ceiling: Some(DECOMPOSER_CEILING), spare_decomposers: false }
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
/// The default ceiling for decomposers, ten times the founding share per site
/// (ruling D27). Measured on the same world with flat dormancy and the
/// default fullness (`tests/limits_decomposers.rs`, seeds 3, 5, 9, 11, 17,
/// 600 ticks; battery seeds 0 to 19 all qualify at 3/1, 4/1, 6/1 and 10/1):
/// the decomposer line's swing after tick 100 is cut from peaks of 434, 373
/// and 241 (seeds 11, 3, 17) to 342, 251 and 208 at 10/1, to 133, 98 and
/// 106 at 3/1. At 10/1 no line is gone at tick 600; at 3/1, 4/1 and 6/1 seed
/// 3's producer line ends at 0 where with decomposers spared it ends at 5
/// (and at 0 by tick 900), a knife edge not a cause: 10/1 leaves it at 5 and
/// 20/1 at 0.
pub const DECOMPOSER_CEILING: [u64; 2] = [10, 1];

impl Limits {
    /// The ceiling for a lineage founded with `share` members per site.
    pub fn most(&self, share: u64) -> u64 {
        Self::ratio(self.ceiling, share)
    }

    /// The ceiling for a decomposer lineage founded with `share` per site.
    pub fn decomposer_most(&self, share: u64) -> u64 {
        Self::ratio(self.decomposer_ceiling.unwrap_or(self.ceiling), share)
    }

    fn ratio([n, d]: [u64; 2], share: u64) -> u64 {
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
    if [Some(limits.ceiling), limits.decomposer_ceiling].into_iter().flatten().any(|c| c[0] == 0 || c[1] == 0) {
        return Err("the limits' ceiling must be a ratio of two numbers above nought".into());
    }
    if limits.stocked == Some(0) {
        return Err("the limits' stocking is nought feeding periods".into());
    }
    let sites = (g.sites.len() as u64).max(1);
    let limited = |role: Role| role.consumer() || (role == Role::Decomposer && !limits.spare_decomposers);
    for i in (0..founding.lineages).filter(|&i| limited(Role::of(founding.web, i))) {
        let decomposer = Role::of(founding.web, i) == Role::Decomposer;
        let lineage: Key = format!("lineage:{i}");
        let founded: u64 = g.population.groups.values()
            .filter(|c| c.entity.alive && c.entity.lineage == lineage)
            .map(|c| c.count)
            .sum();
        let share = founded.div_ceil(sites).max(1);
        let most = if decomposer { limits.decomposer_most(share) } else { limits.most(share) };
        // Matter one member asks over `k` feeding periods, read before the
        // fullness gate lands on the feed (it changes no amount).
        let stocked = match limits.stocked {
            Some(k) => {
                let feed = g.rules.processes.get(&format!("ecology:feed-{i}")).ok_or("a stocked consumer has no feed")?;
                let demand = feed.meal_demand(g.lineages.get(&lineage).and_then(|l| l.decides))?;
                Some(k.checked_mul(demand).ok_or("the limits' stocking overflows")?)
            },
            None => None,
        };
        if let Some(feed) = g.rules.processes.get_mut(&format!("ecology:feed-{i}")) {
            feed.requires.push(Query::Below { who: Binding::Actor, key: format!("matter:{i}-0"), amount: limits.full });
        }
        if let Some(birth) = g.rules.processes.get_mut(&format!("ecology:birth-{i}")) {
            birth.requires.push(Query::MembersBelow { among: BTreeSet::from([lineage.clone()]), amount: most });
            if let Some(k) = stocked {
                birth.requires.push(Query::EdiblePerHead { feed: format!("ecology:feed-{i}"), at_least: k, per_members: 1 });
            }
        }
    }
    Ok(())
}
