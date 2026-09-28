//! The side panel's text rows hold their text.
//!
//! A text row is any element under `.side` with a non-blank text node among
//! its own children. The measure of its text is the tallest painted box of
//! those text nodes, the fragment layout gives the run: no harness call
//! exposes glyph ink, so the run's fragment is the measure. Every row prints
//! its box and text heights, so one run reads the same at any pin; the test
//! calls only `board`, `painted_rect` and `LayoutDom` reads that stood at
//! `f15fd43`.

use super::*;

/// One text row as laid out: its classes, its text, its box height and the
/// tallest of its own text fragments.
struct TextRow {
    class: String,
    text: String,
    row: f32,
    text_height: f32,
}

/// Every text row under `.side`, in document order. Rows without a painted
/// box (a collapsed panel's) generate no box and are not rows.
fn text_rows(harness: &BoardHarness) -> Vec<TextRow> {
    let found = harness.with_dom(|dom| {
        let side = taproot::matching(dom, &Selector::class("side"))
            .first()
            .copied()
            .expect("the panel strip is in the retained tree");
        let mut found = Vec::new();
        let mut pending = vec![side];
        while let Some(node) = pending.pop() {
            let children: Vec<_> = dom.dom_children(node).collect();
            let texts: Vec<_> = children
                .iter()
                .copied()
                .filter(|&child| dom.text(child).is_some_and(|t| !t.trim().is_empty()))
                .collect();
            if !texts.is_empty() {
                let class = dom
                    .attribute(node, &Namespace::from(""), &LocalName::from("class"))
                    .unwrap_or_default()
                    .to_owned();
                let text: String = texts.iter().filter_map(|&t| dom.text(t)).collect();
                found.push((node, class, text, texts));
            }
            pending.extend(children.into_iter().rev());
        }
        found
    });
    found
        .into_iter()
        .filter_map(|(node, class, text, texts)| {
            let row = harness.painted_rect(node)?.3;
            let text_height = texts
                .into_iter()
                .filter_map(|t| harness.painted_rect(t))
                .map(|(_, _, _, h)| h)
                .fold(0.0_f32, f32::max);
            Some(TextRow {
                class,
                text: text.trim().chars().take(24).collect(),
                row,
                text_height,
            })
        })
        .collect()
}

/// Print every row of one state and return the ones shorter than their text.
fn short_rows(state: &str, harness: &BoardHarness) -> Vec<String> {
    let rows = text_rows(harness);
    assert!(!rows.is_empty(), "{state}: the panel lays out text rows");
    let mut short = Vec::new();
    for row in rows {
        let holds = row.row + 0.01 >= row.text_height;
        eprintln!(
            "TEXT-ROW {state:<9} box {:>5.1} text {:>5.1} {} .{} {:?}",
            row.row,
            row.text_height,
            if holds { "holds" } else { "SHORT" },
            row.class,
            row.text,
        );
        if !holds {
            short.push(format!(
                "{state} .{} {:?}: box {} < text {}",
                row.class, row.text, row.row, row.text_height
            ));
        }
    }
    short
}

/// Expanded at zoom 1, then the two transient states the panel diet measures.
#[test]
fn every_side_panel_text_row_holds_its_text() {
    let expanded = board(DESIGN_SIZE, false);
    assert_eq!(expanded.ui_zoom(), 1.0, "measured at zoom 1");
    let mut short = short_rows("expanded", &expanded);

    short.extend(short_rows("composing", &composing_a_whisper(DESIGN_SIZE)));

    let mut picking = board(DESIGN_SIZE, false);
    picking.update(|ui| ui.action_pick = Some((TokenId(1), "attack".to_owned())));
    picking.relayout();
    assert!(picking.state().picking_target());
    short.extend(short_rows("picking", &picking));

    assert!(
        short.is_empty(),
        "{} side panel text rows are shorter than their text:\n{}",
        short.len(),
        short.join("\n")
    );
}
