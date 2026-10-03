// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Part shapes and the function catalogue (rulings 276, 278, 338 to 341, 461,
//! 466 and 492). A part expresses functions; each function in the world's
//! catalogue names the shapes it fits best and how a part comes to express
//! it, and no shape gates a function (492). A process that needs a function
//! binds the actor's lowest-numbered live part expressing it.

use super::Rules;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a part comes to express a function (ruling 341).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Seeding {
    /// Growing a part expresses it, by default on the shapes it fits.
    Grown,
    /// Only a development places it; nothing grows one.
    Acquired,
}

/// One function of the world's catalogue (ruling 338). Its identity is the
/// set of shapes, in no order and without repeats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    /// The shapes it fits best (rulings 466 and 492), which the generator
    /// draws by default; never empty, and gating nothing.
    pub shapes: BTreeSet<Key>,
    pub seeding: Seeding,
}

/// Ruling 276's eight shapes, which a world adopts by naming them. Part
/// shapes keep their own namespace (ruling 360); `shape:` is the world's.
pub const SHAPES: [&str; 8] = [
    "part-shape:lump",
    "part-shape:rod",
    "part-shape:sheet",
    "part-shape:point",
    "part-shape:tube",
    "part-shape:branch",
    "part-shape:shell",
    "part-shape:joint",
];

/// The eight shapes as a world's rules hold them.
pub fn default_shapes() -> BTreeSet<Key> {
    SHAPES.iter().map(|s| s.to_string()).collect()
}

/// Ruling 466's fifteen functions and their fits; secrete alone is acquired.
pub fn default_functions() -> BTreeMap<Key, Function> {
    let function = |shapes: &[&str], seeding| Function {
        shapes: shapes.iter().map(|s| format!("part-shape:{s}")).collect(),
        seeding,
    };
    let grown = |shapes: &[&str]| function(shapes, Seeding::Grown);
    [
        ("contract", grown(&["rod"])),
        ("intake", grown(&["lump", "tube"])),
        ("sense", grown(&["point"])),
        ("fix", grown(&["sheet"])),
        ("secrete", function(&["sheet"], Seeding::Acquired)),
        ("support", grown(&["rod", "shell", "joint", "branch"])),
        ("conduct", grown(&["tube", "branch"])),
        ("gate", grown(&["joint", "tube"])),
        ("store", grown(&["lump"])),
        ("circulate", grown(&["tube", "lump"])),
        ("respire", grown(&["sheet", "branch"])),
        ("excrete", grown(&["tube"])),
        ("reproduce", grown(&["lump"])),
        ("grip", grown(&["rod", "branch", "joint"])),
        ("adhesion", grown(&["sheet", "point"])),
    ]
    .into_iter()
    .map(|(name, f)| (format!("function:{name}"), f))
    .collect()
}

/// Mesocosm's reference body (ruling 460's reading): an adult part weighs
/// the reference mass for each reference segment of voxels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyRules {
    pub reference_mass_mg: u64,
    pub reference_segment_voxels: u64,
}

impl Default for BodyRules {
    fn default() -> Self {
        Self {
            reference_mass_mg: 100,
            reference_segment_voxels: 125,
        }
    }
}

impl Rules {
    /// Whether `function` fits a part of `shape` best (ruling 492).
    pub fn fits(&self, shape: &str, function: &str) -> bool {
        self.functions
            .get(function)
            .is_some_and(|f| f.shapes.contains(shape))
    }

    /// The functions a part of `shape` grows by default: until bodies move,
    /// what the founding generator gives it (ruling 273, R4).
    pub fn grown(&self, shape: &str) -> BTreeSet<Key> {
        let grown = self
            .functions
            .iter()
            .filter(|(_, f)| f.seeding == Seeding::Grown && f.shapes.contains(shape));
        grown.map(|(k, _)| k.clone()).collect()
    }

    /// The world's reference body, Mesocosm's where it states none.
    pub fn body(&self) -> BodyRules {
        self.body.unwrap_or_default()
    }
}

/// A body's lowest-numbered live part expressing `function`: the part a
/// process needing it binds in its actor (ruling 338).
pub(crate) fn expressing(e: &Entity, function: &str) -> Option<Id> {
    let mut live = e.parts.iter().filter(|(_, p)| !p.severed);
    live.find(|(_, p)| p.functions.contains(function))
        .map(|(id, _)| *id)
}
