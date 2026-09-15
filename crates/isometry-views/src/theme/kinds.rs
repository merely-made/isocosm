//! Tile-kind colours: the one place a kind's colour is decided.
//!
//! The `.tile-<kind>` stylesheet rules are generated from these tables, and
//! the scene adapter's terrain material palette reads the same entries, so a
//! tileset edit reaches the DOM tiles and the voxel ground together.

/// `(kind, base, alt)`, where `alt` is the checkerboard shade a kind may omit.
pub(crate) const BASE_TILE_KINDS: &[(&str, &str, Option<&str>)] = &[
    ("grass", "#4f8f3b", Some("#478536")),
    ("water", "#2f629e", Some("#2a5a93")),
    ("stone", "#8d9098", Some("#84878f")),
];

/// The starter campaign's own kinds, emitted with the rest of its sheet.
pub(crate) const WATCHTOWER_TILE_KINDS: &[(&str, &str, Option<&str>)] = &[
    ("rubble", "#81775f", None),
    ("forest-floor", "#405b35", Some("#46613a")),
    ("forest-path", "#8a7953", None),
];

/// `.tile-<kind>` rules for one table, in its own order.
pub(crate) fn tile_kind_css(kinds: &[(&str, &str, Option<&str>)]) -> String {
    let mut css = String::new();
    for (kind, base, alt) in kinds {
        css.push_str(&format!(".tile-{kind} {{ background-color: {base}; }}\n"));
        if let Some(alt) = alt {
            css.push_str(&format!(
                ".tile-{kind}.alt {{ background-color: {alt}; }}\n"
            ));
        }
    }
    css
}

/// The stylesheet's colour for one tile kind, as RGB components in 0..1.
pub fn tile_kind_colour(kind: &str) -> Option<[f32; 3]> {
    BASE_TILE_KINDS
        .iter()
        .chain(WATCHTOWER_TILE_KINDS)
        .find(|(k, _, _)| *k == kind)
        .map(|(_, base, _)| hex_rgb(base))
}

/// `#rrggbb` to components in 0..1. Anything else reads black.
fn hex_rgb(hex: &str) -> [f32; 3] {
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |i: usize| {
        u8::from_str_radix(digits.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0
    };
    [byte(0), byte(2), byte(4)]
}
