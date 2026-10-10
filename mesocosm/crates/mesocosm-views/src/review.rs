// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The trait board: the played line's own turn, on screen. (PE3b)
//!
//! The fourth cambium surface, and the second that appears because the world is
//! **waiting**. Where `succession` asks one question about one critter, this
//! asks the lineage's: the epoch is reckoned, every other line has already
//! weighed what it could, and yours has not.
//!
//! **It is a table, not an editor.** Nothing here arranges tissue, names a
//! cell, or prices anything: every number on it was read off the world by
//! `World::offers` and the driver's review, and the only things a player can do
//! are move the selection, commit the selected candidate, and leave. The
//! candidate's own proposals are *shown* — the game's, and a pack's where one
//! applies — because two proposal sources over one validator is a fact worth
//! being able to see, not because a player picks between them.
//!
//! Host-agnostic like the panels beside it: this crate says what the surface is
//! and what its words are, the host says where the raster lands, and the driver
//! decides when there is anything to show.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};
use isocosm::lineage::{Offer, Reading};
use isomere::{JournalClasses, JournalRow};
use mesocosm_runtime::Trend;

pub type BoardChild = Box<dyn AnyView<Board, (), GenetCtx, GenetElement>>;

/// One candidate's row, already in words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardRow {
    pub name: String,
    pub source: String,
    pub net: String,
    pub price: String,
    pub preview: String,
    pub reason: Option<String>,
    pub selected: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Board {
    pub headline: String,
    pub facts: Vec<(String, String)>,
    pub readings: Vec<String>,
    pub rows: Vec<BoardRow>,
    pub commit: Option<String>,
    pub next: String,
    pub stay: String,
}

impl Board {
    /// The boundary's facts for the played line (lineage::Review, 762).
    pub fn of(tick: u64, lineage: &str, program: Option<&str>, trend: &Trend) -> Self {
        Self {
            headline: "the epoch is over".into(),
            facts: vec![
                ("tick".into(), tick.to_string()),
                (
                    "your line".into(),
                    match program {
                        Some(digest) => format!("{lineage}, born under {digest}"),
                        None => format!("{lineage}, born as it always was"),
                    },
                ),
                ("the enclosure".into(), crate::vitals::replacement_words(trend)),
            ],
            readings: Vec::new(),
            rows: Vec::new(),
            commit: None,
            next: "another candidate".into(),
            stay: "keep the line as it is".into(),
        }
    }

    pub fn has_a_choice(&self) -> bool {
        self.rows.iter().any(|row| row.reason.is_none() && !row.name.starts_with("the status"))
    }
}

pub fn reading_words(reading: &Reading) -> String {
    format!(
        "{} of {} — {}",
        reading.feat.trim_start_matches("feat:"),
        reading.lineage,
        reading.value
    )
}

/// The played line's readings, then how many other lines were read.
pub fn evidence_words(readings: &[Reading], lineage: &str) -> Vec<String> {
    let mut words: Vec<String> = readings
        .iter()
        .filter(|r| r.lineage == lineage)
        .map(reading_words)
        .collect();
    let mut others: Vec<&str> = readings
        .iter()
        .filter(|r| r.lineage != lineage)
        .map(|r| r.lineage.as_str())
        .collect();
    others.sort_unstable();
    others.dedup();
    if !others.is_empty() {
        words.push(format!("{} other line{} read", others.len(), if others.len() == 1 { "" } else { "s" }));
    }
    words
}

fn offer_name(offer: &Offer) -> String {
    match offer.name.as_str() {
        "candidate:stay" => "the status quo".into(),
        name => name.trim_start_matches("candidate:").replace('-', " "),
    }
}

pub fn row_words(offer: &Offer, selected: bool) -> BoardRow {
    BoardRow {
        name: offer_name(offer),
        source: "the boundary".into(),
        net: format!(
            "{} mg held by {} over {} ticks",
            offer.score.held, offer.score.members, offer.score.ticks
        ),
        price: match offer.commands.len() {
            0 => "nothing to change".into(),
            n => format!("{n} change{}", if n == 1 { "" } else { "s" }),
        },
        preview: String::new(),
        reason: offer.why_not.clone(),
        selected,
    }
}

/// What a pack's script made of an offer (781), in the row's words.
pub fn authored_words(proposed: &mesocosm_runtime::Proposed) -> String {
    match &proposed.cells {
        Ok(cells) => format!("{} would change {cells} cells", proposed.script),
        Err(why) => format!("{} declined: {why}", proposed.script),
    }
}

pub fn commit_words(offer: &Offer) -> Option<String> {
    offer
        .takeable()
        .then(|| format!("take {} into the line", offer_name(offer)))
}

pub fn board_root(state: &Board) -> BoardChild {
    let mut children: Vec<BoardChild> = vec![Box::new(
        el::<_, Board, ()>("div", text(state.headline.clone())).attr("class", "board-headline"),
    )];

    let facts = state
        .facts
        .iter()
        .map(|(key, value)| DetailRow::new(key.clone(), value.clone()))
        .collect();
    children.push(Box::new(detail_panel::<Board, ()>(&[DetailSection::new(
        "the boundary",
        facts,
    )])));

    // The evidence, and only when there is some. An epoch in which nothing was
    // worth noting says nothing rather than printing a heading over an
    // absence.
    if !state.readings.is_empty() {
        let readings = state
            .readings
            .iter()
            .map(|words| DetailRow::new("noted", words.clone()))
            .collect();
        children.push(Box::new(detail_panel::<Board, ()>(&[DetailSection::new(
            "what the epoch came to",
            readings,
        )])));
    }

    for row in &state.rows {
        children.push(row_view(row));
    }

    // Three answers, one line each. Two of them are keys that send an intent;
    // the third only moves a cursor, which is why it is stated last.
    if let Some(commit) = &state.commit {
        children.push(Box::new(
            el::<_, Board, ()>("div", text(format!("[R]  {commit}"))).attr("class", "board-answer"),
        ));
    }
    children.push(Box::new(
        el::<_, Board, ()>("div", text(format!("[Tab]  {}", state.next)))
            .attr("class", "board-answer"),
    ));
    children.push(Box::new(
        el::<_, Board, ()>("div", text(format!("[Enter]  {}", state.stay)))
            .attr("class", "board-answer"),
    ));

    Box::new(el::<_, Board, ()>("div", children).attr("class", "board"))
}

/// The board's own class names for a journal row (M3 of the isomere plan).
///
/// The board is styled by [`board_css`] below and drawn into its own raster,
/// not by isomere's shared sheet, so it keeps every name it had: the rows
/// become the shared journal without one of its pixels moving. When the board
/// joins the shared sheet these go.
const BOARD_CLASSES: JournalClasses<'static> = JournalClasses {
    journal: None,
    row: Some("board-row"),
    selected: Some("board-row-on"),
    headline: Some("board-row-name"),
    founding: Some("board-row-figures"),
    live: Some("board-row-reason"),
};

/// One candidate as the shared journal reads it: the name with its sources as
/// the mark, the three figures as the founding line, and the reason as the
/// live line — which is why a row that can be taken says nothing there rather
/// than printing "nothing is wrong" where a player learns to stop reading.
pub fn journal_row(row: &BoardRow) -> JournalRow {
    JournalRow {
        live: row.reason.clone(),
        selected: row.selected,
        ..JournalRow::new(
            row.name.clone(),
            format!("{} / {} / {}", row.net, row.price, row.preview),
        )
        .marked(format!("({})", row.source))
    }
}

/// One row, as three lines: what it is, what it came to, and what stops it.
fn row_view(row: &BoardRow) -> BoardChild {
    isomere::journal_row(&journal_row(row), &BOARD_CLASSES)
}

/// The sheet the board is styled by. Heavier than the vitals panel and darker
/// than the checkpoint's, because this is the screen the world stopped for.
pub fn board_css() -> &'static str {
    r#"
.board {
    width: 588px;
    padding: 14px 16px;
    background-color: #0b0f14f7;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 13px;
}
.board-headline {
    color: #e6d9a8;
    font-size: 18px;
    font-weight: bold;
    margin-bottom: 8px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-top: 6px;
    margin-bottom: 3px;
}
.detail-row { margin-bottom: 2px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; margin-left: 8px; }
.board-row {
    margin-top: 6px;
    padding: 4px 6px;
    background-color: #141a20;
}
.board-row-on {
    background-color: #1d2a22;
}
.board-row-name { color: #eaf2e6; font-weight: bold; }
.board-row-figures { color: #a8b7a4; font-size: 12px; }
.board-row-reason { color: #d8a06a; font-size: 12px; }
.board-answer {
    margin-top: 6px;
    color: #9fd08a;
    font-size: 13px;
}
"#
}

