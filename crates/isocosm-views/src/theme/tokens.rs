//! Token appearance on the DOM board: each class's look baked to a
//! `.token-<class>` sprite rule.
//!
//! The looks are the packs' (`crate::appearance`); the scene board meshes the
//! same resolution live, and a token whose look differs from its class (its
//! own overrides, an equipped item's layer) carries its own baked sprite
//! inline (`board::tokens`). Two projections of one look, not two looks.

use isometer_mesh::bake::{BakeParams, bake_facing};

use crate::appearance::Library;

/// The board's iso stamp: a 2:1 diamond, four facings, two pixels of margin.
pub(crate) const BOARD_BAKE: BakeParams = BakeParams {
    half_w: 2,
    cube_h: 2,
    facings: 4,
    margin: 2,
};

/// The declarations that stand a baked sprite in the 24x36 token box with its
/// feet at the tile.
pub(crate) const SPRITE_FIT: &str =
    "background-size: contain; background-position: bottom center;";

/// Bake every class `library` binds to sprite rules (data-URI PNGs): one per
/// true quarter-turn facing (`.token-<class>.token-facing-<f>`, the turns the
/// creator's preview shows), and the south view as the bare `.token-<class>`
/// for places that show a class without a facing. A class that does not
/// resolve gets no rule, so its token shows as the bare box rather than as
/// some other class.
pub fn class_css(library: &Library) -> String {
    use isocosm_core::Facing;
    let mut css = String::new();
    for class in library.classes() {
        let Ok(resolved) = library.resolve(class, None, []) else {
            continue;
        };
        for (facing, name) in [
            (Facing::South, "south"),
            (Facing::East, "east"),
            (Facing::North, "north"),
            (Facing::West, "west"),
        ] {
            let index = crate::appearance::bake_facing_of(facing);
            let uri = bake_facing(&resolved.voxels, &resolved.palette, index, &BOARD_BAKE)
                .to_png_data_uri();
            let selector = if facing == Facing::South {
                format!(".token-{class}, .token-{class}.token-facing-{name}")
            } else {
                format!(".token-{class}.token-facing-{name}")
            };
            css.push_str(&format!(
                "{selector} {{ background-image: url(\"{uri}\"); {SPRITE_FIT} }}
"
            ));
        }
    }
    css
}

/// The core pack's classes, baked once into [`super::board_css`].
pub(super) fn voxel_token_css() -> String {
    class_css(&Library::core())
}
