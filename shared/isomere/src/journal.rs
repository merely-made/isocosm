// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The journal: ordered rows of what happened, and the summary over them
//! (M3 of the isomere plan).
//!
//! The 2026-09-15 inventory found this panel three times under three names:
//! Eponym's acquisition journal (`.glyphs` of `.glyph` rows under
//! `#journal-summary`), Isometry's message log, and Mesocosm's review rows.
//! Two of them turned out to be the same three lines in the same order — a
//! headline with a mark beside it, a founding line that never moves, and a
//! line that only shows when the live reading has diverged from the founding
//! one:
//!
//! | | Eponym | Mesocosm's board |
//! | --- | --- | --- |
//! | headline | the canon's display mark | the candidate's name |
//! | mark | the glyph id | the proposal sources |
//! | founding | effect · kind · tick | net / price / preview |
//! | live | `now <effect> · <cause>` | why it cannot be taken |
//!
//! So [`JournalRow`] is those four, plus the board's selection cursor, which
//! the session has no use for and leaves `false`.
//!
//! **Isometry's message log is not the third.** It is five flat `.roll-line`
//! divs sharing the dice log's own rule — one sentence each, no mark, no
//! founding line, no live line and no summary — so it is the same shape as
//! `UiState::rolls`, not the same shape as the glyph journal. Promoting it
//! would nest a div inside every line for no reconciliation, so §1's journal
//! row is corrected for Isometry as the viewport row was for the overmap in M1
//! and the examiner row for `sheet.rs` in M2. What Isometry did bring to M3 is
//! its keymap; see [`status`](crate::status).
//!
//! **Why the classes are overridable and the examiner's were not.** The two
//! sources are styled by *different sheets*: the session is styled by M0's
//! shared sheet, the board by its own `board_css` drawn into a netrender
//! raster, which is a surface M4 and M6 own and this lane has no receipt for.
//! [`JournalClasses`] is therefore the same `None` means "what everyone else
//! does" idiom [`ViewportCard`](crate::ViewportCard) uses, with the board
//! supplying its own six names so not one of its pixels moves. When the board
//! joins the shared sheet those overrides go and this comment with them.
//!
//! **Why not `cambium::sectioned_list`.** It was read and not adopted, for the
//! reason `detail_panel` was not adopted in M2: its rows are one string each,
//! styled `.list-row`, and every plain row is focusable and activatable. A
//! journal row is three lines and inert, and making the session's rows
//! focusable would change its Tab order — behaviour M3 does not get to change.
//! `cambium::detail_panel` is the same answer as before. If Cambium's list
//! vocabulary and this one are ever reconciled, it is the sheet that
//! reconciles them.
//!
//! **What the M0 sheet already carried, and what M3 added to it.** The sheet
//! carried the help line (`#controls-help, .controls-help`) and the `.field-*`
//! reading rows the session's journal lines were borrowing; it carried no
//! journal-row rule and no status-line rule, so the plan's §2.2 note is
//! corrected there. M3 added [`FOUNDING_CLASS`] and [`LIVE_CLASS`] to the
//! sheet's existing `.field-name` rule — a pure addition that changes no
//! existing selector, so the session's journal lines read exactly as they did
//! while no longer borrowing a reading row's name.
//!
//! **What crosses the boundary is a string.** No product type reaches this
//! crate: a row is four pieces of text a product already had in words.

use crate::viewport::Child;
use cambium::{el, text};

/// The rows container's class.
pub const JOURNAL_CLASS: &str = "journal";

/// One row's base class.
pub const ROW_CLASS: &str = "journal-row";

/// Appended to [`ROW_CLASS`] when the cursor is on the row.
pub const SELECTED_CLASS: &str = "journal-row-on";

/// The headline line's class.
pub const HEADLINE_CLASS: &str = "journal-headline";

/// The founding line's class. Styled with [`LIVE_CLASS`] by the shared sheet's
/// `.field-name` rule, which is what the session's journal lines already read
/// as.
pub const FOUNDING_CLASS: &str = "journal-founding";

/// The live line's class. See [`FOUNDING_CLASS`].
pub const LIVE_CLASS: &str = "journal-live";

/// The six class names a journal emits, each `None` for the shared one.
///
/// A product that is styled by the shared sheet leaves every field `None`.
/// Mesocosm's trait board, which has its own sheet and its own raster, names
/// all six.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JournalClasses<'a> {
    /// The rows container. `None` is [`JOURNAL_CLASS`].
    pub journal: Option<&'a str>,
    /// A row. `None` is [`ROW_CLASS`].
    pub row: Option<&'a str>,
    /// What a selected row appends. `None` is [`SELECTED_CLASS`].
    pub selected: Option<&'a str>,
    /// The headline line. `None` is [`HEADLINE_CLASS`].
    pub headline: Option<&'a str>,
    /// The founding line. `None` is [`FOUNDING_CLASS`].
    pub founding: Option<&'a str>,
    /// The live line. `None` is [`LIVE_CLASS`].
    pub live: Option<&'a str>,
}

impl<'a> JournalClasses<'a> {
    /// Every class the shared sheet styles. The ordinary case.
    pub const SHARED: Self = Self {
        journal: None,
        row: None,
        selected: None,
        headline: None,
        founding: None,
        live: None,
    };

    fn journal_class(&self) -> &'a str {
        self.journal.unwrap_or(JOURNAL_CLASS)
    }

    fn row_class(&self) -> &'a str {
        self.row.unwrap_or(ROW_CLASS)
    }

    fn selected_class(&self) -> &'a str {
        self.selected.unwrap_or(SELECTED_CLASS)
    }

    fn headline_class(&self) -> &'a str {
        self.headline.unwrap_or(HEADLINE_CLASS)
    }

    fn founding_class(&self) -> &'a str {
        self.founding.unwrap_or(FOUNDING_CLASS)
    }

    fn live_class(&self) -> &'a str {
        self.live.unwrap_or(LIVE_CLASS)
    }
}

/// One entry in the journal.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JournalRow {
    /// What it is, in the product's own words.
    pub headline: String,
    /// The mark beside the headline: an identity, a source, a sigil. Two
    /// spaces separate it from the headline, which is what both sources wrote.
    pub mark: Option<String>,
    /// What it meant when it was founded. This never moves.
    pub founding: String,
    /// What it means now, when that is not the same, or what stops it. `None`
    /// omits the line; `Some("")` still emits it, which is what the session
    /// does so its rows cannot change height as a revision arrives.
    pub live: Option<String>,
    /// Whether the cursor is on this row. Only the board has a cursor.
    pub selected: bool,
}

impl JournalRow {
    /// A plain, unmarked, unselected row.
    pub fn new(headline: impl Into<String>, founding: impl Into<String>) -> Self {
        Self {
            headline: headline.into(),
            mark: None,
            founding: founding.into(),
            live: None,
            selected: false,
        }
    }

    /// The same row with a mark beside its headline.
    #[must_use]
    pub fn marked(mut self, mark: impl Into<String>) -> Self {
        self.mark = Some(mark.into());
        self
    }

    /// The headline line's text: the headline, then the mark two spaces after
    /// it when there is one.
    pub fn headline_text(&self) -> String {
        match &self.mark {
            Some(mark) => format!("{}  {mark}", self.headline),
            None => self.headline.clone(),
        }
    }

    /// Every line the row emits, as class and text, in order.
    ///
    /// [`journal_row`] emits exactly this, so a test reads the row the emitter
    /// builds rather than a restatement of it — the same reason
    /// [`field_cells`](crate::field_cells) exists.
    pub fn lines<'a>(&self, classes: &JournalClasses<'a>) -> Vec<(&'a str, String)> {
        let mut lines = vec![
            (classes.headline_class(), self.headline_text()),
            (classes.founding_class(), self.founding.clone()),
        ];
        if let Some(live) = &self.live {
            lines.push((classes.live_class(), live.clone()));
        }
        lines
    }

    /// The row container's class.
    pub fn class(&self, classes: &JournalClasses<'_>) -> String {
        if self.selected {
            format!("{} {}", classes.row_class(), classes.selected_class())
        } else {
            classes.row_class().to_owned()
        }
    }
}

/// What a product has to say about its journal.
///
/// `rows` is the panel; the rest are the places the sources differ, in the
/// spirit of [`ExaminerModel`](crate::ExaminerModel): `None` is what everyone
/// else does.
#[derive(Clone, Debug, Default)]
pub struct JournalModel<'a> {
    /// The panel's heading. `None` for a product that arranges its own.
    pub title: Option<&'a str>,
    /// The summary over the rows, as the id it is read by and its words. It
    /// leads because in a window this size the rows run past the fold.
    pub summary: Option<(&'a str, String)>,
    /// The entries, in the order the product reads them.
    pub rows: Vec<JournalRow>,
    /// A paragraph in place of the rows when there are none.
    pub note: Option<&'a str>,
    /// The panel's `class`. `None` is no class at all.
    pub class: Option<&'a str>,
    /// The six names above. [`JournalClasses::SHARED`] is the ordinary case.
    pub classes: JournalClasses<'a>,
}

impl<'a> JournalModel<'a> {
    /// A journal titled `title` with nothing in it yet.
    pub fn new(title: &'a str) -> Self {
        Self {
            title: Some(title),
            ..Self::default()
        }
    }
}

/// One row: the headline, the founding line, and the live line when there is
/// one.
pub fn journal_row<State, Action>(
    row: &JournalRow,
    classes: &JournalClasses<'_>,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let lines: Vec<Child<State, Action>> = row
        .lines(classes)
        .into_iter()
        .map(|(class, words)| {
            Box::new(el("div", text(words)).attr("class", class)) as Child<State, Action>
        })
        .collect();
    Box::new(el("div", lines).attr("class", row.class(classes)))
}

/// The rows alone: the container over one row each, or the note when empty.
pub fn journal_rows<State, Action>(
    rows: &[JournalRow],
    note: Option<&str>,
    classes: &JournalClasses<'_>,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let children: Vec<Child<State, Action>> = if rows.is_empty() {
        note.into_iter()
            .map(|note| Box::new(el("p", text(note.to_owned()))) as Child<State, Action>)
            .collect()
    } else {
        rows.iter().map(|row| journal_row(row, classes)).collect()
    };
    Box::new(el("div", children).attr("class", classes.journal_class()))
}

/// The whole panel: an `aside` carrying the heading, the summary line and the
/// rows.
pub fn journal<State, Action>(model: &JournalModel<'_>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut children: Vec<Child<State, Action>> = Vec::with_capacity(3);
    if let Some(title) = model.title {
        children.push(Box::new(el("h2", text(title.to_owned()))));
    }
    if let Some((id, words)) = &model.summary {
        children.push(crate::status::status_line(id, words.clone()));
    }
    children.push(journal_rows(&model.rows, model.note, &model.classes));
    let panel = el("aside", children);
    Box::new(match model.class {
        Some(class) => panel.attr("class", class),
        None => panel,
    })
}
