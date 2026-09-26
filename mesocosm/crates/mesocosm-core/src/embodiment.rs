// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which glyphs a body embodies: the Mesocosm half of the expression table.
//!
//! **A trait is an expressed tract**, `(PartId, ProcessRef)`. Nothing else in
//! the tree is both borne by a part and rule-bearing, and Mark's ruling — a
//! trait or part bears the glyph — lands exactly on that pair.
//!
//! # Pure reading
//!
//! The set of glyphs a body embodies is a function of its **living, attached
//! parts and their expressed tracts**, and of nothing else. No stored field,
//! no event, no journey, no `World`. Everything here takes `&` and returns a
//! new value, so a host that draws an embodied mark cannot express one.
//!
//! # Embodiment reads the phenotype, never the document
//!
//! [`BodyDocument::processes`](crate::process) answers from geometry;
//! [`BodyPhenotype::allocations`] answers from expressed tracts. A development
//! that moves tissue makes the two disagree **by design**, and only the
//! phenotype's is the record of expression. That is why every function here
//! takes a [`BodyPhenotype`], and why the tests assert the disagreement.
//!
//! # What lives on each side of the boundary
//!
//! `wing-glyphs` owns the table — glyph to opaque `namespace:local` trait id,
//! per canon revision — and knows nothing about bodies. This module owns the
//! resolution: a stored [`ProcessRef`] through the world's [`Registry`] to a
//! [`ProcessId`](crate::process::ProcessId), qualified, and looked up. Only
//! Mesocosm knows what a part is, so only this half can be here.

use std::collections::BTreeSet;

use wing_glyphs::{ExpressionTable, GlyphId};

use crate::body::PartId;
use crate::phenotype::BodyPhenotype;
use crate::process::Registry;

/// Every glyph this body currently embodies.
///
/// A glyph is embodied when a **living, attached part bears a tract whose
/// process expresses it**. Several traits may express one glyph, and any one
/// of them is enough: the table's rule is disjunction.
///
/// A tract whose definition this world's registry does not hold contributes
/// nothing. `None` from [`Registry::resolve`] is a real answer and is never
/// substituted for a similar local definition.
pub fn embodied(
    phenotype: &BodyPhenotype,
    registry: &Registry,
    table: &ExpressionTable,
) -> BTreeSet<GlyphId> {
    let mut found = BTreeSet::new();
    for id in expressed_traits(phenotype, registry) {
        found.extend(table.glyphs_of(&id).iter().cloned());
    }
    found
}

/// Every trait this body expresses, as the qualified ids the table keys on.
///
/// The resolution half on its own, so a panel can say which traits a body
/// bears without going through a glyph.
pub fn expressed_traits(phenotype: &BodyPhenotype, registry: &Registry) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for (_, mosaic) in phenotype.allocations() {
        for tract in mosaic.tracts() {
            if let Some(def) = registry.resolve(tract.process) {
                found.insert(def.id.qualified());
            }
        }
    }
    found
}

/// The living parts whose tracts express this glyph, in part order.
///
/// **What bears it**, which is what a mark needs to sit on the face of the
/// part that expresses it rather than at a body centroid. Empty when the
/// glyph is not embodied, which is the same answer as "there is no face to
/// anchor to".
pub fn bearing_parts(
    phenotype: &BodyPhenotype,
    registry: &Registry,
    table: &ExpressionTable,
    glyph: &str,
) -> Vec<PartId> {
    phenotype
        .allocations()
        .filter(|(_, mosaic)| {
            mosaic.tracts().iter().any(|tract| {
                registry.resolve(tract.process).is_some_and(|def| {
                    table
                        .glyphs_of(&def.id.qualified())
                        .iter()
                        .any(|g| g == glyph)
                })
            })
        })
        .map(|(part, _)| part)
        .collect()
}

#[cfg(test)]
#[path = "embodiment/tests.rs"]
mod tests;
