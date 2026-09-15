// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seeds, the derived palette, and the custom-property block the shared sheet
//! reads.
//!
//! A product hands isomere the handful of colours it actually chose. Every
//! remaining slot the shared sheet needs — the states nobody picks by hand,
//! and the ones a product happened to leave alone — is filled by `tinct` off
//! those anchors. Anything a product *did* pick is copied through verbatim, so
//! a seed set spelled with today's hex derives today's palette exactly and the
//! existing capture sets hold. That is the ruling of 2026-09-15.

use tinct::{Srgb, best_on, color_to_hex, contrast, mix, oklch::Oklch, relative_luminance};

/// The colours a product hand-picked.
///
/// The four anchors are required because every derivation runs off them. The
/// rest of the struct is [`Picked`]: one `Option` per derived slot, so a
/// product that chose a hover colour years ago keeps it and a product that
/// never thought about hover gets a consistent one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seeds {
    /// The window itself.
    pub background: Srgb,
    /// Body text.
    pub ink: Srgb,
    /// Secondary text: field names, the header line.
    pub muted: Srgb,
    /// The one colour that means *chosen*: selection fills, focus rings.
    pub accent: Srgb,
    /// The caution hue, if the product has one. `None` derives it.
    pub warning: Option<Srgb>,
    /// The affirmative hue, if the product has one. `None` derives it.
    pub answer: Option<Srgb>,
    /// Slots the product picked by hand rather than leaving to derivation.
    pub picked: Picked,
}

impl Seeds {
    /// The four anchors, nothing picked and nothing else seeded.
    pub const fn anchors(background: Srgb, ink: Srgb, muted: Srgb, accent: Srgb) -> Self {
        Self {
            background,
            ink,
            muted,
            accent,
            warning: None,
            answer: None,
            picked: Picked::NONE,
        }
    }
}

/// Hand-picked overrides for the slots [`derive`] would otherwise fill.
///
/// `None` means "nobody chose this, derive it". `Some` is copied verbatim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Picked {
    pub panel: Option<Srgb>,
    pub card: Option<Srgb>,
    pub help: Option<Srgb>,
    pub accent_ink: Option<Srgb>,
    pub border: Option<Srgb>,
    pub card_border: Option<Srgb>,
    pub button_border: Option<Srgb>,
    pub viewport_border: Option<Srgb>,
    pub button_bg: Option<Srgb>,
    pub button_ink: Option<Srgb>,
    pub hover: Option<Srgb>,
    pub focus: Option<Srgb>,
    pub disabled: Option<Srgb>,
    pub selected: Option<Srgb>,
    pub selected_ink: Option<Srgb>,
    pub selected_border: Option<Srgb>,
    pub error: Option<Srgb>,
}

impl Picked {
    /// Nothing picked: every slot derives.
    pub const NONE: Self = Self {
        panel: None,
        card: None,
        help: None,
        accent_ink: None,
        border: None,
        card_border: None,
        button_border: None,
        viewport_border: None,
        button_bg: None,
        button_ink: None,
        hover: None,
        focus: None,
        disabled: None,
        selected: None,
        selected_ink: None,
        selected_border: None,
        error: None,
    };
}

/// Every colour the shared sheet names. One custom property each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// The window.
    pub background: Srgb,
    /// One elevation up: asides, status panels, control boxes.
    pub panel: Srgb,
    /// The scene card the viewport sits in.
    pub card: Srgb,

    /// Body text.
    pub ink: Srgb,
    /// Field names and the header line.
    pub muted: Srgb,
    /// The control-help line, which some products dim differently.
    pub help: Srgb,

    /// The chosen colour.
    pub accent: Srgb,
    /// Text over an accent fill.
    pub accent_ink: Srgb,

    /// Panel and box outlines.
    pub border: Srgb,
    /// The scene card's outline.
    pub card_border: Srgb,
    /// A button's outline.
    pub button_border: Srgb,
    /// The viewport's own frame.
    pub viewport_border: Srgb,

    /// A button's rest fill.
    pub button_bg: Srgb,
    /// A button's label.
    pub button_ink: Srgb,
    /// A button's hover fill.
    pub hover: Srgb,
    /// The focus outline.
    pub focus: Srgb,
    /// Inactive text.
    pub disabled: Srgb,

    /// A selected row's fill.
    pub selected: Srgb,
    /// A selected row's label.
    pub selected_ink: Srgb,
    /// A selected row's outline.
    pub selected_border: Srgb,

    /// The error line.
    pub error: Srgb,
    /// The caution hue.
    pub warning: Srgb,
    /// The affirmative hue.
    pub answer: Srgb,
}

/// Fill the palette from the seeds: picked slots verbatim, the rest through
/// `tinct`.
///
/// Derivation runs the surface ladder off `background` in OKLCH, so a light
/// seed set gets a light ladder and a dark one a dark ladder without the
/// product saying which it is; `best_on` picks the text over every fill.
pub fn derive(seeds: &Seeds) -> Palette {
    let dark = relative_luminance(seeds.background) < 0.5;
    let base = Oklch::from_srgb(seeds.background);
    let accent = seeds.accent;

    // The ladder: one perceptual step up from the window for panels, a second
    // for controls, and the opposite direction for the scene card so the card
    // reads as a hole rather than a shelf.
    let step = |dl: f64| base.lighten(if dark { dl } else { -dl }).to_srgb();
    let panel = seeds.picked.panel.unwrap_or_else(|| step(0.05));
    let button_bg = seeds.picked.button_bg.unwrap_or_else(|| step(0.09));
    let card = seeds.picked.card.unwrap_or_else(|| step(-0.04));

    // Outlines are the window blended toward the ink, so they track the theme
    // without anybody choosing a grey.
    let line = |t: f64| mix(seeds.background, seeds.ink, t);
    let border = seeds.picked.border.unwrap_or_else(|| line(0.30));
    let danger = Srgb::rgb(0xD5, 0x4E, 0x4E);
    let warning = seeds.warning.unwrap_or(Srgb::rgb(0xE0, 0xA8, 0x46));
    let answer = seeds.answer.unwrap_or(Srgb::rgb(0x4F, 0xB3, 0x6E));

    Palette {
        background: seeds.background,
        panel,
        card,

        ink: seeds.ink,
        muted: seeds.muted,
        help: seeds.picked.help.unwrap_or(seeds.muted),

        accent,
        accent_ink: seeds.picked.accent_ink.unwrap_or_else(|| best_on(accent)),

        border,
        card_border: seeds.picked.card_border.unwrap_or(border),
        button_border: seeds.picked.button_border.unwrap_or_else(|| line(0.36)),
        viewport_border: seeds
            .picked
            .viewport_border
            .unwrap_or_else(|| mix(accent, seeds.background, 0.45)),

        button_bg,
        button_ink: seeds.picked.button_ink.unwrap_or(seeds.ink),
        hover: seeds.picked.hover.unwrap_or_else(|| {
            // A hover is the rest fill carried toward the accent, far enough
            // to be seen and not so far that the label stops reading.
            mix(button_bg, accent, 0.18)
        }),
        focus: seeds.picked.focus.unwrap_or(accent),
        disabled: seeds
            .picked
            .disabled
            .unwrap_or_else(|| mix(seeds.ink, panel, 0.55)),

        selected: seeds.picked.selected.unwrap_or(accent),
        selected_ink: seeds
            .picked
            .selected_ink
            .unwrap_or_else(|| best_on(seeds.picked.selected.unwrap_or(accent))),
        selected_border: seeds.picked.selected_border.unwrap_or_else(|| {
            Oklch::from_srgb(accent)
                .lighten(if dark { 0.14 } else { -0.14 })
                .to_srgb()
        }),

        error: seeds.picked.error.unwrap_or(danger),
        warning,
        answer,
    }
}

/// Is the derived body text legible on the derived panel? A cheap gate a
/// product can assert in its own tests when it changes a seed.
pub fn text_contrast(palette: &Palette) -> f64 {
    contrast(palette.ink, palette.panel)
}

/// The palette as a `:root` block of `--isomere-*` custom properties, in a
/// fixed order so the emitted sheet is stable byte for byte.
pub fn css_vars(palette: &Palette) -> String {
    let rows: [(&str, Srgb); 23] = [
        ("background", palette.background),
        ("panel", palette.panel),
        ("card", palette.card),
        ("ink", palette.ink),
        ("muted", palette.muted),
        ("help", palette.help),
        ("accent", palette.accent),
        ("accent-ink", palette.accent_ink),
        ("border", palette.border),
        ("card-border", palette.card_border),
        ("button-border", palette.button_border),
        ("viewport-border", palette.viewport_border),
        ("button-bg", palette.button_bg),
        ("button-ink", palette.button_ink),
        ("hover", palette.hover),
        ("focus", palette.focus),
        ("disabled", palette.disabled),
        ("selected", palette.selected),
        ("selected-ink", palette.selected_ink),
        ("selected-border", palette.selected_border),
        ("error", palette.error),
        ("warning", palette.warning),
        ("answer", palette.answer),
    ];
    let mut out = String::from(":root {\n");
    for (name, colour) in rows {
        out.push_str("  --isomere-");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(&color_to_hex(colour));
        out.push_str(";\n");
    }
    out.push_str("}\n");
    out
}
