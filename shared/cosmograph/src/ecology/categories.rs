// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Food categories, an optional founding rule of the A1 and stock ecologies
//! (working name `Categories`), which makes owner ruling D37 true for plants
//! and for the dead (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md`
//! D37, D39 model 4; the traits memo `2026-08-29_elements_and_traits_memo.md`
//! section 7, matter typed by provenance, kingdom first).
//!
//! **Tough plant matter.** "Tough plant matter is a category too": a mouth
//! that eats a category it does not tolerate is "eaten, and it harms"
//! (spoilage's pattern, `super::spoilage`). Which tissue is tough is read
//! from the part a bite lands on, never stored: a part of a plant (a body of
//! the flora kingdom) is *tough* when it holds no `function:fix` cell (stalk,
//! trunk, root: tissue that holds the plant up and feeds it, not the plates
//! that fix light), or holds a `function:support` cell; its fixing plates are
//! soft. A mouth tolerates tough matter with `Tolerance::ToughPlant`, a
//! grinding capability: the dead-stock root has it (a decomposer works wood),
//! and a lowered catalog part gets it from the flows `live:tough` and
//! `dead:wood`. A bulk plant mouth has none, so its bite of tough tissue
//! still eats, the matter moves and the budget stays closed, and the eater
//! takes `harm` cells of wound through `harm::hurt`, as spoilage's bite does.
//! The choice was harm and not a smaller yield: a smaller yield would leave
//! matter uneaten with no stock to hold it, and harm already has its record
//! (`Simulation::harms`).
//!
//! **Dead matter in classes.** Dead matter is classed by what it came from
//! ([`DeadClass`]): *litter*, the dead soft part of a plant; *wood*, its dead
//! tough part; *carrion*, the dead flesh of anything not a plant (which
//! spoilage, where also set, ages fresh to toxic as before). The class is
//! derived at the bite from the body's kingdom and the part bitten
//! (`mosaic::ports::admission::Source`), never stored. A dead-stock mouth
//! with no class limit (the decomposer's root) eats all classes; a mouth
//! limited to some (`IntakePort::taking`) refuses the others, as an
//! unadmitted source is refused. There is no dung: the stock and A1
//! ecologies keep a meal whole and deposit no unkept remainder; only world
//! time's partitioned meal returns a waste (to `world:detritus`, which is not
//! a body's matter), and this rule does not layer on world time.
//!
//! **Rot by class.** Rot (`super::rot`) returns every class to the soil. With
//! `wood_rot_every` above one, a dead body's tough plant parts rot only on
//! every that many ticks of the world, so wood is the slowest; the default,
//! one, is rot's single rate. Matter moves by the same `Rot` effect, so the
//! budget stays closed.
//!
//! Unset, no mouth, meal or rot reads any of this and the world founds,
//! hashes and runs as before.

use crate::{
    Founding, Result,
    ecology::Rates,
    process::{IntakePort, Tolerance},
    schema::{Entity, PartId, Tick},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};

const FIX: &str = "function:fix";
const SUPPORT: &str = "function:support";
const FLORA: &str = "kingdom:flora";

fn one() -> u64 {
    1
}

fn is_one(n: &u64) -> bool {
    *n == 1
}

/// How tough plant matter harms and how the dead rot by class, set on a
/// founding. Working names; integers only; the numbers are placeholders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Categories {
    /// The cells of wound a bite of tough matter costs a mouth with no
    /// grinding: the hazard's unit, as spoilage's `harm`.
    pub harm: u32,
    /// A dead body's tough parts rot once in this many ticks (the tick
    /// divisible by it); one is rot's single rate.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub wood_rot_every: u64,
}

impl Default for Categories {
    /// A bite costs two cells, as spoilage's does; wood rots as fast as
    /// the rest.
    fn default() -> Self {
        Self { harm: 2, wood_rot_every: 1 }
    }
}

/// Whether part `id` of `e` is tough plant tissue: of the flora kingdom, and
/// holding no fixing cell or holding a supporting one. Read from the cells
/// the part holds now, so a regrown or edited body reads as it is.
#[must_use]
pub fn tough(e: &Entity, id: PartId) -> bool {
    e.kingdom == FLORA
        && e.parts.get(&id).is_some_and(|p| {
            let held = |f: &str| p.cells.get(f).copied().unwrap_or(0) > 0;
            !held(FIX) || held(SUPPORT)
        })
}

impl Categories {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.harm == 0 {
            return Err("categories' harm must be at least one cell".into());
        }
        if self.wood_rot_every == 0 {
            return Err("categories' wood_rot_every must be at least one tick".into());
        }
        Ok(())
    }

    /// `port` as a founded mouth has it: a mouth for the dead tolerates
    /// tough plant matter (it is the one that works wood), a mouth for the
    /// living plant has no grinding unless its kind declares it.
    #[must_use]
    pub fn founded(&self, port: IntakePort) -> IntakePort {
        if port.admits_deadstock() { port.tolerating(Tolerance::ToughPlant) } else { port }
    }

    /// Whether a dead body's tough parts rest (do not rot) at `now`.
    #[must_use]
    pub fn rests(&self, now: Tick) -> bool {
        now % self.wood_rot_every != 0
    }
}

/// Lays the categories rule on the founded world: the rule on its rules.
/// Mouths are read at the bite, so nothing is stored on them. Does nothing
/// where the founding sets none.
///
/// # Errors
/// Fails where the founding is no ecology, or runs in world time, or a
/// value is out of range.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.categories else { return Ok(()) };
    if !founding.ecology {
        return Err("categories are built for an ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("categories layer on the stock and A1 rates, not world time".into());
    }
    rule.validate()?;
    g.rules.categories = Some(rule);
    Ok(())
}
