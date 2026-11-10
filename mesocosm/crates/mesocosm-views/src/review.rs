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
use cosmograph::lineage::{Offer, Reading, sheet::Sheet};
use std::fmt::Debug;
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

/// What a candidate's grown body would change against the line's own, as
/// signed changes in a fixed order: mass, reach, speed, clutch, upkeep,
/// comfort, water, light, cells by function, systems gained and lost, feeding, then how it lives
/// (case 1215). Empty where nothing differs.
pub fn sheet_words(had: &Sheet, offered: &Sheet) -> String {
    fn signed(from: i128, to: i128) -> Option<String> {
        (from != to).then(|| format!("{:+}", to - from))
    }
    fn word(of: impl Debug) -> String {
        format!("{of:?}").to_lowercase()
    }
    let n = |v: u128| i128::try_from(v).unwrap_or(i128::MAX);
    let mut out: Vec<String> = Vec::new();
    out.extend(signed(n(had.mass), n(offered.mass)).map(|d| format!("{d} mg")));
    out.extend(signed(had.reach.into(), offered.reach.into()).map(|d| format!("{d} reach")));
    if let (Some(a), Some(b)) = (had.speed, offered.speed) {
        out.extend(signed(a.into(), b.into()).map(|d| format!("{d} speed")));
    }
    out.extend(signed(had.clutch.into(), offered.clutch.into()).map(|d| format!("{d} clutch")));
    if let (Some(a), Some(b)) = (had.upkeep, offered.upkeep) {
        out.extend(signed(a.into(), b.into()).map(|d| format!("{d} upkeep")));
    }
    if let (Some(a), Some(b)) = (had.comfort, offered.comfort)
        && a != b
    {
        out.push(format!("comfort {}..{} -> {}..{}", a[0], a[1], b[0], b[1]));
    }
    if let (Some(a), Some(b)) = (had.water, offered.water)
        && a != b
    {
        out.push(format!("water {a} -> {b}"));
    }
    if let (Some(a), Some(b)) = (had.light, offered.light)
        && a != b
    {
        out.push(format!("light {a} -> {b}"));
    }
    let keys: std::collections::BTreeSet<&String> =
        had.functions.keys().chain(offered.functions.keys()).collect();
    for key in keys {
        let (a, b) = (had.functions.get(key).copied(), offered.functions.get(key).copied());
        let (a, b) = (a.unwrap_or(0), b.unwrap_or(0));
        let name = key.trim_start_matches("function:");
        if a != b {
            let sign = if b > a { '+' } else { '-' };
            out.push(format!("{sign}{name} {}", a.abs_diff(b)));
        }
    }
    let bare = |k: &String| k.trim_start_matches("system:").to_string();
    out.extend(offered.systems.difference(&had.systems).map(|k| format!("gains system {}", bare(k))));
    out.extend(had.systems.difference(&offered.systems).map(|k| format!("loses system {}", bare(k))));
    if had.feeding != offered.feeding {
        out.push(format!("feeds {} -> {}", word(had.feeding), word(offered.feeding)));
    }
    if had.trophic != offered.trophic {
        let of = |t: Option<_>| t.map_or("none".into(), word);
        out.push(format!("lives {} -> {}", of(had.trophic), of(offered.trophic)));
    }
    out.join(", ")
}

/// A row in words; `had` is the review's own sheet, which an offer's sheet
/// is set beside for its preview (case 1215).
pub fn row_words(offer: &Offer, selected: bool, had: Option<&Sheet>) -> BoardRow {
    row_words_of(offer, selected, had, None)
}

/// A row in words, with a draft offer's cost set against the review's
/// `budget` in its price (cases 1203; lineage-rounds D11).
pub fn row_words_of(offer: &Offer, selected: bool, had: Option<&Sheet>, budget: Option<u128>) -> BoardRow {
    let changes = match offer.commands.len() {
        0 => "nothing to change".to_string(),
        n => format!("{n} change{}", if n == 1 { "" } else { "s" }),
    };
    BoardRow {
        name: offer_name(offer),
        source: "the boundary".into(),
        net: format!(
            "{} mg held by {} over {} ticks",
            offer.score.held, offer.score.members, offer.score.ticks
        ),
        price: match (offer.cost, budget) {
            (Some(cost), Some(budget)) => format!("{changes}, costs {cost} of {budget} mg"),
            (Some(cost), None) => format!("{changes}, costs {cost} mg"),
            _ => changes,
        },
        preview: had.zip(offer.sheet.as_ref()).map_or_else(String::new, |(a, b)| sheet_words(a, b)),
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


#[cfg(test)]
mod tests {
    use super::*;
    use cosmograph::{lineage::sheet::Sheet, process::FeedingMode, trophic::Trophic};

    fn sheet() -> Sheet {
        Sheet {
            mass: 1000,
            speed: Some(500),
            reach: 3,
            clutch: 2,
            trophic: Some(Trophic::Consumer),
            feeding: FeedingMode::Grazer,
            functions: [("function:contract".to_string(), 8), ("function:store".to_string(), 6)].into(),
            systems: ["system:respiration".to_string()].into(),
            upkeep: Some(4),
            comfort: None,
            water: None,
            light: None,
        }
    }

    #[test]
    fn no_difference_says_nothing() {
        assert_eq!(sheet_words(&sheet(), &sheet()), "");
        // A figure only one side has is no change.
        let mut b = sheet();
        b.speed = None;
        b.upkeep = None;
        assert_eq!(sheet_words(&sheet(), &b), "");
    }

    #[test]
    fn each_change_appears_once_in_a_fixed_order() {
        let had = sheet();
        let mut b = sheet();
        b.mass += 120;
        b.reach += 1;
        b.speed = Some(460);
        b.clutch += 1;
        b.upkeep = Some(6);
        b.functions.insert("function:contract".into(), 20);
        b.functions.insert("function:store".into(), 2);
        b.systems = ["system:circulation".to_string()].into();
        b.feeding = FeedingMode::Omnivore;
        b.trophic = Some(Trophic::Producer);
        assert_eq!(
            sheet_words(&had, &b),
            "+120 mg, +1 reach, -40 speed, +1 clutch, +2 upkeep, +contract 12, -store 4, \
             gains system circulation, loses system respiration, feeds grazer -> omnivore, \
             lives consumer -> producer"
        );
        // A function gained or lost whole is a change of its cells.
        let mut c = sheet();
        c.functions.remove("function:store");
        c.functions.insert("function:fix".into(), 3);
        assert_eq!(sheet_words(&had, &c), "+fix 3, -store 6");
    }

    #[test]
    fn a_moved_comfort_band_reads_after_upkeep() {
        let mut had = sheet();
        had.comfort = Some([10, 30]);
        let mut b = had.clone();
        b.upkeep = Some(6);
        b.comfort = Some([8, 28]);
        b.functions.insert("function:contract".into(), 20);
        assert_eq!(sheet_words(&had, &b), "+2 upkeep, comfort 10..30 -> 8..28, +contract 12");
        // A band only one side has, or an unmoved one, is no change.
        b.comfort = had.comfort;
        b.upkeep = had.upkeep;
        b.functions = had.functions.clone();
        assert_eq!(sheet_words(&had, &b), "");
        b.comfort = None;
        assert_eq!(sheet_words(&had, &b), "");
    }

    #[test]
    fn moved_water_and_light_read_after_the_comfort_band() {
        let mut had = sheet();
        had.comfort = Some([10, 30]);
        had.water = Some(40);
        had.light = Some(0);
        let mut b = had.clone();
        b.comfort = Some([8, 28]);
        b.water = Some(31);
        b.light = Some(12);
        b.functions.insert("function:contract".into(), 20);
        assert_eq!(
            sheet_words(&had, &b),
            "comfort 10..30 -> 8..28, water 40 -> 31, light 0 -> 12, +contract 12"
        );
        // A figure only one side has, or an unmoved one, is no change.
        b.comfort = had.comfort;
        b.functions = had.functions.clone();
        b.light = None;
        assert_eq!(sheet_words(&had, &b), "water 40 -> 31");
        b.water = had.water;
        assert_eq!(sheet_words(&had, &b), "");
    }

    fn offer(cost: Option<u128>, with: Option<Sheet>) -> Offer {
        Offer {
            name: "candidate:wider-gut".into(),
            score: cosmograph::lineage::boundary::Score { held: 900, members: 4, ticks: 60 },
            commands: vec![cosmograph::history::Command::Leave { participant: 0, critter: 0 }],
            why_not: None,
            cost,
            sheet: with,
        }
    }

    #[test]
    fn a_draft_row_shows_its_cost_against_the_budget_and_the_sheet_diff() {
        let mut more = sheet();
        more.mass += 120;
        let row = row_words_of(&offer(Some(120), Some(more.clone())), true, Some(&sheet()), Some(500));
        assert_eq!(row.price, "1 change, costs 120 of 500 mg");
        assert_eq!(row.preview, "+120 mg");
        assert!(journal_row(&row).founding.contains("1 change, costs 120 of 500 mg / +120 mg"), "{:?}", journal_row(&row).founding);
        // No budget, no cost, or no sheet: the row reads as it did.
        assert_eq!(row_words_of(&offer(Some(120), None), false, None, None).price, "1 change, costs 120 mg");
        let plain = row_words(&offer(None, None), false, None);
        assert_eq!((plain.price.as_str(), plain.preview.as_str()), ("1 change", ""));
        assert_eq!(plain, row_words_of(&offer(None, None), false, None, Some(500)));
    }
}
