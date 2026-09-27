// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Part shapes and the function catalogue (rulings 276, 278 and 338 to 341).
//! A part has one shape and expresses functions; each function in the
//! world's catalogue names the shapes that admit it and how a part comes to
//! express it. A process that needs a function binds the actor's
//! lowest-numbered live part expressing it.

use super::Rules;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a part comes to express a function (ruling 341).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Seeding {
    /// Growing a part of an admitting shape expresses it.
    Grown,
    /// Only a development places it; nothing grows one.
    Acquired,
}

/// One function of the world's catalogue (ruling 338). Its identity is the
/// set of shapes, in no order and without repeats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    /// The shapes whose parts may express it; never empty.
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

/// Ruling 339's five functions in use: rods contract, lumps take in, points
/// sense, sheets fix, and sheets secrete once acquired.
pub fn default_functions() -> BTreeMap<Key, Function> {
    let function = |shape: &str, seeding| Function {
        shapes: BTreeSet::from([format!("part-shape:{shape}")]),
        seeding,
    };
    BTreeMap::from([
        ("function:contract".into(), function("rod", Seeding::Grown)),
        ("function:intake".into(), function("lump", Seeding::Grown)),
        ("function:sense".into(), function("point", Seeding::Grown)),
        ("function:fix".into(), function("sheet", Seeding::Grown)),
        (
            "function:secrete".into(),
            function("sheet", Seeding::Acquired),
        ),
    ])
}

impl Rules {
    /// Whether a part of `shape` may express `function`.
    pub fn admits(&self, shape: &str, function: &str) -> bool {
        self.functions
            .get(function)
            .is_some_and(|f| f.shapes.contains(shape))
    }

    /// The functions a part of `shape` expresses by growing: until bodies
    /// move, what the founding generator gives it (ruling 273, R4).
    pub fn grown(&self, shape: &str) -> BTreeSet<Key> {
        let grown = self
            .functions
            .iter()
            .filter(|(_, f)| f.seeding == Seeding::Grown && f.shapes.contains(shape));
        grown.map(|(k, _)| k.clone()).collect()
    }
}

/// A body's lowest-numbered live part expressing `function`: the part a
/// process needing it binds in its actor (ruling 338).
pub(crate) fn expressing(e: &Entity, function: &str) -> Option<Id> {
    let mut live = e.parts.iter().filter(|(_, p)| !p.severed);
    live.find(|(_, p)| p.functions.contains(function))
        .map(|(id, _)| *id)
}
