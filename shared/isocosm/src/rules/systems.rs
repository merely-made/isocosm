// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Systems as world data (rulings 465, 477, 489 to 491, 560 to 565 and 574
//! to 590): functions in roles, sources, stores, gates and effects, read
//! from a body's tree. A role names catalogue functions, every living part,
//! or the parts a bite lands on; a native acts on the systems that name its
//! function, every living part naming none (582). What a route carries is a
//! world rule: so much a cell a tick, a conduct cell by its part's
//! cross-section over the reference segment's.

use crate::schema::Key;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What fills a role.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Fill {
    /// The parts expressing a catalogue function.
    Function(Key),
    /// Every living part (489).
    Living,
    /// The parts a bite lands on (588).
    Bitten,
}

/// A role in a system (465).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    Source,
    Store,
    Gate,
    Effect,
}

impl Role {
    pub const ALL: [Role; 4] = [Role::Source, Role::Store, Role::Gate, Role::Effect];
}

/// A system: what fills each of its roles. One with no source and no
/// effect routes nothing, as the integument (489).
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct System {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub sources: BTreeSet<Fill>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub stores: BTreeSet<Fill>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gates: BTreeSet<Fill>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub effects: BTreeSet<Fill>,
}

impl System {
    pub fn role(&self, r: Role) -> &BTreeSet<Fill> {
        match r {
            Role::Source => &self.sources,
            Role::Store => &self.stores,
            Role::Gate => &self.gates,
            Role::Effect => &self.effects,
        }
    }

    pub fn role_mut(&mut self, r: Role) -> &mut BTreeSet<Fill> {
        match r {
            Role::Source => &mut self.sources,
            Role::Store => &mut self.stores,
            Role::Gate => &mut self.gates,
            Role::Effect => &mut self.effects,
        }
    }

    /// Whether it names `function` in `role` (582's reading).
    pub fn routes(&self, function: &str, role: Role) -> bool {
        self.role(role)
            .iter()
            .any(|f| matches!(f, Fill::Function(k) if k == function))
    }

    /// The catalogue functions it names, in any role.
    pub fn functions(&self) -> BTreeSet<&Key> {
        Role::ALL
            .iter()
            .flat_map(|r| self.role(*r))
            .filter_map(|f| match f {
                Fill::Function(k) => Some(k),
                _ => None,
            })
            .collect()
    }
}

/// Ruling 489's ten systems, which a world adopts by naming them.
pub fn default_systems() -> BTreeMap<Key, System> {
    let f = |names: &[&str]| -> BTreeSet<Fill> {
        names
            .iter()
            .map(|n| match *n {
                "living" => Fill::Living,
                "bitten" => Fill::Bitten,
                n => Fill::Function(format!("function:{n}")),
            })
            .collect()
    };
    let system = |sources: &[&str], stores: &[&str], gates: &[&str], effects: &[&str]| System {
        sources: f(sources),
        stores: f(stores),
        gates: f(gates),
        effects: f(effects),
    };
    [
        (
            "digestive",
            system(&["intake"], &["store"], &["gate"], &["living"]),
        ),
        (
            "photosynthetic",
            system(&["fix"], &["store"], &["gate"], &["living"]),
        ),
        (
            "muscular",
            system(&["living"], &["store"], &["gate"], &["contract"]),
        ),
        ("nervous", system(&["sense"], &[], &["gate"], &["contract"])),
        (
            "glandular",
            system(&["secrete"], &["store"], &["gate"], &["bitten"]),
        ),
        (
            "reproductive",
            system(&["living"], &["store"], &["gate"], &["reproduce"]),
        ),
        (
            "circulatory",
            system(&["circulate"], &["store"], &["gate"], &["living"]),
        ),
        (
            "respiratory",
            system(&["respire"], &[], &["gate"], &["contract"]),
        ),
        (
            "excretory",
            system(&["living"], &[], &["gate"], &["excrete"]),
        ),
        ("integumentary", System::default()),
    ]
    .into_iter()
    .map(|(name, s)| (format!("system:{name}"), s))
    .collect()
}

/// What a route carries (rulings 560, 561, 564 and 565): a cell's capacity
/// a tick, in milligrams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carriage {
    pub per_cell: u64,
}
