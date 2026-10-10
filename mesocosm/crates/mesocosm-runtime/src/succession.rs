// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The questions the run holds at, read from the interim loop's
//! happenings (ruling 762): a birth to the played critter (183), its death
//! with the cohort as further lives (61), and the boundary's review (57).
//! The hold is the driver's; the world never learns anybody stopped.

use isocosm::directing::interim::{Happening, Interim, Turn};
use isocosm::schema::{Id, Method};
use isocosm_overlay::Intent;
use isocosm_overlay::mesocosm::{CheckpointAnswerKind, MesocosmIntent};

use crate::runtime::Envelope;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Birth {
    pub parent: Id,
    pub child: Id,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loss {
    pub critter: Id,
    /// The life the loop took up already, if any.
    pub next: Option<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub tick: u64,
    pub lineage: String,
    /// The unplayed lines' turns, in initiative order.
    pub turns: Vec<Turn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Occasion {
    Birth(Birth),
    Loss(Loss),
    Epoch(Boundary),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    pub tick: u64,
    pub occasion: Occasion,
    /// The lives a player may take up here.
    pub heirs: Vec<Id>,
}

impl Checkpoint {
    pub fn heir(&self) -> Option<Id> {
        self.heirs.first().copied()
    }

    /// Whether `envelope` answers this question.
    pub fn answers(&self, envelope: &Envelope) -> bool {
        let Intent::Game(MesocosmIntent::Checkpoint(answer)) = &envelope.intent else {
            return false;
        };
        matches!(
            (answer.kind, &self.occasion),
            (CheckpointAnswerKind::Birth(_), Occasion::Birth(_))
                | (CheckpointAnswerKind::Death(_), Occasion::Loss(_))
                | (CheckpointAnswerKind::EpochReview(_), Occasion::Epoch(_))
        )
    }
}

/// The question a round opened, if any: a loss first, then a birth, then
/// the boundary.
pub(crate) fn opened(interim: &Interim, happenings: &[Happening]) -> Option<Checkpoint> {
    let tick = interim.session.sim.state().tick;
    let lost = happenings.iter().find_map(|h| match h {
        Happening::Died { critter, next } => Some(Loss {
            critter: *critter,
            next: *next,
        }),
        _ => None,
    });
    if let Some(loss) = lost {
        let heirs = heirs(interim, loss.critter, loss.next);
        return Some(Checkpoint {
            tick,
            occasion: Occasion::Loss(loss),
            heirs,
        });
    }
    let born = happenings.iter().find_map(|h| match h {
        Happening::Born { parent, child } => Some(Birth {
            parent: *parent,
            child: *child,
        }),
        _ => None,
    });
    if let Some(birth) = born {
        return Some(Checkpoint {
            tick,
            heirs: vec![birth.child],
            occasion: Occasion::Birth(birth),
        });
    }
    happenings.iter().find_map(|h| match h {
        Happening::Boundary { tick, turns } => Some(Checkpoint {
            tick: *tick,
            occasion: Occasion::Epoch(Boundary {
                tick: *tick,
                lineage: played_lineage(interim).unwrap_or_default(),
                turns: turns.clone(),
            }),
            heirs: vec![],
        }),
        _ => None,
    })
}

fn played_lineage(interim: &Interim) -> Option<String> {
    let critter = interim.critter()?;
    let pop = &interim.session.sim.state().population;
    pop.get(critter).map(|e| e.lineage.clone())
}

/// The cohort as further lives: the life already taken up first, then the
/// lost critter's living deliberative kin by identity.
fn heirs(interim: &Interim, lost: Id, next: Option<Id>) -> Vec<Id> {
    let pop = &interim.session.sim.state().population;
    let lineage = pop.get(lost).map(|e| e.lineage.clone());
    let mut heirs: Vec<Id> = next.into_iter().collect();
    for (first, group) in &pop.groups {
        let e = &group.entity;
        let kin = Some(&e.lineage) == lineage.as_ref();
        if kin && e.alive && e.method == Method::Deliberative && Some(*first) != next {
            heirs.push(*first);
        }
    }
    heirs
}
