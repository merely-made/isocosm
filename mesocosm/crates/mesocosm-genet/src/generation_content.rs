// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared content admission for generated starts and their command-line preview.

use mesocosm_core::{
    Founding, PartPalette, PartTemplate, Role, RoleShapes, world::generation::Request,
};
use mesocosm_mesh::content::{ContentPack, Palette, Shape};
use serde::{Deserialize, Serialize};

/// Mesocosm's development palette, carried through the mesh's generation
/// boundary.
///
/// The mesh owns generation and addressing; which shapes a world admits is
/// this product's development model, so the palette crosses as a value the
/// mesh only reads and rewrites through [`Palette`].  The wrapper is
/// transparent to serde, so a recorded pack's bytes are the palette's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DevelopmentPalette(pub PartPalette);

/// A content pack in this product's palette vocabulary.
pub type Pack = ContentPack<DevelopmentPalette>;

impl Palette for DevelopmentPalette {
    fn admitted(&self, role: Role) -> Vec<(u8, Shape)> {
        let shapes: RoleShapes = self.0.shapes(role);
        std::iter::once((0, shapes.default))
            .chain(
                shapes
                    .extra
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, template)| {
                        template.map(|template| (index as u8 + 1, template))
                    }),
            )
            .map(|(slot, template)| {
                (
                    slot,
                    Shape {
                        volume: template.volume,
                        half_extent: template.half_extent,
                    },
                )
            })
            .collect()
    }

    fn admit(&mut self, role: Role, slot: u8, shape: Shape) {
        let shapes = match role {
            Role::Mass => &mut self.0.mass,
            Role::Limb => &mut self.0.limb,
            Role::Plate => &mut self.0.plate,
            Role::Sensor => &mut self.0.sensor,
        };
        let template = PartTemplate {
            volume: shape.volume,
            half_extent: shape.half_extent,
        };
        match slot {
            0 => shapes.default = template,
            _ => {
                *shapes.extra[usize::from(slot - 1)]
                    .as_mut()
                    .expect("generation only visits admitted palette slots") = template
            },
        }
    }
}

/// Explicit structure uses the specimen bench's full shape vocabulary.
/// Earlier requests retain their original palette and saved seed streams.
pub fn palette(request: &Request) -> PartPalette {
    if request.criteria.structure.is_some() {
        Founding::SpacedRoster.palette()
    } else {
        Founding::Drawn.palette()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::world::generation::Structure;

    #[test]
    fn structure_admits_the_bench_palette_without_changing_earlier_requests() {
        let mut request = Request::default();
        assert_eq!(palette(&request), Founding::Drawn.palette());
        request.criteria.structure = Some(Structure::default());
        assert_eq!(palette(&request), Founding::SpacedRoster.palette());
        assert_ne!(palette(&request), Founding::Drawn.palette());
        let pack = Pack::generate(DevelopmentPalette(palette(&request))).unwrap();
        let recorded = serde_json::to_vec(&pack).unwrap();
        let restored: Pack = serde_json::from_slice(&recorded).unwrap();
        assert_eq!(restored, pack);
        assert!(restored.resolve_for(pack.palette).is_ok());
    }
}
