//! The board's state colours: the tile tints, the fog shroud and the pane's
//! own ground, each named once so the stylesheet and the scene agree.
//!
//! `kinds.rs` does this for what a tile *is*; this does it for what a tile is
//! *doing*. The DOM board spells these as `.tile-<state>` background rules and
//! a translucent `.fog-shroud` diamond; the scene board has no elements, so it
//! spends a terrain material on each and reads the colour from here. The
//! receipts in `scene::overlay` parse every one of them back out of
//! [`board_css`](super::board_css), so a hand-edited rule fails a test rather
//! than drifting one board away from the other.

/// `(class, hex)` for every state tint the ground layer can wear, **in the
/// stylesheet's own precedence, weakest first**.
///
/// Source order decides among equal specificity, so a later rule wins; the two
/// encounter rules carry an extra class each and win over all of them. Read
/// bottom-up this is: a door beats an encounter site (the three-class rule),
/// an encounter site beats everything below it, and selection is the weakest
/// thing a tile can show.
pub(crate) const TILE_TINTS: &[(&str, &str)] = &[
    ("tile-selected", "#ffd766"),
    ("tile-reach", "#4a6ea8"),
    ("tile-path", "#7fa3d8"),
    ("tile-template", "#d98a4a"),
    ("tile-door", "#9a7bd8"),
    ("tile-encounter", "#b38b49"),
];

/// The fog shroud over remembered terrain, as the `.fog-shroud` rule's
/// `rgba(8, 10, 16, 0.6)` composites: the colour, then the coverage.
pub(crate) const SHROUD: ([f32; 3], f32) = ([8.0 / 255.0, 10.0 / 255.0, 16.0 / 255.0], 0.6);

/// The `.pane` background: the near-black ground the board stands on, which
/// the scene paints where the DOM board simply shows the pane through.
pub(crate) const PANE_GROUND: &str = "#101218";

/// `#rrggbb` to components in 0..1. Anything else reads black.
pub(crate) fn hex_rgb(hex: &str) -> [f32; 3] {
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |i: usize| {
        u8::from_str_radix(digits.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0
    };
    [byte(0), byte(2), byte(4)]
}

/// One colour under the fog shroud, composited exactly as the DOM board's
/// translucent diamond over it.
pub(crate) fn shrouded(colour: [f32; 3]) -> [f32; 3] {
    let (shroud, alpha) = SHROUD;
    [0, 1, 2].map(|c| colour[c] * (1.0 - alpha) + shroud[c] * alpha)
}
