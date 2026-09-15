// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reusable correspondence and acquisition data. Effect IDs are references,
//! not executable spells. Product rules own grants and their consequences.
mod canon;
pub mod divine;
mod expression;
mod journey;
mod pack;

pub use canon::{
    Canon, CanonLimits, CanonSpec, CorrespondenceMove, EffectId, GlyphDefinition, GlyphId,
    ModifierId, VariantDefinition,
};
pub use expression::{ExpressionLimits, ExpressionSpec, ExpressionTable, GlyphExpression, TraitId};
pub use journey::{
    Acquisition, Eligibility, GrantOutcome, GrantRecord, Journey, JourneyLimits, JourneySnapshot,
    JourneyTransition, MotifGroup, Provenance, ProvenanceKind, VariantPolicy,
};
pub use pack::{
    BehaviourKind, CostShape, CostUnit, EffectDeclaration, EffectPack, EffectPackSpec, PackLimits,
    ReceiverClass,
};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MAX_GLYPHS: usize = 4096;
pub const DEFAULT_MAX_VARIANTS: usize = 4096;
pub const DEFAULT_MAX_DECLARATIONS: usize = 4096;
pub const DEFAULT_MAX_EXPRESSIONS: usize = 4096;
/// A glyph is expressed by several traits, never by a catalogue of them: the
/// bound is on the authored claim, not on the trait set itself.
pub const DEFAULT_MAX_TRAITS_PER_GLYPH: usize = 64;
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
