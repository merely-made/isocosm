// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation styles a pack declares by tag (cases 1178, 1182).
//!
//! One table shape for every product: a tag names a look, and each product
//! decides which of its things carry which tag. Display data only; a style
//! never enters native rules, a save or a digest.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// How a body's lit colour reaches the display-encoded target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyDisplay {
    /// Exact sRGB with no tonemap: the baseline, like the terrain (1178).
    #[default]
    #[serde(rename = "srgb")]
    Srgb,
    /// Khronos PBR Neutral, then a 2.2 power curve: the look of Mesocosm's
    /// retired voxel tenant, kept as a style.
    #[serde(rename = "neutral")]
    NeutralGamma22,
}

/// One tag's look. Fields default to the baseline, so a tag names only what
/// it changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    #[serde(default)]
    pub display: BodyDisplay,
}

/// A tag no table declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownStyle(pub String);

/// Declared tags and their looks: a pack's `[styles]` table.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StyleTable(BTreeMap<String, Style>);

impl StyleTable {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn declares(&self, tag: &str) -> bool {
        self.0.contains_key(tag)
    }

    /// The look a declared tag names; an undeclared tag is refused.
    pub fn resolve(&self, tag: &str) -> Result<Style, UnknownStyle> {
        self.0
            .get(tag)
            .copied()
            .ok_or_else(|| UnknownStyle(tag.to_owned()))
    }

    /// Lay a later pack's tags over these: a tag it declares replaces the
    /// same tag, and every other tag stays.
    pub fn overlay(&mut self, later: StyleTable) {
        self.0.extend(later.0);
    }

    /// Tags are non-empty words with no surrounding space.
    pub fn validate(&self) -> Result<(), String> {
        match self
            .0
            .keys()
            .find(|tag| tag.trim().is_empty() || tag.trim() != tag.as_str())
        {
            Some(tag) => Err(format!("style tag {tag:?} is empty or padded")),
            None => Ok(()),
        }
    }
}

impl FromIterator<(String, Style)> for StyleTable {
    fn from_iter<I: IntoIterator<Item = (String, Style)>>(tags: I) -> Self {
        Self(tags.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declared_tag_resolves_and_an_unknown_one_is_refused() {
        let film = Style {
            display: BodyDisplay::NeutralGamma22,
        };
        let table: StyleTable = [("film".to_owned(), film)].into_iter().collect();
        assert_eq!(table.resolve("film"), Ok(film));
        assert_eq!(table.resolve("fog"), Err(UnknownStyle("fog".into())));
        assert!(table.validate().is_ok());
        let padded: StyleTable = [(" film".to_owned(), film)].into_iter().collect();
        assert!(padded.validate().is_err());
    }
}
