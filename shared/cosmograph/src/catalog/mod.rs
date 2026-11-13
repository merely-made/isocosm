// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The catalogs of traits, of effects and of the parts that result: the data
//! half of "trait x kingdom x effect" (mesocosm
//! `design_docs/2026-11-09_lineage_rounds_plan.md`, D37 and D38).
//!
//! **A prototype, and read only.** D37 ruled that "the game part is the
//! variety of the catalog of parts" and that a catalog of traits, one of
//! effects and one of the parts that result can simply be made; D38 lifted
//! the process code's stop rule against catalogs (`process/mod.rs`, header)
//! and kept the memo's check that variety is real
//! (`2026-08-29_elements_and_traits_memo.md` section 2) as a measurement.
//! Nothing here is wired into founding, generation or the tick, so no
//! world's behaviour or hash moves.
//!
//! # What a row is
//!
//! - A **trait** is a standing property of a part (a hide, an enzyme, a
//!   habit), with the process and shape it rides on, the sites it can carry
//!   a payload at, and the kernels that would read it.
//! - An **effect** is what happens at one of five sites ([`Site`]): contact,
//!   being eaten, death and decay, the soil, and eating. The memo's section 7
//!   rules payloads fire at the transfer sites (soil draw and deposit, the
//!   meal, death and decay); contact and eating split the same sites by
//!   whose side a payload sits on.
//! - A **part** is a kingdom's tissue carrying traits and effects. Matter is
//!   typed by provenance, kingdom first (memo section 7), so every part
//!   belongs to one kingdom key the code uses ([`refs::KINGDOMS`]).
//!
//! Parts come two ways, both in datasheets under `datasheets/catalog/`:
//! hand-written rows (`parts_*.toml`) for the distinctive ones, and
//! [`generate`]'s product of the trait and effect rows over kingdoms,
//! filtered for nonsense and deterministic by construction.
//!
//! # Status is derived, never typed
//!
//! A part is [`Status::Expressible`] only when its process is in the native
//! registry and every trait and effect it carries cites an existing rule
//! (`function:`, `kind:` or `rule:` references, each resolved against the
//! code in [`refs`]). Otherwise it is [`Status::NeedsRule`] naming the first
//! missing mechanic, or [`Status::Proposed`] when its process is a verb the
//! registry does not hold. The `[[mechanic]]` rows in `mechanics.toml` are the
//! sim's build list; [`Catalog::waiting_on`] counts parts against each.
//!
//! # Shapes taken from prior art
//!
//! Four data shapes come from mesocosm
//! `design_docs/2026-11-13_effects_prior_art_brief.md` (sections 1, 2, 10);
//! the sources are copyleft or are read for pattern only, so only the shape
//! is used and every row and number is our own:
//! - an effect is keyed by its [`Site`] and carries [`Gate`]s on who it
//!   applies to, with an inverted gate for "harms unless the eater has
//!   tolerance X" (the reagent-condition pattern; D36, D37);
//! - a lasting effect has a [`Timing`]: onset, intensity, duration, decay and
//!   a [`Stacking`] rule, integers only (the onset and duration pattern);
//! - a trait may `unlock` food categories, an enzyme-like entry (the digestion
//!   pattern);
//! - traits carry attack and defence [`Strength`]s by [`Kind`], and every kind
//!   with a defence has an attack to meet it (the attacker, injector,
//!   defender, digestor pattern).
//!
//! No parameter value is taken from any source, so no third-party notice is
//! owed.
//!
//! # Names are provisional
//!
//! Every key is a plain working word. Nothing here is a player-facing name;
//! a naming round is the owner's (doctrine, Naming 3). "Element" is not used
//! for typed matter (the memo's section 7, item 7).

use std::sync::OnceLock;

mod generate;
mod load;
pub mod refs;
mod types;
mod validate;

#[cfg(test)]
mod tests;

pub use generate::{Dropped, generate};
pub use types::{
    Affected, Gate, Kind, Role, Stacking, Strength, Timing, Category, Class, Effect, Mechanic, Model, Origin, Part, ProposedProcess, Site, Status, Trait,
};

/// The whole catalog, loaded once.
#[derive(Clone, Debug)]
pub struct Catalog {
    traits: Vec<Trait>,
    effects: Vec<Effect>,
    parts: Vec<Part>,
    mechanics: Vec<Mechanic>,
    proposed: Vec<ProposedProcess>,
    dropped: Dropped,
}

impl Catalog {
    pub fn traits(&self) -> &[Trait] {
        &self.traits
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// The mechanics the sim lacks, the build list.
    pub fn mechanics(&self) -> &[Mechanic] {
        &self.mechanics
    }

    pub fn proposed_processes(&self) -> &[ProposedProcess] {
        &self.proposed
    }

    /// What the generator refused to emit, by reason.
    pub fn dropped(&self) -> Dropped {
        self.dropped
    }

    pub fn trait_by_key(&self, key: &str) -> Option<&Trait> {
        self.traits.iter().find(|t| t.key == key)
    }

    pub fn effect_by_key(&self, key: &str) -> Option<&Effect> {
        self.effects.iter().find(|e| e.key == key)
    }

    pub fn part_by_key(&self, key: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.key == key)
    }

    pub fn parts_for(&self, kingdom: &str) -> impl Iterator<Item = &Part> {
        self.parts.iter().filter(move |p| p.kingdom == kingdom)
    }

    pub fn parts_serving(&self, model: Model) -> impl Iterator<Item = &Part> {
        self.parts.iter().filter(move |p| p.models.contains(&model))
    }

    /// How many parts wait on each missing mechanic (any, not only first),
    /// most first, ties by key.
    pub fn waiting_on(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
        for part in &self.parts {
            for mechanic in &part.waits_on {
                *counts.entry(mechanic).or_default() += 1;
            }
        }
        let mut list: Vec<(String, usize)> =
            counts.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        list
    }

    /// Checks every rule of the catalog's consistency; the list is empty
    /// when it holds.
    pub fn problems(&self) -> Vec<String> {
        validate::problems(self)
    }
}

/// The embedded catalog, parsed, generated and checked once.
///
/// # Panics
/// Panics if an embedded datasheet is malformed or inconsistent; a test
/// reports the same problems in full.
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let catalog = load::build().expect("the embedded catalog datasheets parse");
        let problems = catalog.problems();
        assert!(problems.is_empty(), "the embedded catalog is inconsistent: {problems:?}");
        catalog
    })
}

pub fn traits() -> &'static [Trait] {
    catalog().traits()
}

pub fn effects() -> &'static [Effect] {
    catalog().effects()
}

pub fn parts() -> &'static [Part] {
    catalog().parts()
}

pub fn parts_for(kingdom: &str) -> impl Iterator<Item = &'static Part> {
    catalog().parts_for(kingdom)
}

pub fn parts_serving(model: Model) -> impl Iterator<Item = &'static Part> {
    catalog().parts_serving(model)
}
