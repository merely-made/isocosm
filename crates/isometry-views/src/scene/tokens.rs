//! Tokens as live bodies: the DOM board's own recipes, meshed.
//!
//! The DOM board bakes `theme::token_recipes` to sprite rules; this hands the
//! same rig and the same palette to [`TokenBody`] (I4) and registers the
//! palette's colour table on the body layer (I6), so a token drawn through the
//! scene wears the colours its sprite wears. Nothing is invented here — a
//! sprite name the recipe table does not carry gets [`PLACEHOLDER`] and is
//! counted, rather than a silently missing piece on the board.
//!
//! **Scale.** A recipe is authored in its own voxels; the board's unit is the
//! tile. The rig is stood [`FOOTPRINT`] of a tile wide, which puts the shipped
//! 8-voxel hero at three quarters of a diamond and its 12-voxel height at
//! about 22 screen pixels against the DOM sprite's 36. Sprite-for-sprite size
//! parity is not B2's (the renderers differ); standing inside its own tile is.

use std::collections::BTreeMap;

use isometer::mesh::VolumeSource;
use isometer::mesh::bake::Palette;
use isometer::mesh::voxel::Voxels;
use isometer::{PaletteColour, TokenBody, VolumeMap, material_colours};
use isometer_core::SpeciesId;
use isometry_core::Facing;

use crate::theme::token_recipes;

/// A token's footprint as a fraction of a tile's diagonal.
pub const FOOTPRINT: f32 = 0.75;

/// The sprite name a token with no recipe is drawn as: one voxel, so the piece
/// is on the board and visibly wrong rather than absent.
pub const PLACEHOLDER: &str = "";

/// Every token body the board can draw, plus the one volume map the frame
/// resolves them through.
///
/// Built once and held: the recipes are static, and a `TokenBody` carries a
/// content-addressed volume, so every sprite of the one rig shares a single
/// mesh and only the colour table differs.
pub struct TokenBodies {
    bodies: BTreeMap<String, TokenBody>,
    /// The scene's volume source: every distinct recipe, collapsed by content.
    pub volumes: VolumeMap,
}

impl Default for TokenBodies {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBodies {
    pub fn new() -> Self {
        let (rig, variants) = token_recipes();
        let mut bodies = BTreeMap::new();
        let mut volumes = VolumeMap::new();
        let mut add = |name: &str, body: TokenBody| {
            if let Some(volume) = body.volumes.volume(body.volume) {
                volumes.insert(body.volume, volume.clone());
            }
            bodies.insert(name.to_owned(), body);
        };
        for (index, (name, _)) in variants.iter().enumerate() {
            add(
                name,
                TokenBody::from_voxels(SpeciesId(index as u32 + 1), MASS_MG, &rig),
            );
        }
        // The stand-in: one voxel of palette index 0, which becomes material
        // 1 and takes the placeholder colour below.
        let mut cell = Voxels::new(1, 1, 1);
        cell.set(0, 0, 0, 0);
        add(
            PLACEHOLDER,
            TokenBody::from_voxels(SpeciesId(0), MASS_MG, &cell),
        );
        Self { bodies, volumes }
    }

    /// The body a sprite name draws as, and whether it is the placeholder.
    pub fn body(&self, sprite: &str) -> (&TokenBody, bool) {
        match self.bodies.get(sprite) {
            Some(body) => (body, false),
            None => (
                self.bodies
                    .get(PLACEHOLDER)
                    .expect("the placeholder body is always built"),
                true,
            ),
        }
    }

    /// The colour table a sprite's materials index into: its palette shifted
    /// by one, as [`material_colours`] gives it. A sprite with no recipe takes
    /// the placeholder colour.
    pub fn colours(&self, sprite: &str) -> Vec<PaletteColour> {
        let (_, variants) = token_recipes();
        variants
            .iter()
            .find(|(name, _)| *name == sprite)
            .map(|(_, palette)| material_colours(palette))
            .unwrap_or_else(|| material_colours(&Palette::new(vec![PLACEHOLDER_COLOUR])))
    }

    /// The scale that stands `sprite` [`FOOTPRINT`] of a tile wide.
    pub fn scale(&self, sprite: &str) -> f32 {
        let size = self.body(sprite).0.size();
        let widest = size[0].max(size[2]).max(1) as f32;
        FOOTPRINT / widest
    }
}

/// A token's mass is not a board concept; the recipe lane wants one number and
/// this is the demo rig's own.
const MASS_MG: u64 = 70_000;

/// Magenta: a placeholder that reads as a mistake at a glance.
const PLACEHOLDER_COLOUR: [u8; 3] = [220, 40, 200];

/// The yaw a four-way facing turns a body to.
///
/// The board's axes are world `+x` for a column and world `+z` for a row, so
/// South (row increasing) is the zero and each quarter turn follows the
/// compass anticlockwise about world up. The rig is symmetric enough that this
/// reads as a turn rather than as a specific pose; B3 owns whatever facing art
/// wants beyond it.
pub fn yaw_of(facing: Facing) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    match facing {
        Facing::South => 0.0,
        Facing::West => FRAC_PI_2,
        Facing::North => std::f32::consts::PI,
        Facing::East => 3.0 * FRAC_PI_2,
    }
}

/// The tint an owner's pieces are drawn in.
///
/// Sides are named by the map, not by the substrate, so the tint is a hash of
/// the owner string folded into a light pastel: distinguishable, and never a
/// colour that swamps the recipe's own palette. No owner draws untinted.
pub fn owner_tint(owner: Option<&str>) -> [f32; 3] {
    let Some(owner) = owner else {
        return [1.0; 3];
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in owner.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    [0, 1, 2].map(|channel| 0.72 + ((hash >> (channel * 8)) & 0xff) as f32 / 255.0 * 0.28)
}
