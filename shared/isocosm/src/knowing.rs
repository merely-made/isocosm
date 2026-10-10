// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a sophont knows of an event (wing rulings 84, 117 and 771; Eponym's
//! epistemic layer re-expressed): notes it holds, beside the reach field's
//! knowing. An observation needs the observer to have been a party or to
//! know the event by the field; a claim reads the event one way on the
//! claimant's own evidence; a report is a note the hearer holds of what it
//! was told; a correction is the claimant's own, and the claim's current
//! reading is its last. Any of it can be wrong (117): nothing checks a
//! reading against what happened. Each record's id is its note's cause.

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const OBSERVE: &str = "know:observe";
pub const CLAIM: &str = "know:claim";
pub const REPORT: &str = "know:report";
pub const CORRECT: &str = "know:correct";
/// The note kinds a world must declare to keep knowing.
pub const NOTE_KINDS: [&str; 4] = [OBSERVE, CLAIM, REPORT, CORRECT];

/// How a claim reads an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Reading {
    Helped,
    Betrayed,
}

impl Reading {
    fn key(self) -> &'static str {
        match self {
            Self::Helped => "helped",
            Self::Betrayed => "betrayed",
        }
    }
    fn of(key: &str) -> Option<Self> {
        match key {
            "helped" => Some(Self::Helped),
            "betrayed" => Some(Self::Betrayed),
            _ => None,
        }
    }
}

/// An event read one way.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Proposition {
    pub event: Key,
    pub reading: Reading,
}

/// A record of knowing, as the log carries it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Knowing {
    Observe { observer: Id, event: Key },
    Claim { claimant: Id, proposition: Proposition, supports: Vec<Key> },
    Report { reporter: Id, hearer: Id, claim: Key, proposition: Proposition },
    Correct { corrector: Id, claim: Key, replacement: Proposition, supports: Vec<Key> },
}

/// A claim as its holder holds it now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Belief {
    pub holder: Id,
    pub claim: Key,
    pub proposition: Proposition,
    pub supports: Vec<Key>,
}

/// A claim's reports and corrections, in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimHistory {
    pub claim: Key,
    pub reports: Vec<Key>,
    pub corrections: Vec<Key>,
}

fn holder(note: &Note) -> Option<Id> {
    note.core.subject.strip_prefix("entity:")?.parse().ok()
}

fn told(note: &Note) -> Option<Proposition> {
    let reading = Reading::of(note.extra.get("reading")?)?;
    Some(Proposition {
        event: note.core.object.clone(),
        reading,
    })
}

fn supports(note: &Note) -> Vec<Key> {
    let joined = note.extra.get("supports").map_or("", String::as_str);
    joined.split(',').filter(|s| !s.is_empty()).map(Into::into).collect()
}

impl Simulation {
    /// Writes one record of knowing; returns its id.
    pub(crate) fn know(&mut self, k: &Knowing) -> Result<Key> {
        match k {
            Knowing::Observe { observer, event } => self.observe(*observer, event),
            Knowing::Claim { claimant, proposition, supports } => {
                let supports = self.supported(*claimant, proposition, supports)?;
                self.hold_note(*claimant, CLAIM, proposition, &[("supports", supports.join(","))])
            },
            Knowing::Report { reporter, hearer, claim, proposition } => {
                let (claimant, claimed) = self.claimed(claim)?;
                let owned = claimant == *reporter && self.current(claim)?.0 == *proposition;
                let received = self.of_kind(REPORT).any(|n| {
                    holder(n) == Some(*reporter)
                        && n.extra.get("claim") == Some(claim)
                        && told(n).as_ref() == Some(proposition)
                });
                if proposition.event != claimed.event || !(owned || received) {
                    return Err("a report of what the reporter does not hold".into());
                }
                let extra = [("claim", claim.clone()), ("reporter", reporter.to_string())];
                self.hold_note(*hearer, REPORT, proposition, &extra)
            },
            Knowing::Correct { corrector, claim, replacement, supports } => {
                let (claimant, original) = self.claimed(claim)?;
                if claimant != *corrector {
                    return Err("only its claimant corrects a claim".into());
                }
                if replacement.event != original.event || self.current(claim)?.0 == *replacement {
                    return Err("a correction that changes the event or nothing".into());
                }
                let supports = self.supported(*corrector, replacement, supports)?;
                let extra = [("claim", claim.clone()), ("supports", supports.join(","))];
                self.hold_note(*corrector, CORRECT, replacement, &extra)
            },
        }
    }

    fn observe(&mut self, observer: Id, event: &str) -> Result<Key> {
        let e = self.state.events.get(event).ok_or("unknown event")?;
        let party = e.subject == observer || e.object == Some(observer);
        if !party && !self.knows(observer, event)? {
            return Err("an observation of what the observer does not know".into());
        }
        let id = format!("observation:{}", self.of_kind(OBSERVE).count());
        self.add_note(observer, event.into(), OBSERVE, String::new(), None, id.clone())?;
        Ok(id)
    }

    fn hold_note(&mut self, holder: Id, kind: &str, p: &Proposition, extra: &[(&str, String)]) -> Result<Key> {
        if !self.state.events.contains_key(&p.event) {
            return Err("unknown event".into());
        }
        let word = kind.trim_start_matches("know:");
        let id = format!("{word}:{}", self.of_kind(kind).count());
        self.room(self.state.notes.len())?;
        let mut note = self.note(holder, p.event.clone(), kind, String::new(), None, id.clone())?;
        note.extra.insert("reading".into(), p.reading.key().into());
        for (k, v) in extra {
            note.extra.insert((*k).into(), v.clone());
        }
        self.state.notes.push(note);
        Ok(id)
    }

    /// The evidence `actor` cites, deduplicated and its own: observations
    /// of the event, or reports it heard of this reading.
    fn supported(&self, actor: Id, p: &Proposition, cited: &[Key]) -> Result<Vec<Key>> {
        let cited: BTreeSet<&Key> = cited.iter().collect();
        if cited.is_empty() {
            return Err("a claim with no evidence".into());
        }
        for c in &cited {
            let n = self.record(c).ok_or_else(|| format!("unknown evidence {c}"))?;
            let fits = match n.core.kind.as_str() {
                OBSERVE => n.core.object == p.event,
                REPORT => told(n).as_ref() == Some(p),
                _ => false,
            };
            if holder(n) != Some(actor) || !fits {
                return Err(format!("{c} is not {actor}'s evidence for this"));
            }
        }
        Ok(cited.into_iter().cloned().collect())
    }

    fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Note> + 'a {
        self.state.notes.iter().filter(move |n| n.core.kind == kind)
    }

    /// The note recording `id`.
    fn record(&self, id: &str) -> Option<&Note> {
        let kinds = NOTE_KINDS;
        self.state.notes.iter().find(|n| n.core.cause == id && kinds.contains(&n.core.kind.as_str()))
    }

    fn claimed(&self, claim: &str) -> Result<(Id, Proposition)> {
        let n = self.record(claim).filter(|n| n.core.kind == CLAIM).ok_or("unknown claim")?;
        Ok((holder(n).ok_or("a claim held by no one")?, told(n).ok_or("a claim of no reading")?))
    }

    /// A claim's current reading and evidence: its last correction's, or
    /// its own.
    fn current(&self, claim: &str) -> Result<(Proposition, Vec<Key>)> {
        let n = self.record(claim).filter(|n| n.core.kind == CLAIM).ok_or("unknown claim")?;
        let mut now = (told(n).ok_or("a claim of no reading")?, supports(n));
        for c in self.of_kind(CORRECT).filter(|c| c.extra.get("claim").map(String::as_str) == Some(claim)) {
            if let Some(p) = told(c) {
                now = (p, supports(c));
            }
        }
        Ok(now)
    }

    /// What `holder` believes: each claim it made, as it reads now.
    pub fn beliefs_of(&self, holder_id: Id) -> Vec<Belief> {
        let claims = self.of_kind(CLAIM).filter(|n| holder(n) == Some(holder_id));
        claims
            .filter_map(|n| {
                let (proposition, supports) = self.current(&n.core.cause).ok()?;
                Some(Belief {
                    holder: holder_id,
                    claim: n.core.cause.clone(),
                    proposition,
                    supports,
                })
            })
            .collect()
    }

    /// The reports `hearer` holds, as (report, claim, proposition).
    pub fn received_by(&self, hearer: Id) -> Vec<(Key, Key, Proposition)> {
        let heard = self.of_kind(REPORT).filter(|n| holder(n) == Some(hearer));
        heard
            .filter_map(|n| Some((n.core.cause.clone(), n.extra.get("claim")?.clone(), told(n)?)))
            .collect()
    }

    pub fn history_of(&self, claim: &str) -> Result<ClaimHistory> {
        self.claimed(claim)?;
        let about = |n: &&Note| n.extra.get("claim").map(String::as_str) == Some(claim);
        Ok(ClaimHistory {
            claim: claim.into(),
            reports: self.of_kind(REPORT).filter(about).map(|n| n.core.cause.clone()).collect(),
            corrections: self.of_kind(CORRECT).filter(about).map(|n| n.core.cause.clone()).collect(),
        })
    }
}
