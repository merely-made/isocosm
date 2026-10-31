// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vitals panel over the native world: what the played critter holds,
//! the kingdom its body reads as (`kingdom::of`), how it feeds, what its
//! graft allowance leaves (`rules::Compatibility`), and the enclosure's
//! windows. The core answers what is; this crate says it.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};
use cosmograph::kingdom::{self, Kingdom};
use cosmograph::mosaic::ports::feeding;
use cosmograph::schema::{Entity, Id};
use cosmograph::simulation::Simulation;
use mesocosm_runtime::{Refusal, TermTable, Trend};

mod graft;
#[cfg(test)]
mod tests;
pub use graft::compatibility_words;

pub type VitalsChild = Box<dyn AnyView<Vitals, (), GenetCtx, GenetElement>>;

const BAR_WIDTH: f32 = 168.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vitals {
    pub energy_mg: Option<u64>,
    pub fullness: f32,
    /// The share of its cells the critter has lost, out of a hundred, and
    /// its mood: what the world's needs make of it (the 2026-10-18 note to
    /// ruling 227). Mood is absent in a world with no mind.
    pub wound: Option<u64>,
    pub mood: Option<i64>,
    pub notice: Option<String>,
    pub replacement: Option<String>,
    pub warning: Option<String>,
    pub kingdom: Option<String>,
    pub feeding: Option<String>,
    pub graft: Option<String>,
}

impl Vitals {
    pub fn is_dead(&self) -> bool {
        self.energy_mg.is_none()
    }
}

/// What a body holds in all: its ledger and its parts' matter.
pub fn held_mg(e: &Entity) -> u64 {
    let parts = e.parts.values().flat_map(|p| p.matter.values());
    e.accounts.values().chain(parts).fold(0, |a, b| a.saturating_add(*b))
}

pub fn kingdom_word(kingdom: Kingdom) -> &'static str {
    match kingdom {
        Kingdom::Producer => "kingdom:flora",
        Kingdom::Consumer => "kingdom:fauna",
        Kingdom::Decomposer => "kingdom:myco",
    }
}

/// The played critter's vitals; dead or absent reads as dead.
pub fn vitals_of(
    sim: &Simulation,
    critter: Option<Id>,
    high_water: u64,
    notice: Option<String>,
    trend: Option<&Trend>,
) -> Vitals {
    vitals_of_with_terms(sim, critter, high_water, notice, trend, &TermTable::default())
}

pub fn vitals_of_with_terms(
    sim: &Simulation, critter: Option<Id>, high_water: u64,
    notice: Option<String>, trend: Option<&Trend>, terms: &TermTable,
) -> Vitals {
    let pop = &sim.state().population;
    let e = critter.and_then(|c| pop.get(c)).filter(|e| e.alive);
    let energy_mg = e.map(held_mg);
    let fullness = match (energy_mg, high_water) {
        (Some(energy), top) if top > 0 => (energy as f32 / top as f32).clamp(0.0, 1.0),
        _ => 0.0,
    };
    Vitals {
        energy_mg,
        fullness,
        wound: e.map(cosmograph::harm::wound_share),
        mood: critter.filter(|_| e.is_some()).and_then(|id| sim.mood(id)),
        notice: notice.filter(|_| energy_mg.is_some()),
        replacement: trend.map(replacement_words),
        warning: trend.and_then(warning_words),
        kingdom: e.and_then(kingdom::of)
            .map(|k| terms.form(kingdom_word(k), "indefinite").to_owned()),
        feeding: e.map(|e| terms.form(crate::dev::part::feeding_word(feeding(e)), "verb").to_owned()),
        graft: critter.filter(|_| e.is_some()).and_then(|id| allowance_words(sim, id)),
    }
}

/// What the world's graft rule allows this body beyond what it keeps.
fn allowance_words(sim: &Simulation, id: Id) -> Option<String> {
    match sim.graft_allowance(id) {
        Ok(None) => None,
        Ok(Some(receipt)) => Some(compatibility_words(&receipt)),
        Err(why) => Some(format!("graft allowance unavailable: {why}")),
    }
}

pub fn replacement_words(trend: &Trend) -> String {
    format!(
        "{} born, {} died in {} ticks",
        trend.born, trend.died, trend.replacement_ticks
    )
}

pub fn warning_words(trend: &Trend) -> Option<String> {
    trend.warns().then(|| {
        format!(
            "the stand has been shrinking for {} ticks: {} mg lost over the last {}; mouths took {} mg in the same window",
            trend.shortfall_ticks,
            trend.stand_change.unsigned_abs(),
            trend.stand_ticks,
            trend.grazed
        )
    })
}

/// A refused envelope, in the panel's words.
pub fn refusal_words(refusal: &Refusal) -> String {
    match refusal {
        Refusal::NotPlayed => "that is not who you play".into(),
        Refusal::Unasked => "nothing is asking that".into(),
        Refusal::Unbuilt(what) => format!("{what} has no native command yet"),
        Refusal::Sim(why) => why.clone(),
    }
}

/// The newest refusal among a run's applied envelopes, if any.
pub fn notice_in<E>(applied: &[(E, Result<String, Refusal>)]) -> Option<String> {
    let mut newest = applied.iter().rev();
    newest.find_map(|(_, outcome)| outcome.as_ref().err().map(refusal_words))
}

/// The labelled facts the panel states, in order. A dead critter states its
/// state alone; a living one its holdings, wounds and mood, and what it is.
pub fn vital_rows(vitals: &Vitals) -> Vec<(&'static str, String)> {
    let mut rows = match vitals.energy_mg {
        Some(energy) => vec![("holds", format!("{energy} mg"))],
        None => vec![("state", "dead".to_owned())],
    };
    let wounded = vitals.wound.map(|share| format!("{share} of 100"));
    let mood = vitals.mood.map(|mood| mood.to_string());
    for (key, value) in [
        ("wounded", &wounded),
        ("mood", &mood),
        ("body", &vitals.kingdom),
        ("feeds", &vitals.feeding),
        ("graft", &vitals.graft),
        ("replacement", &vitals.replacement),
    ] {
        if let Some(value) = value {
            rows.push((key, value.clone()));
        }
    }
    rows
}

pub fn vitals_root(vitals: &Vitals) -> VitalsChild {
    let mut children: Vec<VitalsChild> = Vec::new();

    // The catalog's labelled-facts component, which is exactly what a vital
    // sign is: an inert key and its value. Not a hand-rolled row.
    let rows = vital_rows(vitals)
        .into_iter()
        .map(|(key, value)| DetailRow::new(key, value))
        .collect::<Vec<_>>();
    children.push(Box::new(detail_panel::<Vitals, ()>(&[DetailSection::new(
        "vitals", rows,
    )])));

    // The bar is a box, not a drawing: its fill is a width, and the engine
    // paints it. A dead critter has no bar at all.
    if !vitals.is_dead() {
        let fill = el::<_, Vitals, ()>("div", ())
            .attr("class", "vital-bar-fill")
            .attr(
                "style",
                format!("width: {:.0}px", vitals.fullness * BAR_WIDTH),
            );
        children.push(Box::new(
            el::<_, Vitals, ()>("div", Box::new(fill) as VitalsChild).attr("class", "vital-bar"),
        ));
    }

    if let Some(words) = &vitals.notice {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words.clone())).attr("class", "vital-notice"),
        ));
    }

    // Last, and only when it is true. A warning that is always on screen is a
    // decoration.
    if let Some(words) = &vitals.warning {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words.clone())).attr("class", "vital-warning"),
        ));
    }

    Box::new(el::<_, Vitals, ()>("div", children).attr("class", "vitals"))
}

/// The sheet the panel is styled by. Dark and translucent so the section
/// reads through it, and plain: this is a status panel, not a flourish.
pub fn vitals_css() -> &'static str {
    r#"
.vitals {
    /* The host's raster is 300 wide and this is a content box, so the twelve
       pixels of padding on each side come out of it. Setting the two equal
       clipped the last character off every line that reached the edge. */
    width: 276px;
    padding: 10px 12px;
    background-color: #10141aee;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 14px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-bottom: 4px;
}
.detail-row { margin-bottom: 2px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; font-weight: bold; margin-left: 8px; }
.vital-bar {
    width: 168px;
    height: 8px;
    margin-top: 6px;
    background-color: #263026;
}
.vital-bar-fill {
    height: 8px;
    background-color: #7fc46a;
}
.vital-notice {
    margin-top: 8px;
    color: #e2a06a;
    font-size: 13px;
}
.vital-warning {
    margin-top: 8px;
    color: #d8776a;
    font-size: 12px;
}
"#
}

