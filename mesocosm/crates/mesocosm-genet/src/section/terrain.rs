// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host presentation presets. They never enter a world or a played trace.

use super::{Grade, PALETTE, Section};
use isometer::lens::{TerrainAppearance, TerrainPalette};
use isometer::space::Materials;

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
        let [soil, rock, unknown] = terrain_colours(style);
        self.terrain_appearance = Some(TerrainAppearance {
            soil,
            rock,
            unknown,
            sky: [0.64, 0.73, 0.76],
            underground: [0.09, 0.12, 0.14],
            section_centre,
            clearing_y,
        });
        if before != (self.grade, self.terrain_appearance) {
            self.invalidate_query();
        }
    }

    pub(super) fn bind_terrain_materials(&mut self, materials: Materials) {
        let style = if self.terrain_appearance.is_some() {
            TerrainStyle::Habitat
        } else {
            TerrainStyle::Classic
        };
        self.scene
            .set_terrain_palette(Some(terrain_palette(style, materials)));
    }
}

/// Bind the lift's world-local ids once, for both terrain looks. Water is
/// still opaque geometry; its colour must not fall through to unknown.
fn terrain_palette(style: TerrainStyle, materials: Materials) -> TerrainPalette {
    let [soil, rock, unknown] = terrain_colours(style);
    TerrainPalette::new([unknown])
        .with_material(materials.water, [0.12, 0.34, 0.52])
        .with_material(materials.soil, soil)
        .with_material(materials.rock, rock)
}

fn terrain_colours(style: TerrainStyle) -> [[f32; 3]; 3] {
    match style.resolved() {
        TerrainStyle::Habitat => [[0.30, 0.22, 0.17], [0.25, 0.31, 0.34], [0.60, 0.23, 0.58]],
        _ => [[0.38, 0.24, 0.13], [0.32, 0.34, 0.40], [0.66, 0.20, 0.72]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_looks_resolve_world_material_ids_instead_of_tracer_constants() {
        let native = Materials {
            air: 0,
            water: 1,
            soil: 2,
            rock: 3,
        };
        let reordered = Materials {
            air: 0,
            water: 7,
            soil: 5,
            rock: 1,
        };
        for style in [
            TerrainStyle::Auto,
            TerrainStyle::Classic,
            TerrainStyle::Habitat,
        ] {
            let a = terrain_palette(style, native);
            let b = terrain_palette(style, reordered);
            for (old, new) in [
                (native.water, reordered.water),
                (native.soil, reordered.soil),
                (native.rock, reordered.rock),
            ] {
                assert_eq!(a.colour(old), b.colour(new));
                assert_ne!(a.colour(old), a.colour(0));
            }
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
