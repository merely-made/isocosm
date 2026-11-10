// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The played line's own turn (legacy `world::review`, PE3b): the boundary
//! skips it, and this is the reading its turn is made of. Every candidate
//! the line could weigh, scored by the boundary's own grow-a-copy (684),
//! the status quo first, and those it cannot take kept with the reason.
//! It decides nothing; a commit is the offer's commands, sent as any other.

use super::{draft, program, reckon};
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
    /// The adult mass a variant adds to a fully grown body, in a draft
    /// review (case 1203; lineage-rounds D11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<u128>,
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
    /// What a variant may cost, in a draft review (case 1203).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<u128>,
}

impl Review {
    /// Reads the review for `lineage`, each copy grown `scoring` ticks.
    pub fn of(session: &Session, lineage: &str, scoring: Tick) -> Result<Self> {
        Self::of_using(session, lineage, scoring, &mut boundary::score)
    }

    /// Reads the same offers through an explicitly supplied scoring clock.
    pub fn of_using(session: &Session, lineage: &str, scoring: Tick,
        score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>) -> Result<Self> {
        Self::of_draft_using(session, lineage, scoring, None, score)
    }

    /// The same, as the player's draft when given the tick the round began
    /// and the world's `draft` rule is set: variants only of kinds eaten
    /// whole since then, each with its cost against the line's budget
    /// (cases 1200, 1203; lineage-rounds D10 to D12).
    ///
    /// # Errors
    /// Fails if scoring an offer or growing a variant's body fails.
    pub fn of_draft_using(session: &Session, lineage: &str, scoring: Tick,
        draft: Option<Tick>,
        score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>) -> Result<Self> {
        let budget = draft.and_then(|_| draft::budget(session, lineage));
        Ok(Self {
            tick: session.sim.state().tick,
            lineage: lineage.into(),
            readings: reckon::readings(&session.sim),
            program: program::digest(&program::program(session, lineage)),
            offers: offers_draft_using(session, lineage, scoring, draft, score)?,
            budget,
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
    offers_using(session, lineage, scoring, &mut boundary::score)
}

/// The offer repertoire is unchanged; only the fork's clock is supplied.
pub fn offers_using(session: &Session, lineage: &str, scoring: Tick,
    score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>) -> Result<Vec<Offer>> {
    offers_draft_using(session, lineage, scoring, None, score)
}

/// The offers of a draft review when `draft` names the round's first tick
/// and the world's rule is set (case 1200, 1203), else as `offers_using`.
///
/// # Errors
/// Fails if scoring an offer or growing a variant's body fails.
pub fn offers_draft_using(session: &Session, lineage: &str, scoring: Tick,
    draft: Option<Tick>,
    score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>) -> Result<Vec<Offer>> {
    let sim = &session.sim;
    let stay = Candidate {
        name: "candidate:stay".into(),
        commands: vec![],
    };
    let standing = score(session, lineage, &stay, scoring)?;
    let mut out = vec![Offer {
        name: stay.name,
        score: standing,
        commands: vec![],
        why_not: None,
        cost: None,
    }];
    let living = boundary::standing(session, lineage).1 > 0;
    let most = sim.genesis().rules.directing().variants as usize;
    let line = sim.state().lineages.get(lineage);
    let Some(d) = line.and_then(|l| l.development.as_ref()) else {
        return Ok(out);
    };
    let budget = draft.and_then(|_| draft::budget(session, lineage));
    let eaten = budget.and(draft.zip(line)).map(|(since, l)| draft::eaten_since(l, since));
    let rules = &sim.genesis().rules;
    let offered = super::revise::offered_borne(session, lineage, d, most, eaten.as_ref());
    for (name, development, bears) in offered {
        let cost = match (budget, bears) {
            (Some(_), Some(_)) => Some(draft::cost(rules, d, &development)?),
            _ => None,
        };
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
            Ok(_) => match (cost, budget) {
                (Some(c), Some(b)) if c > b => {
                    Some(format!("costs {c} mg, {} mg over the budget of {b} mg", c - b))
                }
                _ => None,
            },
        };
        let score = match why_not {
            None => score(session, lineage, &candidate, scoring)?,
            Some(_) => standing,
        };
        out.push(Offer {
            name: candidate.name,
            score,
            commands: candidate.commands,
            why_not,
            cost,
        });
    }
    Ok(out)
}
