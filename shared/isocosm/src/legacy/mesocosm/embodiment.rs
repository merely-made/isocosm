// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which glyphs a legacy body embodies, read through the native rule
//! (`isocosm::effects::embodiment`). This adapter lowers a legacy phenotype
//! to its living parts' traits and goes with the world move (wing ruling
//! 755), the legacy organism being that world's.
//!
//! **A trait is an expressed tract**, `(PartId, ProcessRef)`, resolved
//! through the world's [`Registry`] to a qualified process id. Embodiment
//! reads the phenotype, never the document: a development that moves tissue
//! makes the two disagree by design, and only the phenotype's is the record
//! of expression. A tract whose definition the registry does not hold
//! contributes nothing, and is never substituted for a similar one.

use std::collections::BTreeSet;

use wing_glyphs::{ExpressionTable, GlyphId};

use crate::effects::embodiment::{bearing_by, embodied_by, traits_by};
use crate::legacy::mesocosm::body::PartId;
use crate::legacy::mesocosm::phenotype::BodyPhenotype;
use crate::process::Registry;

/// Each living part with the qualified ids of its resolvable tracts.
fn traits<'a>(
    phenotype: &'a BodyPhenotype,
    registry: &'a Registry,
) -> impl Iterator<Item = (PartId, Vec<String>)> + 'a {
    phenotype.allocations().map(move |(part, mosaic)| {
        let resolved = mosaic.tracts().iter();
        let ids = resolved.filter_map(|t| registry.resolve(t.process));
        (part, ids.map(|def| def.id.qualified()).collect())
    })
}

/// Every glyph this body currently embodies.
pub fn embodied(
    phenotype: &BodyPhenotype,
    registry: &Registry,
    table: &ExpressionTable,
) -> BTreeSet<GlyphId> {
    embodied_by(traits(phenotype, registry), table)
}

/// Every trait this body expresses, as the qualified ids the table keys on.
pub fn expressed_traits(phenotype: &BodyPhenotype, registry: &Registry) -> BTreeSet<String> {
    traits_by(traits(phenotype, registry))
}

/// The living parts whose tracts express this glyph, in part order.
pub fn bearing_parts(
    phenotype: &BodyPhenotype,
    registry: &Registry,
    table: &ExpressionTable,
    glyph: &str,
) -> Vec<PartId> {
    bearing_by(traits(phenotype, registry), table, glyph)
}

#[cfg(test)]
#[path = "embodiment/tests.rs"]
mod tests;
