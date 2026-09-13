// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Composable arrangements and organ chains in the existing paid body grammar.
//! Roots describes downward branching, and wings/fins describe static shapes.

use crate::world::generation::Error;
use crate::{Kingdom, PartPalette, Recipe, Rng, Soma};
use serde::{Deserialize, Serialize};

mod draw;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureLayout {
    #[default]
    Chain,
    Radial,
    Crown,
    Mat,
    Vine,
    Roots,
}

impl StructureLayout {
    pub const ALL: [Self; 6] = [
        Self::Chain,
        Self::Radial,
        Self::Crown,
        Self::Mat,
        Self::Vine,
        Self::Roots,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Chain => "chain",
            Self::Radial => "radial",
            Self::Crown => "crown",
            Self::Mat => "mat",
            Self::Vine => "vine",
            Self::Roots => "roots",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureOrgans {
    Bare,
    #[default]
    Legs,
    Wings,
    Fins,
    Leaves,
    Feelers,
}

impl StructureOrgans {
    pub const ALL: [Self; 6] = [
        Self::Bare,
        Self::Legs,
        Self::Wings,
        Self::Fins,
        Self::Leaves,
        Self::Feelers,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Legs => "legs",
            Self::Wings => "wings",
            Self::Fins => "fins",
            Self::Leaves => "leaves",
            Self::Feelers => "feelers",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Structure {
    pub layout: StructureLayout,
    /// Additional organs. Feeding role still supplies its necessary mouth or
    /// canopy, including with Bare. These are not movement capabilities.
    pub organs: StructureOrgans,
    /// Exact structural stretches beyond the feeding root, 2..=6.
    /// Paired organs and chain links add separately counted realized parts.
    pub branch_count: u8,
    /// Maximum seeded length per stretch, 1..=4 segments.
    pub segment_length: u8,
}

impl Default for Structure {
    fn default() -> Self {
        Self {
            layout: StructureLayout::Chain,
            organs: StructureOrgans::Legs,
            branch_count: 4,
            segment_length: 3,
        }
    }
}

impl Structure {
    pub fn required_role(self) -> Option<Kingdom> {
        (self.organs == StructureOrgans::Leaves).then_some(Kingdom::Producer)
    }

    pub(in crate::world::generation) fn validate(self, role: Option<Kingdom>) -> Result<(), Error> {
        if !(2..=6).contains(&self.branch_count) || !(1..=4).contains(&self.segment_length) {
            return Err(Error::Invalid(
                "structural stretches must be 2..6 and length 1..4",
            ));
        }
        if self
            .required_role()
            .is_some_and(|required| role.is_some_and(|role| role != required))
        {
            return Err(Error::Invalid("leaves require producer feeding role"));
        }
        Ok(())
    }

    pub(in crate::world::generation) fn accepts(self, soma: &Soma) -> bool {
        soma.absent.is_empty()
    }

    pub(in crate::world::generation) fn generate(
        self,
        rng: &mut Rng,
        role: Kingdom,
        palette: PartPalette,
    ) -> Recipe {
        draw::recipe(self, rng, role, palette)
    }
}
