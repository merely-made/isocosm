// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host presentation presets. They never enter a world or a played trace.

use super::{Grade, PALETTE, Section};
use isometer::lens::TerrainAppearance;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerrainStyle {
    #[default]
    Auto,
    Classic,
    Habitat,
}

impl Section {
    pub fn configure_terrain(
        &mut self,
        style: TerrainStyle,
        section_centre: [f32; 3],
        clearing_y: f32,
    ) {
        let before = (self.grade, self.terrain_appearance);
        if style.resolved() == TerrainStyle::Classic {
            self.terrain_appearance = None;
            self.grade = Grade::retro(PALETTE);
            if before != (self.grade, self.terrain_appearance) {
                self.invalidate_query();
            }
            return;
        }
        self.grade = Grade {
            fog_start: 1.0,
            ..Grade::clay()
        };
        self.terrain_appearance = Some(TerrainAppearance {
            soil: [0.30, 0.22, 0.17],
            rock: [0.25, 0.31, 0.34],
            unknown: [0.60, 0.23, 0.58],
            sky: [0.64, 0.73, 0.76],
            underground: [0.09, 0.12, 0.14],
            section_centre,
            clearing_y,
        });
        if before != (self.grade, self.terrain_appearance) {
            self.invalidate_query();
        }
    }
}

impl TerrainStyle {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "classic" => Some(Self::Classic),
            "habitat" => Some(Self::Habitat),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Classic => "classic",
            Self::Habitat => "habitat",
        }
    }

    /// Auto is classic; habitat is asked for by name.
    pub fn resolved(self) -> Self {
        match self {
            Self::Auto => Self::Classic,
            explicit => explicit,
        }
    }
}
