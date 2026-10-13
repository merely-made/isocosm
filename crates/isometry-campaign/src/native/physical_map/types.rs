// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::projects::{DebitPreview, DebitSelection};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialBinding { pub id:u8,pub key:Key,pub lineage:Key,pub account:Key,pub density:u64 }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum Operation { Carve, Fill(MaterialBinding) }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellGeometry { pub table:CellEdit,pub shape:Shape,pub operation:Operation }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryProvider {
    pub site:Id,pub window:super::super::PlacementWindow,pub cells:Vec<CellGeometry>,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialQuantity { pub material:MaterialBinding,pub cells:i64,pub quantity:u64 }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step { pub terrain_before:Key,pub materials:Vec<MaterialQuantity>,pub selections:Vec<DebitSelection> }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalMapInput {
    pub version:u32,pub expected_source:Key,pub expected_game:Key,pub expected_terrain:Key,
    pub actor:Id,pub body_revision:u64,pub metadata:MapEdit,pub geometry:GeometryProvider,pub steps:Vec<Step>,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct StepPreview {
    pub index:usize,pub command:Command,pub terrain_before:Key,pub materials:Vec<MaterialQuantity>,
    pub supplied:bool,pub refusal:Option<String>,pub debits:DebitPreview,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct PhysicalMapPreview { pub steps:Vec<StepPreview>,pub complete:bool,pub refusal:Option<String> }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalMapRecord {
    pub input:PhysicalMapInput,pub participant:Id,pub edit_entries:Vec<Key>,pub metadata_entry:Key,pub terrain_after:Key,
}
