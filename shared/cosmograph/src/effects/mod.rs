// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The effects family (families plan, family 6): what a glyph returns as,
//! which glyphs a body embodies, a body's membership for wing-functions, and
//! a subject's glyph journal over accepted acts. Glyphs' rungs stay parked
//! (rulings 279 and 673): this is the moved code, unchanged in behaviour,
//! and nothing here grants, executes or draws on its own.

pub mod embodiment;
pub mod functions;
pub mod pack;
pub mod reading;

pub use embodiment::{bearing_parts, embodied, expressed_traits};
pub use pack::{
    Amount, Bearer, DEFAULT_EFFECT, EffectPackTable, MarkForm, MarkRequest, PackRule, Refusal,
};
pub use reading::{GlyphGrantOutcome, Journal};

use serde::{Deserialize, Serialize};

/// A mark's stroke: the shape a host draws a returned glyph with.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Glyph {
    Quotes,
    Slashes,
    Backticks,
}

impl Glyph {
    pub const ALL: &'static [Self] = &[Self::Quotes, Self::Slashes, Self::Backticks];

    pub fn label(self) -> &'static str {
        match self {
            Self::Quotes => "quotes",
            Self::Slashes => "slashes",
            Self::Backticks => "backticks",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Self::Quotes => "\"",
            Self::Slashes => "/",
            Self::Backticks => "`",
        }
    }
}

#[cfg(test)]
mod tests;
