// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The played line's own turn (legacy `world::review`, PE3b): the boundary
//! skips it, and this is the reading its turn is made of. Every candidate
//! the line could weigh, scored by the boundary's own grow-a-copy (684),
//! the status quo first, and those it cannot take kept with the reason.
//! It decides nothing; a commit is the offer's commands, sent as any other.

use super::{program, reckon};
use crate::directing::interim::boundary::{self, Candidate, Score};
use crate::{Result, Session, history::Command, schema::*};
use serde::{Deserialize, Serialize};

/// One row: a candidate, what growing it came to, and why it cannot be
/// taken when it cannot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub name: Key,
    pub score: Score,
    /// What committing it sends; empty for the status quo.
    pub commands: Vec<Command>,
    pub why_not: Option<String>,
}

impl Offer {
    /// Whether committing it would change anything and be admitted.
    pub fn takeable(&self) -> bool {
        !self.commands.is_empty() && self.why_not.is_none()
    }
}

/// The boundary's reading for the played line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub tick: Tick,
    pub lineage: Key,
    /// The reckoning, every line's.
    pub readings: Vec<reckon::Reading>,
    /// The program the line is born under; `None` for its founding.
    pub program: Option<String>,
    pub offers: Vec<Offer>,
}

impl Review {
    /// Reads the review for `lineage`, each copy grown `scoring` ticks.
    pub fn of(session: &Session, lineage: &str, scoring: Tick) -> Result<Self> {
        Ok(Self {
            tick: session.sim.state().tick,
            lineage: lineage.into(),
            readings: reckon::readings(&session.sim),
            program: program::digest(&program::program(session, lineage)),
            offers: offers(session, lineage, scoring)?,
        })
    }

    /// The commands committing the row at `index`, when it can be taken.
    pub fn commit(&self, index: usize) -> Option<&[Command]> {
        let offer = self.offers.get(index)?;
        offer.takeable().then_some(offer.commands.as_slice())
    }
}

/// The status quo, then every variant of the line's recipe its lexicon
/// offers (752), each scored; one the world would refuse scores as the
/// world stands, beside its reason.
pub fn offers(session: &Session, lineage: &str, scoring: Tick) -> Result<Vec<Offer>> {
    let sim = &session.sim;
    let stay = Candidate {
        name: "candidate:stay".into(),
        commands: vec![],
    };
    let standing = boundary::score(session, lineage, &stay, scoring)?;
    let mut out = vec![Offer {
        name: stay.name,
        score: standing,
        commands: vec![],
        why_not: None,
    }];
    let living = boundary::standing(session, lineage).1 > 0;
    let most = sim.genesis().rules.directing().variants as usize;
    let line = sim.state().lineages.get(lineage);
    let Some(d) = line.and_then(|l| l.development.as_ref()) else {
        return Ok(out);
    };
    for (name, development) in super::revise::offered(session, lineage, d, most) {
        let candidate = Candidate {
            name,
            commands: vec![Command::Revise {
                lineage: lineage.into(),
                development: development.clone(),
            }],
        };
        let why_not = match sim.revisable(lineage, &development) {
            Err(why) => Some(why),
            Ok(_) if !living => Some("no body of the line is left".into()),
            Ok(_) => None,
        };
        let score = match why_not {
            None => boundary::score(session, lineage, &candidate, scoring)?,
            Some(_) => standing,
        };
        out.push(Offer {
            name: candidate.name,
            score,
            commands: candidate.commands,
            why_not,
        });
    }
    Ok(out)
}
