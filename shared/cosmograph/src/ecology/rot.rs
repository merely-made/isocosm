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
//! Unset, no process is added and a world founds and hashes as before.

use super::Rates;
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The soil account producers draw.
const SOIL: &str = "world:soil";

/// How a dead body rots, set on a founding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rot {
    /// The share `[n, d]` of the matter a dead body still holds that it gives
    /// to its site each time it rots, and never less than one unit, so small
    /// corpses finish.
    pub rate: [u64; 2],
}

impl Default for Rot {
    /// A fiftieth of what remains.
    fn default() -> Self {
        Self { rate: [1, 50] }
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
    for i in 0..founding.lineages {
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
