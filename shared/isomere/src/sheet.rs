// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The rules the products' sheets already agree on, plus the sizes they do
//! not.
//!
//! The 2026-09-15 inventory found the Mesocosm bench sheet and the Eponym
//! session sheet to be the same sheet rule for rule and in the same order;
//! only colours, four font sizes and three box numbers differed. [`shared`]
//! is that sheet with every difference lifted into a custom property, and
//! [`sheet`] is the whole document: the palette block, the size block, the
//! shared rules, then whatever the product still has to say for itself.
//!
//! The class vocabulary is the one already in the two views, so promotion is a
//! move and not a rename.

use crate::palette::{Palette, Seeds, css_vars, derive};

/// The lengths the two sheets disagreed on: four font sizes, the three box
/// numbers, and the gaps that went with them.
///
/// Every field is a CSS length (or a bare number, for line heights), so a
/// product can hand over `calc(100vh - 335px)` as easily as `440px`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sizes {
    /// Root font size.
    pub font: &'static str,
    /// Root line height for `p`.
    pub line: &'static str,
    /// `header` bottom gap.
    pub header_gap: &'static str,
    /// `h1` size.
    pub h1: &'static str,
    /// `h2` size.
    pub h2: &'static str,
    /// `h2` bottom gap.
    pub h2_gap: &'static str,
    /// `p` vertical margin.
    pub p_gap: &'static str,
    /// The gap between the scene column and the panel column.
    pub main_gap: &'static str,
    /// `.scene-card` padding.
    pub card_pad: &'static str,
    /// Box one: the viewport's height.
    pub viewport_height: &'static str,
    /// `.viewport` padding.
    pub viewport_pad: &'static str,
    /// `.viewport` frame width. `0` for a product with no frame.
    pub viewport_border_width: &'static str,
    /// `.toolbar` gap between buttons.
    pub toolbar_gap: &'static str,
    /// `.toolbar` top margin.
    pub toolbar_top: &'static str,
    /// `button` padding.
    pub button_pad: &'static str,
    /// `button` corner radius.
    pub button_radius: &'static str,
    /// `button` font size.
    pub button_font: &'static str,
    /// `.parts` gap between chips.
    pub parts_gap: &'static str,
    /// Box two: the parts palette's height.
    pub parts_height: &'static str,
    /// `.parts` bottom gap.
    pub parts_gap_bottom: &'static str,
    /// Box three: the reading column's height.
    pub reading_height: &'static str,
    /// `.field` bottom gap.
    pub field_gap: &'static str,
    /// `.field-name` font size.
    pub field_name_font: &'static str,
    /// `.field-name` bottom gap.
    pub field_name_gap: &'static str,
    /// `.field-value` font size.
    pub field_value_font: &'static str,
    /// `.field-value` line height.
    pub field_value_line: &'static str,
    /// The error line's reserved height, so it cannot reflow the column.
    pub error_height: &'static str,
    /// The error line's font size.
    pub error_font: &'static str,
    /// The control-help line's font size.
    pub help_font: &'static str,
}

impl Sizes {
    /// The defaults a product gets for saying nothing. Roughly the bench's
    /// reading sizes, which are the larger of the two sources.
    pub const DEFAULT: Self = Self {
        font: "15px",
        line: "1.5",
        header_gap: "20px",
        h1: "28px",
        h2: "20px",
        h2_gap: "16px",
        p_gap: "8px",
        main_gap: "24px",
        card_pad: "16px",
        viewport_height: "440px",
        viewport_pad: "8px",
        viewport_border_width: "3px",
        toolbar_gap: "8px",
        toolbar_top: "14px",
        button_pad: "8px 12px",
        button_radius: "5px",
        button_font: "14px",
        parts_gap: "6px",
        parts_height: "150px",
        parts_gap_bottom: "18px",
        reading_height: "430px",
        field_gap: "12px",
        field_name_font: "12px",
        field_name_gap: "3px",
        field_value_font: "14px",
        field_value_line: "1.4",
        error_height: "24px",
        error_font: "15px",
        help_font: "12px",
    };
}

impl Default for Sizes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The sizes as a `:root` block of `--isomere-*` custom properties, in a fixed
/// order so the emitted sheet is stable byte for byte.
pub fn css_sizes(sizes: &Sizes) -> String {
    let rows: [(&str, &str); 29] = [
        ("font-size", sizes.font),
        ("line", sizes.line),
        ("header-gap", sizes.header_gap),
        ("h1", sizes.h1),
        ("h2", sizes.h2),
        ("h2-gap", sizes.h2_gap),
        ("p-gap", sizes.p_gap),
        ("main-gap", sizes.main_gap),
        ("card-pad", sizes.card_pad),
        ("viewport-height", sizes.viewport_height),
        ("viewport-pad", sizes.viewport_pad),
        ("viewport-border-width", sizes.viewport_border_width),
        ("toolbar-gap", sizes.toolbar_gap),
        ("toolbar-top", sizes.toolbar_top),
        ("button-pad", sizes.button_pad),
        ("button-radius", sizes.button_radius),
        ("button-font", sizes.button_font),
        ("parts-gap", sizes.parts_gap),
        ("parts-height", sizes.parts_height),
        ("parts-gap-bottom", sizes.parts_gap_bottom),
        ("reading-height", sizes.reading_height),
        ("field-gap", sizes.field_gap),
        ("field-name-font", sizes.field_name_font),
        ("field-name-gap", sizes.field_name_gap),
        ("field-value-font", sizes.field_value_font),
        ("field-value-line", sizes.field_value_line),
        ("error-height", sizes.error_height),
        ("error-font", sizes.error_font),
        ("help-font", sizes.help_font),
    ];
    let mut out = String::from(":root {\n");
    for (name, value) in rows {
        out.push_str("  --isomere-");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push_str(";\n");
    }
    out.push_str("}\n");
    out
}

/// The rules the bench and the session sheets agree on, with every colour and
/// every disputed length as a custom property.
///
/// Rule order is the order both sheets were already written in, because a
/// later product rule overriding an earlier shared one is the whole mechanism
/// by which a product keeps its own look.
pub fn shared() -> &'static str {
    SHARED
}

const SHARED: &str = r#"
* { box-sizing:border-box; }
html, body { margin:0; padding:0; background:var(--isomere-background); color:var(--isomere-ink); font:var(--isomere-font-size) sans-serif; }
header { margin-bottom:var(--isomere-header-gap); }
h1 { margin:0; font-size:var(--isomere-h1); font-weight:700; }
h2 { margin:0 0 var(--isomere-h2-gap); font-size:var(--isomere-h2); }
p { margin:var(--isomere-p-gap) 0; line-height:var(--isomere-line); }
header p { color:var(--isomere-muted); }
#controls-help, .controls-help { color:var(--isomere-help); font-size:var(--isomere-help-font); }
main { display:flex; gap:var(--isomere-main-gap); align-items:flex-start; }
.scene-card { position:relative; overflow:hidden; padding:var(--isomere-card-pad); border:2px solid var(--isomere-card-border); background:var(--isomere-card); }
.viewport { display:block; width:100%; height:var(--isomere-viewport-height); min-height:240px; padding:var(--isomere-viewport-pad); border:var(--isomere-viewport-border-width) solid var(--isomere-viewport-border); color:rgb(255,255,255); }
.toolbar { display:flex; flex-wrap:wrap; gap:var(--isomere-toolbar-gap); margin-top:var(--isomere-toolbar-top); }
button { padding:var(--isomere-button-pad); border:1px solid var(--isomere-button-border); border-radius:var(--isomere-button-radius); background:var(--isomere-button-bg); color:var(--isomere-button-ink); font:var(--isomere-button-font) sans-serif; cursor:pointer; }
button:hover { background:var(--isomere-hover); }
button:focus { outline:2px solid var(--isomere-focus); outline-offset:2px; }
.parts { display:flex; flex-wrap:wrap; gap:var(--isomere-parts-gap); max-height:var(--isomere-parts-height); overflow:auto; margin-bottom:var(--isomere-parts-gap-bottom); }
.part.selected { background:var(--isomere-selected); color:var(--isomere-selected-ink); border-color:var(--isomere-selected-border); }
.reading { max-height:var(--isomere-reading-height); overflow:auto; }
.field { margin-bottom:var(--isomere-field-gap); }
.field-name, .journal-founding, .journal-live { font-size:var(--isomere-field-name-font); color:var(--isomere-muted); margin-bottom:var(--isomere-field-name-gap); }
.field-value { font-size:var(--isomere-field-value-font); line-height:var(--isomere-field-value-line); }
#notice, #viewport-error, .error-line { min-height:var(--isomere-error-height); font-size:var(--isomere-error-font); color:var(--isomere-error); }
"#;

/// The whole sheet: the palette block, the size block, the shared rules, then
/// the product's own. Uses [`Sizes::DEFAULT`]; a product with its own lengths
/// calls [`sheet_with`].
pub fn sheet(seeds: &Seeds, extra: &str) -> String {
    sheet_with(seeds, &Sizes::DEFAULT, extra)
}

/// [`sheet`] with an explicit size override.
pub fn sheet_with(seeds: &Seeds, sizes: &Sizes, extra: &str) -> String {
    from_palette(&derive(seeds), sizes, extra)
}

/// [`sheet_with`] over an already-derived palette, for a product that wants to
/// read or assert the palette before emitting it.
pub fn from_palette(palette: &Palette, sizes: &Sizes, extra: &str) -> String {
    let mut out = String::with_capacity(4096);
    out.push('\n');
    out.push_str(&css_vars(palette));
    out.push_str(&css_sizes(sizes));
    out.push_str(SHARED.trim_start_matches('\n'));
    out.push_str(extra.trim_start_matches('\n'));
    out
}
