// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rot, an optional rule of the A1 and stock ecologies. A dead body keeps its
//! matter until a decomposer eats it, so a world with few decomposers locks
//! matter up in its dead while the producers' soil runs down. With rot set on
//! a founding, each lineage gets an agentless process that falls due every
//! tick: at each site it draws one of that lineage's dead there (weighted by
//! the matter they hold, as a meal is) and the body gives a fraction of what
//! it still holds, at least one unit, to the site.
//!
//! The matter arrives as `world:soil`, the account producers draw (`produce`
//! transfers soil, never a lineage's own account). The `Rot` effect would
//! otherwise credit the site each account as itself, `matter:{i}-0`, which
//! nothing at a site draws from: a decomposer eats bodies, not the ground.
//! So the effect names `into`, the same return upkeep makes of a body's
//! matter. The matter moves by the ordinary `Rot` effect, so the budget stays
//! closed and the flow record has its legs from the body to the site.
//!
//! The process takes one body per lineage per site per tick, as Mesocosm's
//! `body:rot` takes one per site, so a corpse's share of the rate falls as
//! more dead lie at its site. A dead body keeps its matter in place for the
//! decomposers: rot takes a part and the meal is of what remains.
//!
//! A decomposer does the same work faster: `decomposer_bites` multiplies the
//! decomposers' feedings, so more of a corpse's matter goes to their bodies
//! before rot returns it to the soil. In a bodied world nothing eats a
//! decomposer yet: a mouth eats what its intake port admits, and no port
//! admits their matter.
//!
//! With the categories rule (`super::categories`), a dead plant's tough parts
//! rot only once in `wood_rot_every` ticks; the default is this single rate.
//!
//! Unset, no process is added and a world founds and hashes as before.

use super::{Rates, Role};
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The soil account producers draw.
const SOIL: &str = "world:soil";

/// The decomposer bites a world that wants decomposers faster than rot
/// founds with (Mesocosm's default). Measured on the played A1 world
/// (`tests/corpse_share.rs`, seeds 0 to 19, 600 ticks, flat dormancy, default
/// limits, rot a fiftieth): with rot and one bite the decomposers took 52
/// percent of the dead matter that left corpses and ended at a median of 72;
/// 2, 3, 5 and 8 bites took 61, 63, 65 and 66 percent and a median of 82,
/// 94, 86 and 100; 12 took 66.5 percent, a median of 112 (101 with no rot),
/// and 2 seeds losing a line (3 with rot alone, 3 with no rot). A bigger
/// meal, a higher decomposer ceiling or none at all moved nothing: a bite
/// takes one part of a corpse and the parts are small.
pub const DECOMPOSER_BITES: u64 = 12;

/// How a dead body rots, set on a founding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rot {
    /// The share `[n, d]` of the matter a dead body still holds that it gives
    /// to its site each time it rots, and never less than one unit, so small
    /// corpses finish.
    pub rate: [u64; 2],
    /// When set to `k`, a decomposer feeds `k` times where it fed once, each
    /// feeding drawing its own bite of the corpse. A bite is one part of a
    /// body and a part holds a unit or two, so a bigger meal takes no more
    /// than the part it bites; more bites take more. With rot as the
    /// baseline the decomposers do the same work faster, and what they eat
    /// is held in their bodies, not returned to the
    /// soil. Unset, a decomposer feeds as it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decomposer_bites: Option<u64>,
}

impl Default for Rot {
    /// A fiftieth of what remains.
    fn default() -> Self {
        Self { rate: [1, 50], decomposer_bites: None }
    }
}

impl Rot {
    /// What one rotting gives: the share of `held`, at least one. The
    /// process computes the same by an expression, over a body's own
    /// account; this is the number it comes to.
    #[must_use]
    pub fn gives(&self, held: u64) -> u64 {
        let [n, d] = self.rate;
        let share = u128::from(held) * u128::from(n) / u128::from(d.max(1));
        u64::try_from(share).unwrap_or(u64::MAX).max(1)
    }

    fn amount(&self, key: &str) -> Amount {
        let [n, d] = self.rate;
        let held = Expr::Read(Reading::Account { who: Binding::Target, key: key.into() });
        let share = Expr::Div(
            Box::new(Expr::Mul(vec![held, Expr::Const(i64::try_from(n).unwrap_or(i64::MAX))])),
            Box::new(Expr::Const(i64::try_from(d).unwrap_or(i64::MAX))),
        );
        Amount::Computed(Expr::Max(vec![Expr::Const(1), share]))
    }
}

/// Lays rot on the founded world's lineages. Does nothing where the founding
/// sets none.
///
/// # Errors
/// Fails where the founding is no ecology, or runs in world time (whose dead
/// matter has its own return), or the rate is nought or has no divisor.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rot) = &founding.rot else { return Ok(()) };
    if !founding.ecology {
        return Err("rot is built for an ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("rot layers on the stock and A1 rates, not world time".into());
    }
    if rot.rate[0] == 0 || rot.rate[1] == 0 {
        return Err("rot's rate must be a ratio of two numbers above nought".into());
    }
    if rot.decomposer_bites == Some(0) {
        return Err("rot's decomposer bites are nought feedings".into());
    }
    for i in 0..founding.lineages {
        if let Some(k) = rot.decomposer_bites
            && Role::of(founding.web, i) == Role::Decomposer
            && let Some(feed) = g.rules.processes.get(&format!("ecology:feed-{i}")).cloned()
        {
            for n in 1..k {
                let mut more = feed.clone();
                more.id = format!("ecology:feed-{i}-{n}");
                g.rules.processes.insert(more.id.clone(), more);
            }
        }
        let mut p = process(
            &format!("ecology:rot-{i}"),
            Causation::Agentless,
            vec![Effect::Rot { who: Binding::Target, amount: rot.amount(&format!("matter:{i}-0")), into: Some(SOIL.into()) }],
        );
        p.requires.push(Query::Holds { who: Binding::Target, at_least: 1 });
        p.target = Some(Target {
            same_place: true,
            alive: Some(false),
            lineage: None,
            among: BTreeSet::from([format!("lineage:{i}")]),
            weighted: true,
        });
        p.period = Some(1);
        // After the meals (priority 10 at most), so a body is eaten and then rotted.
        p.priority = 50;
        g.rules.processes.insert(p.id.clone(), p);
    }
    Ok(())
}
