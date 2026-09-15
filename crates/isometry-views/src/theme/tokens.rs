//! Token appearance: the one voxel rig and the palette each sprite wears.
//!
//! [`token_recipes`] is the source of truth. The DOM board bakes it to
//! `.token-*` sprite rules here; the scene board (B2) hands the same rig and
//! the same palette to `isometer_mesh::TokenBody` and draws it live. Two
//! projections of one recipe, not two recipes.

use isometer_mesh::bake::{BakeParams, Palette, bake_facing, demo};
use isometer_mesh::voxel::Voxels;

/// The board's iso stamp: a 2:1 diamond, four facings, two pixels of margin.
pub(crate) const BOARD_BAKE: BakeParams = BakeParams {
    half_w: 2,
    cube_h: 2,
    facings: 4,
    margin: 2,
};

/// The rig every starter token is drawn from, and each sprite's palette.
///
/// Palette-swaps of one rig (skin index 0, shirt index 1), which proves
/// recolour across the starter bestiary; per-monster voxel models arrive with
/// parts packs (P3).
pub(crate) fn token_recipes() -> (Voxels, Vec<(&'static str, Palette)>) {
    let (rig, base) = demo::hero();
    let recolor = |skin: [u8; 3], shirt: [u8; 3]| -> Palette {
        Palette::new(
            base.0
                .iter()
                .enumerate()
                .map(|(i, c)| match i {
                    0 => skin,
                    1 => shirt,
                    _ => *c,
                })
                .collect(),
        )
    };
    let variants = vec![
        ("knight", base.clone()),
        ("hero", base.clone()),
        ("scavenger", recolor([184, 139, 102], [116, 105, 69])),
        ("traveler", recolor([220, 179, 142], [84, 123, 158])),
        ("goblin", recolor([140, 165, 110], [72, 110, 60])),
        ("orc", recolor([120, 140, 95], [92, 70, 55])),
        ("skeleton", recolor([226, 223, 211], [198, 194, 180])),
        ("wolf", recolor([150, 150, 158], [92, 92, 100])),
    ];
    (rig, variants)
}

/// Bake the demo voxel rig to `.token-*` sprite rules (data-URI PNGs), called
/// once from [`board_css`]. `background-size: contain` plus a bottom anchor
/// stands the sprite in the 24x36 token box with its feet at the tile.
pub(super) fn voxel_token_css() -> String {
    let (rig, variants) = token_recipes();
    let mut css = String::new();
    for (class, pal) in &variants {
        let uri = bake_facing(&rig, pal, 0, &BOARD_BAKE).to_png_data_uri();
        css.push_str(&format!(
            ".token-{class} {{ background-image: url(\"{uri}\"); \
             background-size: contain; background-position: bottom center; }}\n"
        ));
    }
    css
}
