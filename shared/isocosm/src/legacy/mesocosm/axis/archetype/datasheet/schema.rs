// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sheets' own shapes, as serde reads them. Everything is
//! `deny_unknown_fields`, so a misspelt key is refused rather than read as its
//! default.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::legacy::mesocosm::axis::{Anchor, Appendage, ChainFacing};
use crate::legacy::mesocosm::plan::{Facing, Role};

/// Livery's header, which every sheet carries.
macro_rules! header_fields {
    ($sheet:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        pub(super) struct $sheet {
            pub(super) schema: u32,
            pub(super) owner: String,
            pub(super) consumer: String,
            pub(super) status: String,
            pub(super) sources: BTreeMap<String, Source>,
            $($(#[$attr])* pub(super) $field: $ty),*
        }

        impl $sheet {
            pub(super) fn check(&self, sheet: &str) -> Result<(), String> {
                if self.schema != 1 {
                    return Err(format!("{sheet}: schema {} is not 1", self.schema));
                }
                let said = |text: &String| !text.trim().is_empty();
                let sourced = !self.sources.is_empty()
                    && self.sources.values().all(|source| said(&source.doc) && said(&source.note));
                if !(said(&self.owner) && said(&self.consumer) && said(&self.status) && sourced) {
                    return Err(format!("{sheet}: owner, consumer, status and sources are required"));
                }
                Ok(())
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub(super) doc: String,
    pub(super) note: String,
}

header_fields!(SetSheet {
    #[serde(default)]
    palettes: BTreeMap<String, PaletteEntry>,
    #[serde(default)]
    chain: BTreeMap<String, Vec<StepEntry>>,
    #[serde(default)]
    body: BTreeMap<String, BodyEntry>,
});

header_fields!(FoundingsSheet {
    default: String,
    founding: BTreeMap<String, FoundingEntry>,
});

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FoundingEntry {
    pub(super) palette: String,
    #[serde(default)]
    pub(super) producer: Vec<String>,
    #[serde(default)]
    pub(super) consumer: Vec<String>,
    #[serde(default)]
    pub(super) decomposer: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PaletteEntry {
    pub(super) extends: Option<String>,
    #[serde(default)]
    pub(super) mass: Vec<ShapeEntry>,
    #[serde(default)]
    pub(super) limb: Vec<ShapeEntry>,
    #[serde(default)]
    pub(super) plate: Vec<ShapeEntry>,
    #[serde(default)]
    pub(super) sensor: Vec<ShapeEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShapeEntry {
    pub(super) name: String,
    pub(super) tag: u8,
    pub(super) half_extent: [i32; 3],
    pub(super) replaces: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StepEntry {
    pub(super) role: RoleWord,
    pub(super) shape: String,
    pub(super) facing: ChainFacingWord,
    #[serde(default)]
    pub(super) distal: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BodyEntry {
    pub(super) variance: u8,
    pub(super) tagma: Vec<TagmaEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TagmaEntry {
    pub(super) segments: u8,
    pub(super) segment: Option<String>,
    pub(super) appendage: Option<AppendageWord>,
    pub(super) shape: Option<String>,
    pub(super) mouth: Option<MouthWord>,
    pub(super) worn: Option<WornWord>,
    pub(super) per_segment: Option<u8>,
    pub(super) layout: Option<LayoutEntry>,
    pub(super) chain: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LayoutEntry {
    pub(super) parent: Option<u8>,
    pub(super) anchor: AnchorWord,
    pub(super) facing: FacingWord,
    pub(super) variance: Option<u8>,
}

/// A lower-case word in a sheet for each variant of a code enum.
macro_rules! words {
    ($word:ident => $code:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub(super) enum $word { $($variant),* }

        impl From<$word> for $code {
            fn from(word: $word) -> Self {
                match word { $($word::$variant => $code::$variant),* }
            }
        }
    };
}

words!(AppendageWord => Appendage { None, Limb, Feeler, Plate, Mouth, Vane });
words!(RoleWord => Role { Mass, Limb, Plate, Sensor });
words!(AnchorWord => Anchor { Base, Middle, Tip });
words!(FacingWord => Facing { Front, Back, Left, Right, Above, Below });
words!(ChainFacingWord => ChainFacing { Outward, Inward, Above, Below, Front, Back });

/// Which bank a mouth is drawn from: the mass bank as bulk, the limb bank as
/// an actuator.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum MouthWord {
    Bulk,
    Jaw,
}

/// Whether a plate is held out from the body or worn against it.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WornWord {
    Held,
    Covering,
}
