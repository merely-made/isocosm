// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Dev truth about one part of a native body: its shape, the functions its
//! cells hold, its intake port and the body's feeding mode (766), whether it
//! lives, and where it came from.

use isocosm::mosaic::ports::{active, feeding};
use isocosm::process::{FeedingMode, IntakePort};
use isocosm::schema::{Id, PartId};
use isocosm::simulation::Simulation;
use isometer_core::classify;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartReading {
    pub organism: String,
    pub id: String,
    pub role: String,
    pub process: String,
    pub intake: String,
    pub feeding: String,
    pub condition: String,
    pub discovery_condition: String,
    pub history_event: String,
    pub lineage: String,
    pub donor: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartInspection {
    pub reading: Option<PartReading>,
    pub notice: Option<String>,
}

fn unavailable(why: &str) -> PartInspection {
    PartInspection {
        reading: None,
        notice: Some(why.into()),
    }
}

pub fn part_of(sim: &Simulation, critter: Id, part: PartId) -> PartInspection {
    let Some(e) = sim.state().population.get(critter) else {
        return unavailable("the selected critter is unavailable");
    };
    let (Some(p), Some(geometry)) = (e.parts.get(&part), e.body.as_ref().and_then(|b| b.part(part)))
    else {
        return unavailable("the selected part is unavailable");
    };
    let cells: Vec<String> = p
        .cells
        .iter()
        .map(|(f, n)| format!("{} ({n})", f.trim_start_matches("function:")))
        .collect();
    let origin = geometry.origin.map_or("founding".to_string(), |tag| {
        format!("line {}", tag.saturating_sub(1))
    });
    PartInspection {
        reading: Some(PartReading {
            organism: critter.to_string(),
            id: part.0.to_string(),
            role: super::follow::role_word(classify(geometry.half_extent)).into(),
            process: if cells.is_empty() { "none".into() } else { cells.join(", ") },
            intake: active(p).map_or("not an intake".into(), port_words),
            feeding: feeding_word(feeding(e)).into(),
            condition: if e.lives(part) { "living" } else { "gone" }.into(),
            discovery_condition: String::new(),
            history_event: String::new(),
            lineage: e.lineage.clone(),
            donor: origin,
        }),
        notice: None,
    }
}

fn port_words(port: IntakePort) -> String {
    format!("{port:?}").to_lowercase()
}

pub fn feeding_word(mode: FeedingMode) -> &'static str {
    match mode {
        FeedingMode::Producer => "fixes its own",
        FeedingMode::Grazer => "grazes",
        FeedingMode::Predator => "hunts",
        FeedingMode::Omnivore => "eats what it finds",
        FeedingMode::Scavenger => "scavenges",
    }
}
