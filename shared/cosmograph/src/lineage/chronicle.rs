// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a creature is when it is not a body: its lineage, what each part was
//! made of, and the deeds other games appended (the record family, 673 and
//! 770). Additive facts, opaque preservation, deferred interpretation: a
//! deed this game cannot read is kept byte for byte, and re-entry reads the
//! deeds it can. Moved from legacy Mesocosm with the world move; re-entry
//! onto a native body is the after-pass's.

use std::ops::{Deref, DerefMut};

use isometer_core::{WireError, frame, unframe};
use serde::{Deserialize, Serialize};
pub use wing_formats::{Deed, PartOrigin};

use super::body::LineageBody;

pub const CHRONICLE_SCHEMA: &str = wing_formats::CHRONICLE_SCHEMA;
pub const CHRONICLE_MAGIC: [u8; 8] = wing_formats::CHRONICLE_MAGIC;
pub const CHRONICLE_VERSION: u16 = wing_formats::CHRONICLE_VERSION;

/// The verb this game appends when another game's play cost a part.
pub const LOST_PART: &str = "lost-part";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Chronicle(pub wing_formats::Chronicle);

impl Deref for Chronicle {
    type Target = wing_formats::Chronicle;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Chronicle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// What this game reads a deed as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consequence {
    LostPart { part: u32 },
    Unread,
}

impl Chronicle {
    pub fn of(body: &LineageBody) -> Self {
        Self(wing_formats::Chronicle {
            species: body.species().0,
            parts: body
                .parts
                .iter()
                .map(|part| body.part_origin(part.id))
                .collect(),
            deeds: Vec::new(),
        })
    }

    /// The only way to change a chronicle.
    pub fn append(&mut self, deed: Deed) {
        self.deeds.push(deed);
    }

    pub fn incorporated_parts(&self) -> usize {
        let parts = self.parts.iter();
        parts.filter(|part| part.is_incorporated()).count()
    }

    pub fn read(&self) -> impl Iterator<Item = (&Deed, Consequence)> {
        self.deeds.iter().map(|deed| (deed, interpret(deed)))
    }

    pub fn unread(&self) -> impl Iterator<Item = &Deed> {
        let deeds = self.deeds.iter();
        deeds.filter(|deed| interpret(deed) == Consequence::Unread)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, WireError> {
        frame(CHRONICLE_MAGIC, CHRONICLE_VERSION, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        let chronicle: Self = unframe(CHRONICLE_MAGIC, CHRONICLE_VERSION, bytes)?;
        wing_formats::validate_chronicle(&chronicle.0)?;
        Ok(chronicle)
    }
}

/// A malformed detail under our own verb stays unread rather than guessed.
fn interpret(deed: &Deed) -> Consequence {
    if deed.verb != LOST_PART {
        return Consequence::Unread;
    }
    match <[u8; 4]>::try_from(deed.detail.as_slice()) {
        Ok(bytes) => Consequence::LostPart {
            part: u32::from_le_bytes(bytes),
        },
        Err(_) => Consequence::Unread,
    }
}
