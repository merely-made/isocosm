// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M1: the viewport card and the error line, as tests.

use super::*;
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
