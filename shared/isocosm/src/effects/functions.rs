// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's current membership for wing-functions' evaluator.
//!
//! The caller owns subject identity and durable network state. Rebuild this
//! reading after each accepted body change; never save it as another body.
//! Eligibility is membership, not a claim that every living part can perform
//! every process: construction admission assigns the functions.

use crate::schema::{Entity, PartId};
use std::collections::BTreeSet;
pub use wing_functions::PartRef;
use wing_functions::generation::{BodyTract, GenerationBatch, GeneratorSettings, generate};

/// `parts` as `subject`'s part references.
pub fn refs(subject: u64, parts: impl IntoIterator<Item = PartId>) -> BTreeSet<PartRef> {
    parts
        .into_iter()
        .map(|part| PartRef {
            subject,
            part: part.0,
        })
        .collect()
}

/// `e`'s living parts as `subject`'s part references.
pub fn live_parts(subject: u64, e: &Entity) -> BTreeSet<PartRef> {
    refs(subject, e.living().map(|(id, _)| id))
}

/// Bind a generated network to tracts on `live` parts; refuses the first
/// tract on a part not among them. Tract roles come from an authored
/// construction rule, not from part order.
pub fn generate_among(
    live: &BTreeSet<PartRef>,
    seed: u64,
    settings: &GeneratorSettings,
    tracts: &[BodyTract],
) -> Result<GenerationBatch, PartRef> {
    if let Some(tract) = tracts.iter().find(|tract| !live.contains(&tract.part)) {
        return Err(tract.part);
    }
    Ok(generate(seed, settings, tracts))
}

/// [`generate_among`] on `e`'s living parts.
pub fn generate_for_body(
    subject: u64,
    e: &Entity,
    seed: u64,
    settings: &GeneratorSettings,
    tracts: &[BodyTract],
) -> Result<GenerationBatch, PartRef> {
    generate_among(&live_parts(subject, e), seed, settings, tracts)
}
