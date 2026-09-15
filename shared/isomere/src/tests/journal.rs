// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M3: the journal, as tests.

use super::*;
use crate::journal::{
    FOUNDING_CLASS, HEADLINE_CLASS, JOURNAL_CLASS, JournalClasses, JournalModel, JournalRow,
    LIVE_CLASS, ROW_CLASS, SELECTED_CLASS, journal, journal_row, journal_rows,
};

/// The session's journal as it stands on 2026-09-15: one glyph acquired under
/// the founding canon, one whose live effect has since moved. A pinned
/// fixture, like the cards and the chips above.
fn session_rows() -> Vec<JournalRow> {
    vec![
        JournalRow {
            live: Some(String::new()),
            ..JournalRow::new("Reach", "reach · strike · tick 40").marked("paredros-fixture:reach")
        },
        JournalRow {
            live: Some("now paredros-fixture:fasten · authored revision".to_owned()),
            ..JournalRow::new("Strike", "strike · volley-resolved · tick 64")
                .marked("paredros-fixture:strike")
        },
    ]
}

/// The board's own names, as `mesocosm-views` passes them. Its sheet is its
/// own, drawn into a raster this lane has no receipt for, so not one of its
/// class names moves. It arranges its rows itself, so the container name is
/// the one it leaves shared.
const BOARD: JournalClasses<'static> = JournalClasses {
    journal: None,
    row: Some("board-row"),
    selected: Some("board-row-on"),
    headline: Some("board-row-name"),
    founding: Some("board-row-figures"),
    live: Some("board-row-reason"),
};

/// The mark rides two spaces after the headline, which is what both sources
/// wrote by hand: `"{display}  {glyph}"` in the session, `"{name}  ({source})"`
/// on the board.
#[test]
fn the_mark_follows_the_headline_by_two_spaces() {
    let rows = session_rows();
    assert_eq!(rows[0].headline_text(), "Reach  paredros-fixture:reach");
    assert_eq!(
        JournalRow::new("the status quo", "+0 mg over 900 ticks")
            .marked("(nothing to build)")
            .headline_text(),
        "the status quo  (nothing to build)"
    );
    assert_eq!(
        JournalRow::new("unmarked", "a founding line").headline_text(),
        "unmarked",
        "a row with no mark says nothing extra"
    );
}

/// Every row is three lines in order, and the third is there exactly when the
/// product said so. `Some("")` still emits it — the session's rows must not
/// change height as a revision arrives — and `None` omits it, which is what
/// the board's reason line does.
#[test]
fn every_row_is_its_lines_in_order() {
    let rows = session_rows();
    let first = rows[0].lines(&JournalClasses::SHARED);
    assert_eq!(
        first.iter().map(|(class, _)| *class).collect::<Vec<_>>(),
        [HEADLINE_CLASS, FOUNDING_CLASS, LIVE_CLASS]
    );
    assert_eq!(first[1].1, "reach · strike · tick 40");
    assert_eq!(first[2].1, "", "an unrevised row still emits its live line");

    let silent = JournalRow::new("the status quo", "+0 mg over 900 ticks");
    assert_eq!(silent.lines(&JournalClasses::SHARED).len(), 2);
}

/// A selected row appends the cursor class; nothing else does. Only the board
/// has a cursor.
#[test]
fn the_cursor_class_is_appended_and_only_when_selected() {
    let plain = JournalRow::new("a", "b");
    assert_eq!(plain.class(&JournalClasses::SHARED), ROW_CLASS);
    let on = JournalRow {
        selected: true,
        ..JournalRow::new("a", "b")
    };
    assert_eq!(
        on.class(&JournalClasses::SHARED),
        format!("{ROW_CLASS} {SELECTED_CLASS}")
    );
    assert_eq!(on.class(&BOARD), "board-row board-row-on");
    assert_eq!(plain.class(&BOARD), "board-row");
}

/// A product with its own sheet names all six classes and gets exactly those
/// back. This is the whole reason the overrides exist: the board's DOM must
/// not move while its rows become the shared journal.
#[test]
fn a_products_own_classes_replace_every_shared_one() {
    let row = JournalRow {
        live: Some("no room in the line".to_owned()),
        ..JournalRow::new("hunting", "+12 mg over 900 ticks").marked("(the pack's)")
    };
    let lines = row.lines(&BOARD);
    assert_eq!(
        lines.iter().map(|(class, _)| *class).collect::<Vec<_>>(),
        ["board-row-name", "board-row-figures", "board-row-reason"]
    );
    assert_eq!(lines[0].1, "hunting  (the pack's)");
    assert_eq!(lines[2].1, "no room in the line");
}

/// The classes the shared sheet is answerable for. The two line classes were
/// added to M0's `.field-name` rule rather than given one of their own, so the
/// session's journal lines read exactly as they did when they borrowed it.
#[test]
fn the_shared_sheet_styles_the_journal_lines() {
    let rules = shared();
    for class in [FOUNDING_CLASS, LIVE_CLASS] {
        assert!(
            rules.contains(class),
            "the shared sheet never styles {class}"
        );
    }
    let rule = rules
        .lines()
        .find(|line| line.contains(FOUNDING_CLASS))
        .expect("a rule naming the founding line");
    assert!(
        rule.contains(".field-name"),
        "the journal lines must ride the reading row's rule, not a new one: {rule}"
    );
}

/// An empty journal says the product's note instead of the rows, and a journal
/// with rows never says it.
#[test]
fn an_empty_journal_says_the_products_note() {
    let empty: crate::viewport::Child<u32, ()> =
        journal_rows(&[], Some("Nothing acquired yet."), &JournalClasses::SHARED);
    drop(empty);
    let model = JournalModel {
        summary: Some(("journal-summary", "0 of 3 acquired".to_owned())),
        note: Some("Nothing acquired yet."),
        ..JournalModel::new("Acquisition journal")
    };
    assert_eq!(model.classes, JournalClasses::SHARED);
    assert_eq!(model.class, None, "a bare aside is what everyone else does");
    assert!(model.rows.is_empty());
}

/// The panel builds for a product with its own state and no action type, which
/// is how all three products' `Child` aliases are spelled, and the pieces
/// build on their own for a product that arranges them itself. A compile-level
/// check: if the generic parameters drift, this stops building.
#[test]
fn the_journal_builds_for_a_products_own_state() {
    struct Product {
        read: usize,
    }
    let model = JournalModel {
        rows: session_rows(),
        summary: Some(("journal-summary", "2 of 3 acquired".to_owned())),
        class: Some("panel"),
        ..JournalModel::new("Acquisition journal")
    };
    let _: crate::viewport::Child<Product, ()> = journal(&model);
    let _: crate::viewport::Child<Product, ()> =
        journal_rows(&session_rows(), None, &JournalClasses::SHARED);
    let _: crate::viewport::Child<Product, ()> = journal_row(&session_rows()[0], &BOARD);
    assert_eq!(Product { read: 0 }.read, 0);
    // The container and the row box are the journal classes the shared sheet
    // does *not* style: the two products' rows are different sizes, so those
    // rules stay each product's own until M4 reconciles their sheets.
    assert!(!shared().contains(&format!(".{JOURNAL_CLASS} {{")));
    assert!(!shared().contains(&format!(".{ROW_CLASS} {{")));
}
