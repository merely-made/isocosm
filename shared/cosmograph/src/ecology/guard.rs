// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Guard, an optional founding rule of the bodied A1 and stock ecologies
//! (working name `Guard`; `crate::defence` and `Rules::defence` are the
//! pack-authored protected-part law and keep that word), which makes owner
//! model 2, defence and counter-defence, true (`mesocosm/design_docs/
//! 2026-11-09_lineage_rounds_plan.md` D39: "toxin, armour, spines, tough
//! tissue against tolerance, grinding and piercing mouths; every defence
//! costs matter"; the traits memo `2026-08-29_elements_and_traits_memo.md`
//! section 1C, payloads at the meal, "payloads act on bodies, payloads do
//! not act on payloads"; the effects brief `2026-11-13_effects_prior_art_brief.md`,
//! ALIEN's attacker and defender strengths, BSD-3-Clause, and CDDA's armour
//! per part, pattern only).
//!
//! **Where each lives.** A part's defence is on its port
//! ([`crate::process::Strength`]), the record a mouth already carries, held
//! by an organ that is not a mouth and active only while the function under
//! it (support for armour and spines, secrete for a gland) holds cells there,
//! so a wound that takes the cells takes the defence. A mouth carries its
//! attack by kind in the same record. Nothing else is stored; the readings
//! are in [`read`], and the bite's glue in `stage::staged::meal::guard`.
//!
//! **Armour** resists a bite *of the part it is on*. A mouth bites by its
//! stronger attack (crush, else pierce); the part's armour of that kind, less
//! the mouth's attack of it, is the resistance in tenths (at most nine), and
//! the bite takes that many tenths less, in whole units by the bite's own
//! draw (a bite of one unit against four tenths is taken six times in ten).
//! What is not taken stays in the prey, so the budget stays closed. A founded mouth, which declares
//! no attack, bites with [`Guard::bite`].
//!
//! **Contact harm** (a spine, a nettle's hair) wounds the eater of the part
//! it is on by [`Guard::spine_harm`] cells, through `harm::hurt` as spoilage's
//! bite does, unless the eater's body tolerates it (`Tolerance::Spines`) or
//! its mouth's pierce attack is at least the spine's (it works between them).
//! The meal still happens: "eaten, and it harms".
//!
//! **A gland** (`Process::Secrete`) charges toxin to whatever eats the body:
//! the sum, over its secreting cells, of each cell's strength (the part's
//! declared one, else [`Guard::gland`]) is the load, and the eater takes one
//! cell of wound per [`Guard::dose`] of it, at least one and at most
//! [`Guard::toxin_cap`], unless its body tolerates it (`Tolerance::Toxin`).
//!
//! **What it costs.** Every guard cell is a cell: it is on the part's
//! capacity, so it is a cell that part does not give to intake, fixing or
//! contraction, and it is matter the body holds. And each pays standing rent:
//! a process beside each lineage's upkeep (`guard:rent-<i>`, as
//! `climate:strain-<i>` is) spends one matter, to the soil, for each
//! [`Guard::rent_cells`] cells the body holds of support and of secrete,
//! rounded up, so any guard costs at least what upkeep does. That is
//! `Process::Secrete`'s "standing rent".
//!
//! Unset, no process is added, nothing reads a port's strength, and the world
//! founds, hashes and runs as before.

use super::Rates;
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use serde::{Deserialize, Serialize};

pub mod read;
pub use read::Ward;

const SUPPORT: &str = "function:support";
const SECRETE: &str = "function:secrete";

/// How armour, spines and glands act and cost, set on a founding. Working
/// names; integers only; every number is a placeholder nothing has measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guard {
    /// The crush attack of a mouth that declares none, on the catalog's
    /// nought to ten scale: a founded mouth's.
    pub bite: u8,
    /// The toxin strength of a secreting cell whose part declares none.
    pub gland: u8,
    /// The cells of wound a bite of a spined part costs the eater.
    pub spine_harm: u32,
    /// The toxin load (strength times secreting cells) that costs one cell.
    pub dose: u32,
    /// The most cells one bite of a toxic body costs.
    pub toxin_cap: u32,
    /// The guard cells (support and secrete) that pay one matter of rent at
    /// each upkeep, rounded up.
    pub rent_cells: u32,
}

impl Default for Guard {
    fn default() -> Self {
        Self { bite: 2, gland: 2, spine_harm: 2, dose: 16, toxin_cap: 4, rent_cells: 8 }
    }
}

impl Guard {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.bite > 10 || self.gland > 10 {
            return Err("guard's bite and gland are strengths of nought to ten".into());
        }
        if self.spine_harm == 0 {
            return Err("guard's spine_harm must be at least one cell".into());
        }
        if self.dose == 0 {
            return Err("guard's dose must be above nought".into());
        }
        if self.toxin_cap == 0 {
            return Err("guard's toxin_cap must be at least one cell".into());
        }
        if self.rent_cells == 0 {
            return Err("guard's rent_cells must be above nought".into());
        }
        Ok(())
    }

    /// One lineage's standing rent: one matter for each `rent_cells` cells
    /// of support and secrete, rounded up.
    fn rent(&self) -> Expr {
        let cells = |function: &str| Expr::Read(Reading::Cells { who: Binding::Actor, function: function.into() });
        let held = Expr::Add(vec![cells(SUPPORT), cells(SECRETE), Expr::Const(i64::from(self.rent_cells) - 1)]);
        Expr::Div(Box::new(held), Box::new(Expr::Const(i64::from(self.rent_cells))))
    }
}

/// Lays the guard rule on the founded world: the rule on its rules, and a
/// rent process beside each lineage's upkeep. Does nothing where the founding
/// sets none.
///
/// # Errors
/// Fails where the founding is no bodied ecology, runs in world time, a
/// value is out of range, or the rent processes pass the world's limit.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.guard else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("guard is built for the bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("guard layers on the stock and A1 rates, not world time".into());
    }
    rule.validate()?;
    g.rules.guard = Some(rule);
    let mut made = Vec::new();
    for i in 0..founding.lineages {
        let Some(upkeep) = g.rules.processes.get(&format!("ecology:upkeep-{i}")) else { continue };
        let Some((from, into)) = upkeep.effects.iter().find_map(crate::climate::paying) else { continue };
        let spend = Effect::Spend { from, to: Binding::Place, amount: Amount::Computed(rule.rent()), into: Some(into) };
        let mut rent = process(&format!("guard:rent-{i}"), Causation::Transition, vec![spend]);
        rent.requires = upkeep.requires.iter().filter(|q| matches!(q, Query::Trait { .. })).cloned().collect();
        rent.period = upkeep.period;
        rent.priority = upkeep.priority;
        made.push(rent);
    }
    if g.rules.processes.len() + made.len() > g.rules.limits.processes {
        return Err("guard's rent per lineage exceeds the world's process limit".into());
    }
    g.rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    Ok(())
}
