// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The parts examiner: an addressable palette of parts and the reading beside
//! it (M2 of the isomere plan).
//!
//! The 2026-09-15 inventory found this panel three times. Two of them are the
//! same panel: the Paredros session's subject sheet and the Mesocosm bench's
//! parts examiner are both a heading, a `.parts` palette of focusable
//! `button.part` chips with a selected and a state class, and a `.reading`
//! column of `.field` / `.field-name` / `.field-value` rows. Isometry's
//! `sheet.rs` shares only the word: it is a character-sheet overlay of
//! `.sheet-row` lines with no palette and no addressable part, so it is not a
//! consumer and did not move.
//!
//! **Why not `cambium::detail_panel`.** Its rows are the right *idea* — inert
//! key/value pairs — but it emits `.detail-row` with inline `span` children,
//! and the shared M0 sheet styles `.field` blocks. Retargeting onto it would
//! restyle both products' reading columns, which is a pixel change M2 does not
//! get to make. The classes here are the ones the two scenarios and the sheet
//! already agreed on, so promotion is a move and not a rename. If Cambium's
//! `.detail-*` vocabulary and this one are ever reconciled, it is the sheet
//! that reconciles them, not this module.
//!
//! **What crosses the boundary is an id and a label.** A row carries a `u64`,
//! the number the product's own part identity wraps; no product type reaches
//! this crate, and the selection handler is the product's own.

use std::rc::Rc;

use cambium::{OptionalAction, clickable, el, focusable, text};

use crate::viewport::Child;

/// The palette's container class.
pub const PARTS_CLASS: &str = "parts";

/// A chip's base class. A state class is appended to it.
pub const PART_CLASS: &str = "part";

/// The reading column's container class.
pub const READING_CLASS: &str = "reading";

/// One reading row's class.
pub const FIELD_CLASS: &str = "field";

/// The name half of a [`FIELD_CLASS`] row.
pub const FIELD_NAME_CLASS: &str = "field-name";

/// The value half of a [`FIELD_CLASS`] row.
pub const FIELD_VALUE_CLASS: &str = "field-value";

/// One addressable part in the palette.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExaminerRow<'a> {
    /// What the product's own part identity wraps. Handed back to `on_select`
    /// and nothing else; this crate never interprets it.
    pub id: u64,
    /// The chip's text and its `aria-label`. Both acceptance scenarios click a
    /// part by this string, so it is the product's word, never a derived one.
    pub label: String,
    /// A condition class appended to [`PART_CLASS`]. The session's `severed`
    /// is the only one today.
    pub state_class: Option<&'a str>,
    /// Whether this is the selected part. Always published as `aria-pressed`.
    pub selected: bool,
    /// Whether the chip refuses the click and leaves Tab traversal. No product
    /// sets it today: the session's severed parts stay selectable.
    pub disabled: bool,
}

impl<'a> ExaminerRow<'a> {
    /// A plain, selectable, unselected chip.
    pub fn new(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state_class: None,
            selected: false,
            disabled: false,
        }
    }

    /// The chip's effective class.
    ///
    /// A state class *replaces* the selected class rather than joining it,
    /// because that is what the session wrote by hand: a severed part reads as
    /// severed even while it is the selected one. `aria-pressed` still tells
    /// the truth about selection either way.
    pub fn class(&self) -> String {
        match (self.state_class, self.selected) {
            (Some(state), _) => format!("{PART_CLASS} {state}"),
            (None, true) => format!("{PART_CLASS} selected"),
            (None, false) => PART_CLASS.to_owned(),
        }
    }

    /// Every attribute the chip carries, in the order it carries them.
    pub fn button_attrs(&self) -> Vec<(&'static str, String)> {
        let mut attrs = vec![
            ("class", self.class()),
            ("aria-label", self.label.clone()),
            ("aria-pressed", self.selected.to_string()),
        ];
        if self.disabled {
            attrs.push(("aria-disabled", "true".to_owned()));
        }
        attrs
    }
}

/// What a product has to say about its examiner.
///
/// `title`, `rows` and `readings` are the panel; the last three are the places
/// the two sources genuinely differ, in the spirit of
/// [`ViewportCard`](crate::ViewportCard): `None` is what everyone else does.
pub struct ExaminerModel<'a, State, Action> {
    /// The panel's heading.
    pub title: &'a str,
    /// The palette, in the order the product reads its parts.
    pub rows: Vec<ExaminerRow<'a>>,
    /// The reading column, as name-value pairs in order.
    pub readings: Vec<(String, String)>,
    /// A slot under the reading for a product's own drawing of the selected
    /// part. No product fills it yet. M5 was expected to bring Paredros's
    /// body-sheet schematic here and did not: that schematic is drawn into a
    /// netrender scene with stroked paths, so moving it into a DOM slot is a
    /// rewrite in another medium rather than a move, and it was retired with
    /// the rest of the body sheet instead. The slot stands for whoever draws
    /// one next.
    pub schematic: Option<Child<State, Action>>,
    /// The container's `class`. `None` is no class at all, which is what the
    /// bench's bare `aside` carries; the session passes `panel`.
    pub class: Option<&'a str>,
    /// A plain paragraph after the reading rows, for a product with nothing
    /// selected to read. The bench's "Choose a visible part" line is the only
    /// caller. Whether it shows is the product's judgement rather than "no
    /// rows", because the bench can carry a generation-change row above it
    /// with no reading under it.
    pub note: Option<&'a str>,
}

impl<'a, State, Action> ExaminerModel<'a, State, Action> {
    /// An examiner titled `title` with nothing in it yet.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            rows: Vec::new(),
            readings: Vec::new(),
            schematic: None,
            class: None,
            note: None,
        }
    }
}

/// One chip: a focusable, clickable `button.part` that hands its id back.
///
/// A disabled row emits the same button without the click and without the
/// focus wrapper, so it is inert rather than silently swallowing a press.
fn chip<State, Action, F, OA>(row: &ExaminerRow<'_>, on_select: Rc<F>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, u64) -> OA + 'static,
{
    let id = row.id;
    let mut button = el("button", text(row.label.clone()));
    for (name, value) in row.button_attrs() {
        button = button.attr(name, value);
    }
    if row.disabled {
        return Box::new(button);
    }
    Box::new(focusable(clickable(button, move |state: &mut State, _| {
        on_select(state, id)
    })))
}

/// The three elements a reading row is made of, as class and text, outermost
/// first.
///
/// [`field`] emits exactly this, so a test reads the row the emitter builds
/// rather than a restatement of it — the same reason
/// [`ViewportCard::leaf_attrs`](crate::ViewportCard::leaf_attrs) exists.
pub fn field_cells(name: &str, value: &str) -> [(&'static str, String); 3] {
    [
        (FIELD_CLASS, String::new()),
        (FIELD_NAME_CLASS, name.to_owned()),
        (FIELD_VALUE_CLASS, value.to_owned()),
    ]
}

/// One reading row: a `.field` block of a name over a value.
pub fn field<State, Action>(name: &str, value: &str) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let [(block, _), (name_class, name), (value_class, value)] = field_cells(name, value);
    Box::new(
        el(
            "div",
            (
                el("div", text(name)).attr("class", name_class),
                el("div", text(value)).attr("class", value_class),
            ),
        )
        .attr("class", block),
    )
}

/// The palette alone: `.parts` over one chip per row, in order.
pub fn parts_palette<State, Action, F, OA>(
    rows: &[ExaminerRow<'_>],
    on_select: F,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, u64) -> OA + 'static,
{
    let on_select = Rc::new(on_select);
    let chips: Vec<Child<State, Action>> = rows
        .iter()
        .map(|row| chip(row, Rc::clone(&on_select)))
        .collect();
    Box::new(el("div", chips).attr("class", PARTS_CLASS))
}

/// The reading column alone: `.reading` over the name-value rows, then the
/// note, then the schematic slot.
pub fn reading_column<State, Action>(
    readings: &[(String, String)],
    note: Option<&str>,
    schematic: Option<Child<State, Action>>,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut children: Vec<Child<State, Action>> = readings
        .iter()
        .map(|(name, value)| field(name, value))
        .collect();
    if let Some(note) = note {
        children.push(Box::new(el("p", text(note.to_owned()))));
    }
    children.extend(schematic);
    Box::new(el("div", children).attr("class", READING_CLASS))
}

/// The whole panel: an `aside` carrying the heading, the palette and the
/// reading column.
///
/// `on_select` is the product's, taking the product's own state and the id of
/// the row that was clicked, exactly as the two hand-written panels did.
pub fn examiner<State, Action, F, OA>(
    model: ExaminerModel<'_, State, Action>,
    on_select: F,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, u64) -> OA + 'static,
{
    let ExaminerModel {
        title,
        rows,
        readings,
        schematic,
        class,
        note,
    } = model;
    let children: Vec<Child<State, Action>> = vec![
        Box::new(el("h2", text(title.to_owned()))),
        parts_palette(&rows, on_select),
        reading_column(&readings, note, schematic),
    ];
    let panel = el("aside", children);
    Box::new(match class {
        Some(class) => panel.attr("class", class),
        None => panel,
    })
}
