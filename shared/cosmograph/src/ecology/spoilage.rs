// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Spoilage, an optional founding rule of the A1 and stock ecologies, which
//! makes two things the owner ruled (`mesocosm/design_docs/
//! 2026-11-09_lineage_rounds_plan.md`, D36 and D37) true in the sim.
//!
//! **Dead flesh ages.** A dead body's flesh is fresh, then aged, then toxic,
//! by the ticks since it died: "plenty of scavengers and predators like cats
//! eat newly dead to slightly old dead flesh ... the quality of the dead
//! flesh would degrade with time into toxins". The class is derived each
//! time from `Entity::died` and the tick, and never stored. Only flesh
//! (animal and decomposer matter) ages; a dead plant's matter stays as it was
//! (tough plant matter is a category the owner has not ruled).
//!
//! **Eaten, and it harms.** A mouth that meets matter it does not tolerate
//! is not refused: the meal happens, the matter moves, and the eater takes
//! `harm` cells of wound, as a hazard's hit does (`crate::harm`). What a
//! mouth tolerates is on its intake port (`process::Tolerance`), and what it
//! admits widens under the rule: a mouth that admits living animal matter
//! also bites the dead flesh of animals, and the living and dead flesh of
//! decomposers, which is toxic to every mouth without that tolerance (D36).
//! Founding gives a jaw (a mouth admitting living animal matter) a tolerance
//! of aged flesh, and the decomposer's root (a mouth for the dead) a
//! tolerance of all dead flesh and of decomposer flesh; the bulk plant mouth
//! keeps none.
//!
//! Rot is unchanged: spoilage only classifies and harms. Unset, the world
//! founds, hashes and runs as before, and a meal of matter a mouth does not
//! admit is refused. The harm of a bite is recorded in `Simulation::harms`.

use super::Rates;
use crate::{
    Founding, Result,
    mosaic::ports::admission::Source,
    process::{IntakePort, Tolerance},
    rules::{FeedingSources, Rules},
    schema::{Entity, Tick},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// How old a dead body's flesh is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Flesh {
    Fresh,
    Aged,
    Toxic,
}

fn yes() -> bool {
    true
}

fn is_yes(b: &bool) -> bool {
    *b
}

/// How dead flesh ages and what a bite of what a mouth does not tolerate
/// costs, set on a founding. Working names; integers only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spoilage {
    /// Ticks since death from which a body's flesh is aged.
    pub aged_after: Tick,
    /// Ticks since death from which it is toxic.
    pub toxic_after: Tick,
    /// The cells of wound a bite of untolerated matter costs its eater: the
    /// hazard's unit (`harm::hazard::CELLS`).
    pub harm: u32,
    /// Whether decomposer flesh, living or dead, is toxic to a mouth with no
    /// tolerance for it (D36). Set by default.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub decomposer_flesh_toxic: bool,
}

impl Default for Spoilage {
    /// Aged after 30 ticks and toxic after 100, against rot's default of a
    /// fiftieth a tick; a bite costs two cells. Placeholders: nothing has
    /// measured them.
    fn default() -> Self {
        Self { aged_after: 30, toxic_after: 100, harm: 2, decomposer_flesh_toxic: true }
    }
}

impl Spoilage {
    /// The class of the flesh of a body that died at `died` (unknown if
    /// none, which reads fresh), at tick `now`.
    #[must_use]
    pub fn class(&self, died: Option<Tick>, now: Tick) -> Flesh {
        let Some(age) = died.map(|d| now.saturating_sub(d)) else { return Flesh::Fresh };
        match age {
            a if a >= self.toxic_after => Flesh::Toxic,
            a if a >= self.aged_after => Flesh::Aged,
            _ => Flesh::Fresh,
        }
    }

    /// The class of the flesh `e` holds at `now`: fresh while it lives.
    #[must_use]
    pub fn of(&self, e: &Entity, now: Tick) -> Flesh {
        if e.alive { Flesh::Fresh } else { self.class(e.died, now) }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.aged_after == 0 {
            return Err("spoilage's aged_after must be above nought ticks".into());
        }
        if self.toxic_after <= self.aged_after {
            return Err("spoilage's toxic_after must be later than its aged_after".into());
        }
        if self.harm == 0 {
            return Err("spoilage's harm must be at least one cell".into());
        }
        Ok(())
    }

    /// `port` as a founded mouth has it under this rule: a mouth for the
    /// dead tolerates all dead flesh and decomposer flesh, a mouth admitting
    /// living animal matter (a jaw) tolerates aged flesh, a mouth for plants
    /// alone tolerates none. A tolerance already declared stays.
    #[must_use]
    pub fn founded(&self, port: IntakePort) -> IntakePort {
        use crate::process::NisKind::Consumer;
        let mut port = port;
        if port.admits_deadstock() {
            port = port
                .tolerating(Tolerance::AgedFlesh)
                .tolerating(Tolerance::ToxicFlesh)
                .tolerating(Tolerance::DecomposerFlesh);
        }
        if port.admits_live(Consumer) {
            port = port.tolerating(Tolerance::AgedFlesh);
        }
        port
    }
}

/// Lays spoilage on the founded world: the rule on its rules, the founded
/// mouths' tolerances, and the feeding processes' sources, which widen with
/// what a mouth now admits. Does nothing where the founding sets none.
///
/// # Errors
/// Fails where the founding is no ecology, or runs in world time (whose
/// meals are partitioned), or a value is out of range.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.spoilage else { return Ok(()) };
    if !founding.ecology {
        return Err("spoilage is built for an ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("spoilage layers on the stock and A1 rates, not world time".into());
    }
    rule.validate()?;
    g.rules.spoilage = Some(rule);
    for template in g.rules.kinds.values_mut() {
        template.port = template.port.map(|port| rule.founded(port));
    }
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        let ports: Vec<_> = e.parts.iter().filter_map(|(id, p)| p.port.map(|port| (*id, port))).collect();
        for (id, port) in ports {
            if let Some(part) = e.part_mut(&id) {
                part.port = Some(rule.founded(port));
            }
        }
    }
    refeed(g)
}

/// Works out again which lineages each lineage's ordinary meal admits, and
/// whether its prey is living, dead or either, from the mouth its founders
/// carry, as founding did before the rule widened what a mouth admits.
pub(crate) fn refeed(g: &mut Genesis) -> Result<()> {
    let ids: Vec<_> = g
        .rules
        .processes
        .iter()
        .filter(|(id, p)| {
            id.starts_with("ecology:feed-")
                && p.target.is_some()
                && p.port_feeding.as_ref().is_some_and(|f| f.meal.is_none() && matches!(f.sources, FeedingSources::Only(_)))
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in ids {
        let lineage = id.trim_start_matches("ecology:feed-").split('-').next().map(|i| format!("lineage:{i}"));
        let process = g.rules.processes[&id].clone();
        let feeding = process.port_feeding.as_ref().ok_or("a feed has no port")?;
        let port = g
            .population
            .groups
            .values()
            .filter(|c| Some(&c.entity.lineage) == lineage.as_ref())
            .find_map(|c| feeding.part(&c.entity).ok().and_then(|(_, part)| part.port));
        let Some(port) = port else { continue };
        let (sources, alive) = admitted(&g.rules, &g.lineages, port)?;
        let p = g.rules.processes.get_mut(&id).ok_or("a feed vanished")?;
        if let Some(f) = &mut p.port_feeding {
            f.sources = FeedingSources::Only(sources);
        }
        if let Some(target) = &mut p.target {
            target.alive = alive;
        }
    }
    Ok(())
}

/// The lineages whose matter `port` admits, living or dead, and whether it
/// takes only the living, only the dead, or either.
fn admitted(
    rules: &Rules,
    lineages: &std::collections::BTreeMap<String, crate::schema::Lineage>,
    port: IntakePort,
) -> Result<(BTreeSet<String>, Option<bool>)> {
    let (mut sources, mut seen) = (BTreeSet::new(), [false; 2]);
    for account in rules.accounts.keys() {
        if !matches!(rules.accounts[account], crate::rules::AccountKind::Matter { .. }) {
            continue;
        }
        for (slot, living) in [true, false].into_iter().enumerate() {
            let source = Source::of(account, living, rules, lineages)?;
            if source.admitted_by(port).is_ok() {
                sources.insert(source.lineage().to_owned());
                seen[slot] = true;
            }
        }
    }
    Ok((sources, match seen {
        [true, true] => None,
        [false, true] => Some(false),
        _ => Some(true),
    }))
}
