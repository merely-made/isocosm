// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M2: the examiner, as tests.

use super::*;
use crate::examiner::{
    ExaminerModel, ExaminerRow, FIELD_CLASS, FIELD_NAME_CLASS, FIELD_VALUE_CLASS, PART_CLASS,
    PARTS_CLASS, READING_CLASS, examiner, parts_palette, reading_column,
};

/// The session's palette as it stands on 2026-09-15: one intact part, one
/// selected, one severed. A pinned fixture, like the cards above.
fn session_rows() -> Vec<ExaminerRow<'static>> {
    vec![
        ExaminerRow::new(1, "torso (1)"),
        ExaminerRow {
            selected: true,
            ..ExaminerRow::new(3, "gripping limb C (3)")
        },
        ExaminerRow {
            state_class: Some("severed"),
            ..ExaminerRow::new(4, "gripping limb D (4) severed")
        },
    ]
}

/// Every row reaches the palette as a button, in order, carrying the label the
/// scenario clicks it by.
#[test]
fn every_row_is_a_button_with_its_label_in_order() {
    let rows = session_rows();
    let labels: Vec<String> = rows
        .iter()
        .map(|row| {
            let attrs = row.button_attrs();
            assert_eq!(attrs.first().map(|(name, _)| *name), Some("class"));
            attr(&attrs, "aria-label")
                .expect("every chip is labelled")
                .to_owned()
        })
        .collect();
    assert_eq!(
        labels,
        [
            "torso (1)",
            "gripping limb C (3)",
            "gripping limb D (4) severed"
        ]
    );
    let ids: Vec<u64> = rows.iter().map(|row| row.id).collect();
    assert_eq!(ids, [1, 3, 4], "the palette keeps the product's order");
}

/// Selection is published as `aria-pressed` on every chip, pressed or not —
/// the bench already did this, the session did not, and the shared chip does.
#[test]
fn selection_is_published_as_aria_pressed() {
    let rows = session_rows();
    let pressed: Vec<String> = rows
        .iter()
        .map(|row| {
            attr(&row.button_attrs(), "aria-pressed")
                .expect("a pressed state")
                .to_owned()
        })
        .collect();
    assert_eq!(pressed, ["false", "true", "false"]);
}

/// The classes are the ones the M0 sheet styles and the two panels wrote by
/// hand: a bare chip, the selected chip, and a state class that reads over
/// selection rather than joining it.
#[test]
fn the_chip_classes_are_the_sheets_own() {
    assert_eq!(ExaminerRow::new(0, "Part 0").class(), "part");
    let rows = session_rows();
    assert_eq!(rows[1].class(), "part selected");
    assert_eq!(rows[2].class(), "part severed");
    let severed_and_selected = ExaminerRow {
        state_class: Some("severed"),
        selected: true,
        ..ExaminerRow::new(4, "gripping limb D (4) severed")
    };
    assert_eq!(
        severed_and_selected.class(),
        "part severed",
        "a condition reads over selection, as the session wrote it"
    );
    assert_eq!(
        attr(&severed_and_selected.button_attrs(), "aria-pressed"),
        Some("true"),
        "and the pressed state still tells the truth"
    );
    for class in [PART_CLASS, PARTS_CLASS, READING_CLASS, FIELD_CLASS] {
        assert!(
            shared().contains(class),
            "the shared sheet never styles .{class}"
        );
    }
    assert!(shared().contains(".part.selected"));
    assert!(shared().contains(FIELD_NAME_CLASS));
    assert!(shared().contains(FIELD_VALUE_CLASS));
}

/// Every reading reaches the column as a name over a value, in the order the
/// product read them. The session's own seven rows are the fixture.
#[test]
fn every_reading_is_a_name_value_pair_in_order() {
    let readings: Vec<(String, String)> = [
        ("Subject", "701"),
        ("Anatomy revision", "4"),
        ("Learned techniques", "0"),
        ("Intact parts", "5 of 6"),
        ("Selected part", "gripping limb C"),
        ("Attached to", "part 1"),
        ("Bounds", "[0, 0, 0] .. [2, 2, 2]"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    let cells: Vec<[(&'static str, String); 3]> = readings
        .iter()
        .map(|(name, value)| crate::examiner::field_cells(name, value))
        .collect();
    assert_eq!(cells.len(), readings.len(), "no row is dropped or invented");
    for (cell, (name, value)) in cells.iter().zip(&readings) {
        assert_eq!(cell[0].0, FIELD_CLASS, "the row is a .field block");
        assert_eq!(
            (cell[1].0, cell[1].1.as_str()),
            (FIELD_NAME_CLASS, name.as_str())
        );
        assert_eq!(
            (cell[2].0, cell[2].1.as_str()),
            (FIELD_VALUE_CLASS, value.as_str()),
            "the value follows its name, never the other way round"
        );
    }
    let names: Vec<&str> = cells.iter().map(|cell| cell[1].1.as_str()).collect();
    assert_eq!(names.first(), Some(&"Subject"));
    assert_eq!(names.last(), Some(&"Bounds"));
}

/// A disabled row is inert: no product sets it today, so this is the only
/// place the branch is exercised.
#[test]
fn a_disabled_row_says_so() {
    let row = ExaminerRow {
        disabled: true,
        ..ExaminerRow::new(9, "stump (9)")
    };
    assert_eq!(attr(&row.button_attrs(), "aria-disabled"), Some("true"));
    assert_eq!(
        attr(
            &ExaminerRow::new(9, "stump (9)").button_attrs(),
            "aria-disabled"
        ),
        None
    );
}

/// The model defaults to what everyone else does: no container class, no note,
/// no schematic.
#[test]
fn the_model_defaults_to_the_shared_shape() {
    let model: ExaminerModel<'_, u32, ()> = ExaminerModel::new("Subject sheet");
    assert_eq!(model.title, "Subject sheet");
    assert_eq!(model.class, None);
    assert_eq!(model.note, None);
    assert!(model.rows.is_empty() && model.readings.is_empty());
    assert!(model.schematic.is_none());
}

/// The panel builds for a product with its own state and no action type, which
/// is how both products' `Child` aliases are spelled, and the pieces build on
/// their own for a product that arranges them itself. A compile-level check: if
/// the generic parameters drift, this stops building.
#[test]
fn the_examiner_builds_for_a_products_own_state() {
    struct Product {
        picked: u64,
    }
    let readings = vec![
        ("Subject".to_owned(), "701".to_owned()),
        ("Anatomy revision".to_owned(), "4".to_owned()),
    ];
    let model = ExaminerModel {
        rows: session_rows(),
        readings: readings.clone(),
        class: Some("panel"),
        note: Some("Choose a visible part, or use the part buttons."),
        schematic: Some(crate::viewport::error_line::<Product, ()>(
            "schematic",
            "a drawing",
        )),
        ..ExaminerModel::new("Subject sheet")
    };
    let _: crate::viewport::Child<Product, ()> =
        examiner(model, |state: &mut Product, id| state.picked = id);
    let _: crate::viewport::Child<Product, ()> =
        parts_palette(&session_rows(), |state: &mut Product, id| state.picked = id);
    let _: crate::viewport::Child<Product, ()> = reading_column(&readings, None, None);
}
