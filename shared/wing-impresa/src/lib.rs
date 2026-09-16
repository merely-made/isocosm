// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The impresa: what a pointable thing has come to be associated with, and
//! what has lapsed, over time. A glyph, a critter, a borg, a character, a
//! faction, a place, an item and a lot of matter all bear one, so this crate
//! knows none of those nouns.
//!
//! **The record type and its validation only.** A product owns storage in its
//! own accepted history and derives its readings as projections over what it
//! stored. The accepted record is fact; an observer's held associations are
//! belief and a promoted association is legend, and neither reading lives
//! here — a second authority in the kernel is exactly what the organ is meant
//! to prevent.
mod impresa;
mod kinds;

pub use impresa::{Impresa, ImpresaLimits, ImpresaSpec, ObjectId, Record, Stance, SubjectId};
pub use kinds::{KindSet, KindSetLimits, KindSetSpec, SEEDED_KINDS};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MAX_ENTRIES: usize = 65536;
pub const DEFAULT_MAX_KINDS: usize = 256;
/// A cause is the evidence string of the accepted event a record cites, not
/// the event itself: wide enough to carry a receipt, narrow enough that an
/// impresa cannot quietly become the product's event log.
pub const DEFAULT_MAX_CAUSE_BYTES: usize = 512;
pub const DEFAULT_MAX_JSON_BYTES: usize = 8 * 1024 * 1024;

fn identifier(value: &str) -> Result<(), String> {
    let Some((namespace, local)) = value.split_once(':') else {
        return Err(format!("ID requires namespace:local: {value:?}"));
    };
    let valid = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-./".contains(&b))
    };
    if value.len() > 256 || !valid(namespace) || !valid(local) {
        return Err(format!("invalid namespaced ID: {value:?}"));
    }
    Ok(())
}

fn bounded_text(value: &str, label: &str, limit: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > limit {
        return Err(format!("{label} must contain 1..{limit} UTF-8 bytes"));
    }
    Ok(())
}
