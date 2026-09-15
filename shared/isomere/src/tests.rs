// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M0's done-condition, as tests.
//!
//! The two seed sets below are **pinned fixtures**, not the products' own
//! definitions: they are the hex the Mesocosm bench sheet and the Paredros
//! session sheet carried on 2026-09-15, the day the sheet was promoted. Their
//! job is to fail if derivation ever stops reproducing that day's colours,
//! which is the condition the capture sets rest on. The products keep their
//! own copies beside their views.

use crate::{
    Palette, Picked, Seeds, Sizes, Srgb, color_from_hex, css_vars, derive, shared, sheet,
    sheet_with,
};

fn hex(s: &str) -> Srgb {
    color_from_hex(s).expect("fixture hex")
}

// =============================================================================
// The two seed sets, as of 2026-09-15
// =============================================================================

/// Mesocosm's specimen bench: a light terrarium palette.
fn bench_seeds() -> Seeds {
    Seeds {
        background: hex("#eeeae1"),
        ink: hex("#27332e"),
        muted: hex("#65736a"),
        accent: hex("#315c3e"),
        warning: None,
        answer: None,
        picked: Picked {
            panel: Some(hex("#faf8f2")),
            card: Some(hex("#dce3d7")),
            border: Some(hex("#c3cabc")),
            card_border: Some(hex("#b7c1b3")),
            button_border: Some(hex("#a6b3a5")),
            viewport_border: Some(hex("#6f8470")),
            button_bg: Some(hex("#faf8f2")),
            button_ink: Some(hex("#263d2e")),
            hover: Some(hex("#dfebda")),
            focus: Some(hex("#367f56")),
            accent_ink: Some(hex("#ffffff")),
            selected_ink: Some(hex("#ffffff")),
            // The bench never restyled a selected chip's outline, so it keeps
            // the button's own.
            selected_border: Some(hex("#a6b3a5")),
            // `#notice` inherits the body ink today.
            error: Some(hex("#27332e")),
            ..Picked::NONE
        },
    }
}

/// Paredros's session: a dark instrument palette.
fn session_seeds() -> Seeds {
    Seeds {
        background: hex("#1b1d21"),
        ink: hex("#dfe3e6"),
        muted: hex("#8e9aa2"),
        accent: hex("#2f6b46"),
        warning: None,
        answer: None,
        picked: Picked {
            panel: Some(hex("#23272c")),
            card: Some(hex("#000000")),
            help: Some(hex("#93a0a8")),
            border: Some(hex("#39424a")),
            card_border: Some(hex("#39424a")),
            button_border: Some(hex("#4a555e")),
            // The session draws no frame on the viewport; the width is zero,
            // so this colour never reaches a pixel.
            viewport_border: Some(hex("#39424a")),
            button_bg: Some(hex("#2e343a")),
            button_ink: Some(hex("#e4e9ec")),
            hover: Some(hex("#3a434b")),
            focus: Some(hex("#6fa8dc")),
            accent_ink: Some(hex("#e4e9ec")),
            selected_ink: Some(hex("#e4e9ec")),
            selected_border: Some(hex("#49a06a")),
            error: Some(hex("#e0a0a0")),
            ..Picked::NONE
        },
    }
}

/// Every colour the bench sheet printed on 2026-09-15, slot by slot.
fn bench_today() -> Palette {
    Palette {
        background: hex("#eeeae1"),
        panel: hex("#faf8f2"),
        card: hex("#dce3d7"),
        ink: hex("#27332e"),
        muted: hex("#65736a"),
        help: hex("#65736a"),
        accent: hex("#315c3e"),
        accent_ink: hex("#ffffff"),
        border: hex("#c3cabc"),
        card_border: hex("#b7c1b3"),
        button_border: hex("#a6b3a5"),
        viewport_border: hex("#6f8470"),
        button_bg: hex("#faf8f2"),
        button_ink: hex("#263d2e"),
        hover: hex("#dfebda"),
        focus: hex("#367f56"),
        disabled: derive(&bench_seeds()).disabled,
        selected: hex("#315c3e"),
        selected_ink: hex("#ffffff"),
        selected_border: hex("#a6b3a5"),
        error: hex("#27332e"),
        warning: derive(&bench_seeds()).warning,
        answer: derive(&bench_seeds()).answer,
    }
}

/// Every colour the session sheet printed on 2026-09-15, slot by slot.
fn session_today() -> Palette {
    Palette {
        background: hex("#1b1d21"),
        panel: hex("#23272c"),
        card: hex("#000000"),
        ink: hex("#dfe3e6"),
        muted: hex("#8e9aa2"),
        help: hex("#93a0a8"),
        accent: hex("#2f6b46"),
        accent_ink: hex("#e4e9ec"),
        border: hex("#39424a"),
        card_border: hex("#39424a"),
        button_border: hex("#4a555e"),
        viewport_border: hex("#39424a"),
        button_bg: hex("#2e343a"),
        button_ink: hex("#e4e9ec"),
        hover: hex("#3a434b"),
        focus: hex("#6fa8dc"),
        disabled: derive(&session_seeds()).disabled,
        selected: hex("#2f6b46"),
        selected_ink: hex("#e4e9ec"),
        selected_border: hex("#49a06a"),
        error: hex("#e0a0a0"),
        warning: derive(&session_seeds()).warning,
        answer: derive(&session_seeds()).answer,
    }
}

#[test]
fn bench_seeds_reproduce_todays_bench_hex() {
    assert_eq!(derive(&bench_seeds()), bench_today());
}

#[test]
fn session_seeds_reproduce_todays_session_hex() {
    assert_eq!(derive(&session_seeds()), session_today());
}

/// The point of the ruling: every hex literal that appeared in either sheet's
/// shared half must appear, spelled out, in that product's emitted palette
/// block. If tinct ever rounds a slot differently this fails with the colour
/// named, rather than quietly rebasing a capture set.
#[test]
fn every_hex_the_two_sheets_used_is_emitted() {
    let bench = css_vars(&derive(&bench_seeds()));
    for want in [
        "#EEEAE1", "#27332E", "#65736A", "#FAF8F2", "#DCE3D7", "#C3CABC", "#B7C1B3", "#A6B3A5",
        "#6F8470", "#263D2E", "#DFEBDA", "#367F56", "#315C3E", "#FFFFFF",
    ] {
        assert!(bench.contains(want), "bench palette lost {want}:\n{bench}");
    }
    let session = css_vars(&derive(&session_seeds()));
    for want in [
        "#1B1D21", "#DFE3E6", "#8E9AA2", "#93A0A8", "#23272C", "#000000", "#39424A", "#4A555E",
        "#2E343A", "#E4E9EC", "#3A434B", "#6FA8DC", "#2F6B46", "#49A06A", "#E0A0A0",
    ] {
        assert!(
            session.contains(want),
            "session palette lost {want}:\n{session}"
        );
    }
}

// =============================================================================
// Class vocabulary
// =============================================================================

/// The classes and ids the bench view and the session view both emit and the
/// shared sheet is answerable for. Read off the two `view.rs` files on
/// 2026-09-15.
const SHARED_VOCABULARY: &[&str] = &[
    ".scene-card",
    ".viewport",
    ".toolbar",
    "button",
    "button:hover",
    "button:focus",
    ".parts",
    ".part.selected",
    ".reading",
    ".field",
    ".field-name",
    ".field-value",
    "#controls-help",
    "#notice",
    "#viewport-error",
    "header",
    "h1",
    "h2",
];

#[test]
fn shared_rules_name_every_shared_class() {
    let rules = shared();
    for selector in SHARED_VOCABULARY {
        assert!(
            rules.contains(selector),
            "the shared sheet never names {selector}"
        );
    }
}

#[test]
fn both_products_sheets_carry_the_shared_vocabulary() {
    for (name, seeds) in [("bench", bench_seeds()), ("session", session_seeds())] {
        let css = sheet(&seeds, "");
        for selector in SHARED_VOCABULARY {
            assert!(css.contains(selector), "{name} sheet lost {selector}");
        }
    }
}

/// A product's own rules land after the shared ones, which is the only reason
/// a product override works at all.
#[test]
fn product_rules_follow_the_shared_rules() {
    let css = sheet(
        &bench_seeds(),
        ".scene-card.decorated { border-color:#846b49; }\n",
    );
    let shared_at = css.find(".scene-card {").expect("shared card rule");
    let product_at = css.find(".scene-card.decorated").expect("product rule");
    assert!(shared_at < product_at, "product rules came first");
}

// =============================================================================
// Stability
// =============================================================================

#[test]
fn css_vars_is_stable() {
    let got = css_vars(&derive(&bench_seeds()));
    let want = "\
:root {
  --isomere-background: #EEEAE1;
  --isomere-panel: #FAF8F2;
  --isomere-card: #DCE3D7;
  --isomere-ink: #27332E;
  --isomere-muted: #65736A;
  --isomere-help: #65736A;
  --isomere-accent: #315C3E;
  --isomere-accent-ink: #FFFFFF;
  --isomere-border: #C3CABC;
  --isomere-card-border: #B7C1B3;
  --isomere-button-border: #A6B3A5;
  --isomere-viewport-border: #6F8470;
  --isomere-button-bg: #FAF8F2;
  --isomere-button-ink: #263D2E;
  --isomere-hover: #DFEBDA;
  --isomere-focus: #367F56;
  --isomere-disabled: #9B9F9A;
  --isomere-selected: #315C3E;
  --isomere-selected-ink: #FFFFFF;
  --isomere-selected-border: #A6B3A5;
  --isomere-error: #27332E;
  --isomere-warning: #E0A846;
  --isomere-answer: #4FB36E;
}
";
    assert_eq!(got, want);
}

#[test]
fn css_vars_is_deterministic() {
    let a = css_vars(&derive(&session_seeds()));
    let b = css_vars(&derive(&session_seeds()));
    assert_eq!(a, b);
    assert_eq!(a.matches("--isomere-").count(), 23);
}

#[test]
fn sizes_reach_the_sheet() {
    let sizes = Sizes {
        viewport_height: "calc(100vh - 335px)",
        ..Sizes::DEFAULT
    };
    let css = sheet_with(&bench_seeds(), &sizes, "");
    assert!(css.contains("--isomere-viewport-height: calc(100vh - 335px);"));
    assert!(css.contains("height:var(--isomere-viewport-height);"));
}

/// Nothing in the emitted sheet may still be a bare hex: every colour the
/// shared half prints has to come through a property, or a product could not
/// reseed it.
#[test]
fn shared_rules_carry_no_literal_colours() {
    let rules = shared();
    for line in rules.lines() {
        let decls = line.split_once('{').map(|(_, d)| d).unwrap_or("");
        assert!(
            !decls.contains('#'),
            "a shared rule still spells a literal colour: {line}"
        );
    }
}

/// Seeding nothing but the four anchors still yields a legible palette — the
/// path a product takes when it has no hand-picked states to preserve.
#[test]
fn bare_anchors_derive_a_legible_palette() {
    for seeds in [
        Seeds::anchors(
            hex("#eeeae1"),
            hex("#27332e"),
            hex("#65736a"),
            hex("#315c3e"),
        ),
        Seeds::anchors(
            hex("#1b1d21"),
            hex("#dfe3e6"),
            hex("#8e9aa2"),
            hex("#2f6b46"),
        ),
    ] {
        let p = derive(&seeds);
        assert!(crate::text_contrast(&p) >= 4.5, "{p:?}");
        assert!(crate::contrast(p.accent_ink, p.accent) >= 3.0, "{p:?}");
        assert_ne!(p.hover, p.button_bg, "hover indistinguishable from rest");
    }
}

// =============================================================================
// M1: the viewport card and the error line
// =============================================================================

use crate::viewport::{
    CARD_CLASS, ERROR_CLASS, LEAF_CLASS, ViewportCard, error_attrs, viewport_card,
};

/// The two cards as the products spell them on 2026-09-15: pinned fixtures,
/// like the seed sets above, so a drift in what the card emits fails here
/// before it fails a scenario.
fn bench_card<'a>(class: &'a str, style: &'a str) -> ViewportCard<'a> {
    ViewportCard {
        id: Some("specimen-viewport"),
        class: Some(class),
        style: Some(style),
        card_class: Some("scene-card decorated"),
        ..ViewportCard::new(7001, (640, 400), "Specimen")
    }
}

fn session_card(description: &str) -> ViewportCard<'_> {
    ViewportCard {
        id: Some("scene-viewport"),
        description: Some(description),
        ..ViewportCard::new(9001, (720, 440), "Scene")
    }
}

fn attr<'a>(attrs: &'a [(&'static str, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v.as_str())
}

/// The leaf carries the key the host registered it under, and the box the host
/// painted, as `custom_leaf` would have written them.
#[test]
fn the_leaf_carries_its_key_and_box() {
    let attrs = session_card("nothing selected").leaf_attrs();
    assert_eq!(attr(&attrs, "key"), Some("9001"));
    assert_eq!(
        attr(&attrs, "style"),
        Some("display:block;width:720px;height:440px")
    );
    assert_eq!(attrs.first().map(|(n, _)| *n), Some("key"), "{attrs:?}");
}

/// The two attributes both acceptance scenarios select the viewport by
/// (`Selector::role("img").containing(..)`). If either moves, the pixel lanes
/// stop finding the viewport at all.
#[test]
fn the_leaf_is_an_image_with_the_products_label() {
    for (card, label) in [
        (bench_card("viewport", ""), "Specimen"),
        (session_card("nothing selected"), "Scene"),
    ] {
        let attrs = card.leaf_attrs();
        assert_eq!(attr(&attrs, "role"), Some("img"));
        assert_eq!(attr(&attrs, "aria-label"), Some(label));
    }
}

/// Classes default to the shared vocabulary and a product replaces exactly the
/// one it named. The bench's tints and its decorated frame are the only two
/// callers that do.
#[test]
fn classes_default_to_the_shared_vocabulary() {
    let plain = session_card("");
    assert_eq!(plain.leaf_class(), LEAF_CLASS);
    assert_eq!(plain.container_class(), CARD_CLASS);
    assert_eq!(attr(&plain.leaf_attrs(), "class"), Some("viewport"));

    let tinted = bench_card("viewport warm", "");
    assert_eq!(attr(&tinted.leaf_attrs(), "class"), Some("viewport warm"));
    assert_eq!(tinted.container_class(), "scene-card decorated");
}

/// A `style` replaces the leaf's box in the leaf's own attribute slot, empty
/// string included — which is what the bench wrote by hand, and what its
/// transform demonstration depends on.
#[test]
fn a_style_replaces_the_leaf_box() {
    let erased = bench_card("viewport", "").leaf_attrs();
    assert_eq!(attr(&erased, "style"), Some(""));
    assert_eq!(erased[1].0, "style", "the override keeps the leaf's slot");

    let transformed = bench_card("viewport", "transform:rotate(7deg) scale(0.9);").leaf_attrs();
    assert_eq!(
        attr(&transformed, "style"),
        Some("transform:rotate(7deg) scale(0.9);")
    );
}

/// The description rides only when a product publishes one; the session does,
/// the bench does not.
#[test]
fn the_description_is_the_products_to_publish() {
    assert_eq!(
        attr(&session_card("Part 2").leaf_attrs(), "aria-description"),
        Some("Part 2")
    );
    assert_eq!(
        attr(&bench_card("viewport", "").leaf_attrs(), "aria-description"),
        None
    );
}

/// The error line keeps the product's id — both scenarios read it by id — and
/// adds the shared class the sheet styles all three selectors by.
#[test]
fn the_error_line_keeps_its_id_and_gains_the_shared_class() {
    for id in ["notice", "viewport-error"] {
        let attrs = error_attrs(id);
        assert_eq!(attr(&attrs, "id"), Some(id));
        assert_eq!(attr(&attrs, "class"), Some(ERROR_CLASS));
        assert_eq!(attr(&attrs, "role"), Some("status"), "it is a live region");
        assert!(
            shared().contains(&format!("#{id}")),
            "the shared sheet never styles #{id}"
        );
    }
    assert!(shared().contains(ERROR_CLASS));
}

/// The card builds for a product with its own state and no action type, which
/// is how both products' `Child` aliases are spelled. A compile-level check:
/// if the generic parameters drift, this stops building.
#[test]
fn the_card_builds_for_a_products_own_state() {
    struct Product {
        picked: u32,
    }
    let card = session_card("");
    let _: crate::viewport::Child<Product, ()> = viewport_card(
        &card,
        Some(Box::new(crate::viewport::error_line::<Product, ()>(
            "overlay",
            "over the scene",
        ))),
        Some(|state: &mut Product, _event: cambium::PointerEvent| state.picked += 1),
    );
}
