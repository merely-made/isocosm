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

/// Bake every class `library` binds to a `.token-<class>` rule (data-URI
/// PNGs). A class that does not resolve gets no rule, so its token shows as
/// the bare box rather than as some other class.
pub fn class_css(library: &Library) -> String {
    let mut css = String::new();
    for class in library.classes() {
        let Ok(resolved) = library.resolve(class, None, []) else {
            continue;
        };
        let uri = bake_facing(&resolved.voxels, &resolved.palette, 0, &BOARD_BAKE).to_png_data_uri();
        css.push_str(&format!(
            ".token-{class} {{ background-image: url(\"{uri}\"); {SPRITE_FIT} }}
"
        ));
    }
    css
}

/// The core pack's classes, baked once into [`super::board_css`].
pub(super) fn voxel_token_css() -> String {
    class_css(&Library::core())
}
