// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Dev truth about one part of a native body: its shape, the functions its
//! cells hold, its intake port and the body's feeding mode (766), whether it
//! lives, and where it came from.

use isocosm::mosaic::ports::{active, feeding};
use isocosm::process::{FeedingMode, IntakePort, NisKind};
use mesocosm_runtime::TermTable;
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
    pub scruple: String,
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
    part_of_with_terms(sim, critter, part, &TermTable::default())
}

pub fn part_of_with_terms(
    sim: &Simulation, critter: Id, part: PartId, terms: &TermTable,
) -> PartInspection {
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
            intake: active(p).map_or("not an intake".into(), |port| port_words(port, terms)),
            feeding: terms.form(feeding_word(feeding(e)), "verb").into(),
            condition: if e.lives(part) { "living" } else { "gone" }.into(),
            discovery_condition: String::new(),
            history_event: String::new(),
            lineage: e.lineage.clone(),
            donor: origin,
            scruple: match isocosm::matter::scruple::of(p, &sim.genesis().rules) {
                Ok(mix) if mix.is_empty() => "no tissue".into(),
                Ok(mix) => mix.iter().map(|(line, mg)| format!("{line}: {mg} mg"))
                    .collect::<Vec<_>>().join(", "),
                Err(why) => why,
            },
        }),
        notice: None,
    }
}

fn port_words(port: IntakePort, terms: &TermTable) -> String {
    let mut admitted = Vec::new();
    for (kind, id) in [(NisKind::Producer, "kingdom:flora"),
        (NisKind::Consumer, "kingdom:fauna"), (NisKind::Decomposer, "kingdom:myco")] {
        if port.admits_live(kind) { admitted.push(terms.form(id, "label")); }
    }
    if port.admits_deadstock() { admitted.push(terms.form("intake:deadstock", "label")); }
    admitted.join(", ")
}

/// The reading's identity, separate from a pack's display form.
pub fn feeding_word(mode: FeedingMode) -> &'static str {
    match mode {
        FeedingMode::Producer => "feeding:producer",
        FeedingMode::Grazer => "feeding:grazer",
        FeedingMode::Predator => "feeding:predator",
        FeedingMode::Omnivore => "feeding:omnivore",
        FeedingMode::Scavenger => "feeding:scavenger",
    }
}
