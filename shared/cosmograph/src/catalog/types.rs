// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The catalog's row types: what a datasheet row parses to, and what the
//! loader derives from it (a status, never a stored verdict).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The five places a payload fires (elements-and-traits memo section 7,
/// item 1: "scheme C's, firing on provenance at the three transfer sites";
/// the soil draw and deposit, the meal and death each split by whose side of
/// it the payload sits on, plus the passive contact of two bodies).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Site {
    /// Bodies touching or sharing a place, with nobody eating anybody.
    Contact,
    /// What the eater meets: this body as a meal.
    Eaten,
    /// This body dead and decaying.
    Death,
    /// What this body draws from or deposits into the soil.
    Soil,
    /// What this body does when it is the one eating.
    Eating,
}

impl Site {
    pub const ALL: [Site; 5] = [Site::Contact, Site::Eaten, Site::Death, Site::Soil, Site::Eating];
}

/// The ecological models the owner approved (lineage-rounds D39), in the
/// order the owner listed them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    /// 1. Competition for limiting resources.
    Competition,
    /// 2. Defence and counter-defence.
    Defence,
    /// 3. Predator and prey, with refuges.
    PredatorPrey,
    /// 4. The detritus web.
    Detritus,
    /// 5. Mutualism.
    Mutualism,
    /// 6. Parasites and disease.
    Parasite,
    /// 7. Succession and disturbance.
    Succession,
}

impl Model {
    pub const ALL: [Model; 7] = [
        Model::Competition,
        Model::Defence,
        Model::PredatorPrey,
        Model::Detritus,
        Model::Mutualism,
        Model::Parasite,
        Model::Succession,
    ];
}

/// What kind of thing a trait is, so the generator can say which effects
/// make sense riding on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Structure,
    Chemistry,
    Metabolism,
    Behaviour,
    Lifehistory,
    Symbiosis,
}

/// What an effect does to the world, coarsely.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Harm,
    Nourish,
    Resource,
    Signal,
    Shelter,
    Transport,
    Structure,
}

/// Whether the sim can run a row today.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Runnable through these existing rules (see `refs`).
    Expressible { via: Vec<String> },
    /// Waits on a named mechanic the sim lacks (a `[[mechanic]]` row).
    NeedsRule { mechanic: String },
    /// Rests on a proposed process, a verb the registry does not hold.
    Proposed { process: String },
}

impl Status {
    pub fn word(&self) -> &'static str {
        match self {
            Status::Expressible { .. } => "expressible",
            Status::NeedsRule { .. } => "needs_rule",
            Status::Proposed { .. } => "proposed",
        }
    }
}

/// How a row got into the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Authored,
    Generated,
}

/// Who a gate looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Affected {
    /// The body doing the eating (the meal's far side).
    Eater,
    /// The body being fed on or infected.
    Host,
    /// The body being attacked or eaten.
    Prey,
    /// The body that touches this one.
    Toucher,
}

/// A condition on who an effect applies to: it applies when the affected
/// body has `requires` (a trait key, or a `FLOWS` food category its traits
/// unlock), or, `inverted`, when it does not. "Harms unless the eater has
/// tolerance X" is an inverted gate (D36, D37). Shape after the reagent
/// conditions of the prior art brief, section 10, item 2; rows are ours.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    pub affected: Affected,
    pub requires: String,
    #[serde(default)]
    pub inverted: bool,
}

/// How repeated doses of a lasting effect combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stacking {
    /// Intensities add.
    Stack,
    /// A new dose restarts the duration; intensity stays.
    Refresh,
    /// Only the strongest dose counts.
    Strongest,
}

/// A lasting effect's timing, all integers in world ticks and whole
/// intensity steps (no floats, no randomness): wait `onset`, act at
/// `intensity` for `duration`, then lose `decay` intensity a tick. Shape
/// after the onset, duration, intensity, decay and stacking fields of the
/// brief's section 10, items 1 and 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timing {
    pub onset: u32,
    pub intensity: u32,
    pub duration: u32,
    pub decay: u32,
    pub stacking: Stacking,
}

/// The kinds of harm an attack and a defence are paired by (brief section
/// 10, item 4: attacker, injector, defender and digestor strengths).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Crush,
    Pierce,
    Inject,
    Toxin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Attack,
    Defend,
}

/// A strength of one role against one kind; the numbers are provisional
/// placeholders on a 0 to 10 scale, written here, not taken from any source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strength {
    pub kind: Kind,
    pub role: Role,
    pub amount: u32,
}

/// A trait row as the datasheet writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TraitRow {
    pub key: String,
    pub summary: String,
    pub category: Category,
    pub process: String,
    pub shape: String,
    pub kingdoms: Vec<String>,
    pub models: Vec<Model>,
    pub sites: Vec<Site>,
    #[serde(default)]
    pub admits: Vec<String>,
    #[serde(default)]
    pub emits: Vec<String>,
    pub cost_mg: u32,
    #[serde(default)]
    pub reads: Vec<String>,
    pub analogue: String,
    #[serde(default)]
    pub via: Vec<String>,
    #[serde(default)]
    pub needs: Option<String>,
    #[serde(default)]
    pub unlocks: Vec<String>,
    #[serde(default)]
    pub strengths: Vec<Strength>,
}

/// An effect row as the datasheet writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EffectRow {
    pub key: String,
    pub summary: String,
    pub site: Site,
    pub class: Class,
    pub models: Vec<Model>,
    /// Trait categories this effect may ride on (the generator's filter).
    pub accepts: Vec<Category>,
    /// Native function names (or proposed process names) of the traits it
    /// may ride on; empty is any.
    #[serde(default)]
    pub processes: Vec<String>,
    /// When not empty, the only traits it may ride on (a fine filter for
    /// pairs the other clauses cannot tell apart).
    #[serde(default)]
    pub only: Vec<String>,
    /// Empty is any kingdom.
    #[serde(default)]
    pub kingdoms: Vec<String>,
    #[serde(default)]
    pub admits: Vec<String>,
    #[serde(default)]
    pub emits: Vec<String>,
    pub cost_mg: u32,
    pub analogue: String,
    #[serde(default)]
    pub via: Vec<String>,
    #[serde(default)]
    pub needs: Option<String>,
    #[serde(default)]
    pub gates: Vec<Gate>,
    #[serde(default)]
    pub timing: Option<Timing>,
}

/// A hand-written part as the datasheet writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PartRow {
    pub key: String,
    pub kingdom: String,
    pub process: String,
    pub shape: String,
    pub traits: Vec<String>,
    #[serde(default)]
    pub effects: BTreeMap<Site, Vec<String>>,
    #[serde(default)]
    pub admits: Vec<String>,
    #[serde(default)]
    pub emits: Vec<String>,
    /// Omitted, the sum of its traits' and effects' costs.
    #[serde(default)]
    pub cost_mg: Option<u32>,
    pub models: Vec<Model>,
    pub analogue: String,
}

/// A mechanic the sim lacks, named once so every row that waits on it
/// counts toward the build list.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mechanic {
    pub key: String,
    pub summary: String,
    #[serde(default)]
    pub cites: Vec<String>,
}

/// A verb the registry does not hold, proposed for the catalog (D38 lifts
/// the stop rule on new processes).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedProcess {
    pub key: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trait {
    pub key: String,
    pub summary: String,
    pub category: Category,
    pub process: String,
    pub shape: String,
    pub kingdoms: Vec<String>,
    pub models: Vec<Model>,
    pub sites: Vec<Site>,
    pub admits: Vec<String>,
    pub emits: Vec<String>,
    pub cost_mg: u32,
    /// The kernels that read this trait (the memo's first test counts them).
    pub reads: Vec<String>,
    pub analogue: String,
    /// The existing rules it runs through, when it needs no new mechanic.
    pub via: Vec<String>,
    /// The mechanic it waits on, when it does.
    pub needs: Option<String>,
    /// Food categories (`FLOWS` words) this trait lets its body take, the
    /// enzyme pattern of the brief (section 10, item 3).
    pub unlocks: Vec<String>,
    pub strengths: Vec<Strength>,
    pub status: Status,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effect {
    pub key: String,
    pub summary: String,
    pub site: Site,
    pub class: Class,
    pub models: Vec<Model>,
    pub accepts: Vec<Category>,
    pub processes: Vec<String>,
    pub only: Vec<String>,
    pub kingdoms: Vec<String>,
    pub admits: Vec<String>,
    pub emits: Vec<String>,
    pub cost_mg: u32,
    pub analogue: String,
    pub via: Vec<String>,
    pub needs: Option<String>,
    /// Who it applies to; all gates must hold.
    pub gates: Vec<Gate>,
    /// For a lasting effect; `None` is instant.
    pub timing: Option<Timing>,
    pub status: Status,
}

/// One part of the catalog: what a kingdom's tissue could be, given its
/// traits and what it does at each transfer site.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Part {
    pub key: String,
    pub kingdom: String,
    /// `mesocosm:<function>` (in the registry) or `proposed:<verb>`.
    pub process: String,
    /// The shape it grows on, `part-shape:*`.
    pub shape: String,
    pub traits: Vec<String>,
    /// Effect keys at each site; a site may be absent.
    pub effects: BTreeMap<Site, Vec<String>>,
    /// What it admits and emits, in `FLOWS`' words.
    pub admits: Vec<String>,
    pub emits: Vec<String>,
    /// Matter cost in milligrams at the reference body (`BodyRules`).
    pub cost_mg: u32,
    pub models: Vec<Model>,
    pub analogue: String,
    pub status: Status,
    /// Every mechanic this part waits on, sorted (the status names the
    /// first); empty when expressible.
    pub waits_on: Vec<String>,
    pub origin: Origin,
}

impl Part {
    /// The memo's "two names for one thing" fingerprint (section 2, test 4).
    pub fn fingerprint(&self) -> String {
        let mut traits = self.traits.clone();
        traits.sort();
        let effects: Vec<String> = self
            .effects
            .iter()
            .map(|(site, keys)| {
                let mut keys = keys.clone();
                keys.sort();
                format!("{site:?}={}", keys.join(","))
            })
            .collect();
        let mut admits = self.admits.clone();
        admits.sort();
        let mut emits = self.emits.clone();
        emits.sort();
        format!(
            "{}|{}|{}|{}|{}>{}",
            self.kingdom,
            self.process,
            traits.join(","),
            effects.join(";"),
            admits.join(","),
            emits.join(",")
        )
    }

    pub fn effect_keys(&self) -> impl Iterator<Item = &String> {
        self.effects.values().flatten()
    }
}
