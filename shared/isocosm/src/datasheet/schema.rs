// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sheets' own shapes, as serde reads them. Everything is
//! `deny_unknown_fields`, so a misspelt key is refused rather than read as its
//! default.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Livery's header, which every sheet carries.
macro_rules! header_fields {
    ($sheet:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $sheet {
            pub schema: u32,
            pub owner: String,
            pub consumer: String,
            pub status: String,
            pub sources: BTreeMap<String, Source>,
            $($(#[$attr])* pub $field: $ty),*
        }

        impl $sheet {
            pub fn check(&self, sheet: &str) -> Result<(), String> {
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
pub struct Source {
    pub doc: String,
    pub note: String,
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
pub struct FoundingEntry {
    pub palette: String,
    #[serde(default)]
    pub producer: Vec<String>,
    #[serde(default)]
    pub consumer: Vec<String>,
    #[serde(default)]
    pub decomposer: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteEntry {
    pub extends: Option<String>,
    #[serde(default)]
    pub mass: Vec<ShapeEntry>,
    #[serde(default)]
    pub limb: Vec<ShapeEntry>,
    #[serde(default)]
    pub plate: Vec<ShapeEntry>,
    #[serde(default)]
    pub sensor: Vec<ShapeEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeEntry {
    pub name: String,
    pub tag: u8,
    pub half_extent: [i32; 3],
    pub replaces: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepEntry {
    pub role: RoleWord,
    pub shape: String,
    pub facing: ChainFacingWord,
    #[serde(default)]
    pub distal: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyEntry {
    pub variance: u8,
    pub tagma: Vec<TagmaEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagmaEntry {
    pub segments: u8,
    pub segment: Option<String>,
    pub appendage: Option<AppendageWord>,
    pub shape: Option<String>,
    pub mouth: Option<MouthWord>,
    pub worn: Option<WornWord>,
    pub per_segment: Option<u8>,
    pub layout: Option<LayoutEntry>,
    pub chain: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutEntry {
    pub parent: Option<u8>,
    pub anchor: AnchorWord,
    pub facing: FacingWord,
    pub variance: Option<u8>,
}

/// A lower-case word in a sheet for each variant of a code enum.
macro_rules! words {
    ($word:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $word { $($variant),* }
    };
}

words!(AppendageWord {
    None,
    Limb,
    Feeler,
    Plate,
    Mouth,
    Vane
});
words!(RoleWord {
    Mass,
    Limb,
    Plate,
    Sensor
});
words!(AnchorWord { Base, Middle, Tip });
words!(FacingWord {
    Front,
    Back,
    Left,
    Right,
    Above,
    Below
});
words!(ChainFacingWord {
    Outward,
    Inward,
    Above,
    Below,
    Front,
    Back
});

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouthWord {
    Bulk,
    Jaw,
}

/// Whether a plate is held out from the body or worn against it.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WornWord {
    Held,
    Covering,
}
