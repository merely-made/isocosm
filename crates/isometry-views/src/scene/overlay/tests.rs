//! The overlay channel's receipts: the palette's shape, the stylesheet's own
//! precedence, and the colours read back out of the sheet they came from.

use super::super::terrain::material_of;
use super::*;
use crate::demo::demo_map;
use crate::theme::{PANE_GROUND, board_css};
use isometry_core::TileKindId;

/// The value a `.<class> { background-color: ... }` rule carries, read out of
/// the generated sheet. The first match wins, so a `.alt` or a
/// higher-specificity twin never stands in for the base rule.
fn rule_colour(css: &str, class: &str) -> [f32; 3] {
    let needle = format!("{class} {{ background-color: ");
    let at = css
        .find(&needle)
        .unwrap_or_else(|| panic!("the sheet carries no rule for {class}"));
    let start = at + needle.len();
    hex_rgb(&css[start..start + 7])
}

/// Every tint the scene paints is the colour the DOM board's own rule carries.
/// Hand-edit either and this fails, which is the whole point of naming them
/// once.
#[test]
fn every_tint_is_the_colour_its_stylesheet_rule_carries() {
    let css = board_css();
    for tint in Tint::ALL {
        assert_eq!(
            tint.colour(),
            rule_colour(&css, tint.class()),
            "{} disagrees with its stylesheet rule",
            tint.class()
        );
    }
}

/// The two colours that are not tile rules: the pane's ground, which the scene
/// paints where the DOM board simply shows the pane, and the shroud's
/// translucent black.
#[test]
fn the_pane_ground_and_the_shroud_are_the_sheets_own() {
    let css = board_css();
    let block = |selector: &str| {
        let at = css
            .find(selector)
            .unwrap_or_else(|| panic!("no {selector} rule"));
        let end = css[at..].find('}').expect("a closed rule") + at;
        css[at..end].to_owned()
    };
    assert!(
        block(".pane {").contains(&format!("background-color: {PANE_GROUND};")),
        "the pane's ground is what the scene paints for a background"
    );
    let (colour, alpha) = SHROUD;
    assert!(
        block(".fog-shroud {").contains(&format!(
            "rgba({}, {}, {}, {alpha})",
            (colour[0] * 255.0).round(),
            (colour[1] * 255.0).round(),
            (colour[2] * 255.0).round(),
        )),
        "the shroud composites exactly what the sheet lays over a tile"
    );
}

/// The stylesheet's cascade decides which tint a tile wearing several shows,
/// and this is that order: a door outright, then an encounter site, then the
/// four single-class rules in source order.
#[test]
fn a_tile_wearing_several_tints_shows_the_one_the_cascade_would() {
    let at = (3, 4);
    let mut overlays = Overlays {
        selected: Some(at),
        ..Overlays::default()
    };
    assert_eq!(overlays.tint(at), Some(Tint::Selected));
    overlays.reach.insert(at);
    assert_eq!(overlays.tint(at), Some(Tint::Reach), "reach over selection");
    overlays.path.insert(at);
    assert_eq!(overlays.tint(at), Some(Tint::Path), "path over reach");
    overlays.template.insert(at);
    assert_eq!(overlays.tint(at), Some(Tint::Template));
    overlays.encounters.insert(at);
    assert_eq!(
        overlays.tint(at),
        Some(Tint::Encounter),
        "the two-class encounter rule outranks every single-class one"
    );
    overlays.doors.insert(at);
    assert_eq!(
        overlays.tint(at),
        Some(Tint::Door),
        "and `.tile.tile-door.tile-encounter` carries the door's colour"
    );
    assert_eq!(overlays.tint((9, 9)), None, "a plain tile wears its kind");
}

/// The table's blocks land where the layout says, the shrouded twin of a
/// colour is that colour under the sheet's own shroud, and nothing runs past
/// the tracer's bound.
#[test]
fn the_palette_lays_the_kinds_the_tints_and_their_shrouded_twins_in_order() {
    let map = demo_map();
    let layout = BoardPalette::of(&map);
    let palette = terrain_palette(&map);
    let kinds = map.tile_kinds.len();
    assert!(kinds <= SHROUDED_KIND_LIMIT, "the demo keeps its shroud");
    assert_eq!(palette.len(), layout.len());
    assert!(palette.len() <= MAX_TERRAIN_MATERIALS);
    assert_eq!(
        palette.colour(0),
        Some([0.0, 0.0, 0.0]),
        "entry 0 is unknown"
    );

    for (index, kind) in map.tile_kinds.iter().enumerate() {
        let material = material_of(TileKindId(index as u16));
        assert_eq!(material as usize, index + 1, "kinds come first");
        let colour = tile_kind_colour(kind).unwrap_or([0.0, 0.0, 0.0]);
        assert_eq!(palette.colour(material), Some(colour));
        assert_eq!(
            palette.colour(layout.shrouded_material(material)),
            Some(shrouded(colour)),
            "{kind} under the shroud"
        );
    }
    for tint in Tint::ALL {
        let material = layout.tint_material(tint);
        assert!(material as usize > kinds, "tints follow the kinds");
        assert_eq!(palette.colour(material), Some(tint.colour()));
        assert_eq!(
            palette.colour(layout.shrouded_material(material)),
            Some(shrouded(tint.colour())),
            "{} under the shroud",
            tint.class()
        );
    }
}

/// A board with too many kinds to hold the doubled half shrouds everything
/// with one flat entry rather than overrunning the tracer's 64.
#[test]
fn a_board_past_the_bound_keeps_one_flat_shroud() {
    let mut map = demo_map();
    while map.tile_kinds.len() <= SHROUDED_KIND_LIMIT {
        map.intern_tile_kind(&format!("kind-{}", map.tile_kinds.len()));
    }
    let layout = BoardPalette::of(&map);
    let palette = terrain_palette(&map);
    assert!(palette.len() <= MAX_TERRAIN_MATERIALS);
    let flat = layout.shrouded_material(1);
    assert_eq!(
        layout.shrouded_material(layout.tint_material(Tint::Path)),
        flat,
        "past the bound every shrouded tile takes the one entry"
    );
    assert_eq!(palette.colour(flat), Some(shrouded([0.0, 0.0, 0.0])));
}
