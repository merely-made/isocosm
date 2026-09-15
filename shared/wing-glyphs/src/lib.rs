// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reusable correspondence and acquisition data. Effect IDs are references,
//! not executable spells. Product rules own grants and their consequences.
mod canon;
pub mod divine;
mod journey;

pub use canon::{
    Canon, CanonLimits, CanonSpec, CorrespondenceMove, EffectId, GlyphDefinition, GlyphId,
    ModifierId, VariantDefinition,
};
pub use journey::{
    Acquisition, Eligibility, GrantOutcome, GrantRecord, Journey, JourneyLimits, JourneySnapshot,
    JourneyTransition, MotifGroup, Provenance, ProvenanceKind, VariantPolicy,
};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MAX_GLYPHS: usize = 4096;
pub const DEFAULT_MAX_VARIANTS: usize = 4096;
pub const DEFAULT_MAX_GRANTS: usize = 16384;
pub const DEFAULT_MAX_TRANSITIONS: usize = 65536;
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
