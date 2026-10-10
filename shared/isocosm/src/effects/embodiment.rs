// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which glyphs a body embodies: the sim's half of the expression table.
//!
//! A glyph is embodied when a living part bears a trait the table says
//! expresses it; any one trait is enough (the table's rule is disjunction).
//! On a native body a trait is a catalogue function a living part expresses,
//! named by its key (`function:intake`), the function a legacy tract's
//! process lowers onto (wing ruling 750). Pure readings: nothing here takes
//! `&mut`, so drawing an embodied mark cannot express one.

use crate::schema::{Entity, PartId};
use std::collections::BTreeSet;
use wing_glyphs::{ExpressionTable, GlyphId};

/// Every glyph `e` currently embodies.
pub fn embodied(e: &Entity, table: &ExpressionTable) -> BTreeSet<GlyphId> {
    embodied_by(traits_of(e), table)
}

/// Every trait `e`'s living parts express, as the ids the table keys on.
pub fn expressed_traits(e: &Entity) -> BTreeSet<String> {
    traits_by(traits_of(e))
}

/// The living parts whose traits express `glyph`, in part order: what a
/// mark sits on. Empty when the glyph is not embodied.
pub fn bearing_parts(e: &Entity, table: &ExpressionTable, glyph: &str) -> Vec<PartId> {
    bearing_by(traits_of(e), table, glyph)
}

/// Each living part of `e` with the functions it expresses, in part order.
fn traits_of(e: &Entity) -> impl Iterator<Item = (PartId, Vec<String>)> + '_ {
    e.living()
        .map(|(id, p)| (id, p.functions.iter().cloned().collect()))
}

/// [`embodied`] over any body's living parts and their traits, so a body
/// kept elsewhere (the legacy phenotype until its world moves, 755) reads
/// through the same rule.
pub fn embodied_by<I, T>(parts: I, table: &ExpressionTable) -> BTreeSet<GlyphId>
where
    I: IntoIterator<Item = (PartId, T)>,
    T: IntoIterator<Item = String>,
{
    let mut found = BTreeSet::new();
    for id in traits_by(parts) {
        found.extend(table.glyphs_of(&id).iter().cloned());
    }
    found
}

/// The union of the traits over `parts`.
pub fn traits_by<I, T>(parts: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = (PartId, T)>,
    T: IntoIterator<Item = String>,
{
    parts.into_iter().flat_map(|(_, traits)| traits).collect()
}

/// [`bearing_parts`] over any body's living parts and their traits.
pub fn bearing_by<I, T>(parts: I, table: &ExpressionTable, glyph: &str) -> Vec<PartId>
where
    I: IntoIterator<Item = (PartId, T)>,
    T: IntoIterator<Item = String>,
{
    let bears = |t: String| table.glyphs_of(&t).iter().any(|g| g == glyph);
    parts
        .into_iter()
        .filter_map(|(part, traits)| traits.into_iter().any(bears).then_some(part))
        .collect()
}
