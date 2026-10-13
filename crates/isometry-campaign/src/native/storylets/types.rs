// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::StoryletInput;
use crate::{ItemId, StoryletEffect};
use crate::native::{CharacterPlacement, items::ItemInput, physical_map::PhysicalMapInput};
use isocosm::schema::{Id, Key};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposedStoryletInput {
    pub version: u32,
    pub storylet: StoryletInput,
    pub expected_game: Key,
    pub steps: Vec<StoryletStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StoryletStep {
    Map { effect: u32, input: PhysicalMapInput },
    Item { effect: u32, item: ItemId, inputs: Vec<ItemInput> },
    Placement { expected_native: Key, expected_game: Key, placements: Vec<CharacterPlacement> },
}

/// Accepted public outputs, never private requirements or secret-store contents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposedStoryletPublic {
    pub key: Key,
    pub source_digest: Key,
    /// Disclosed cast only; an empty projection may omit the host-private cast.
    pub cast: BTreeMap<Key, Key>,
    pub effects: Vec<StoryletEffect>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryletLink {
    pub step: usize,
    pub entries: Vec<Key>,
    pub entity: Option<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposedStoryletRecord {
    pub input: ComposedStoryletInput,
    pub participant: Id,
    pub public: ComposedStoryletPublic,
    pub entries: Vec<Key>,
    pub marker: Key,
    pub links: Vec<StoryletLink>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedStoryletPreview {
    pub complete: bool,
    pub refusal: Option<String>,
    pub public: Option<ComposedStoryletPublic>,
}
