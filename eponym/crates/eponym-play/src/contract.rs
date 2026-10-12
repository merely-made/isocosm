// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym through its contract (wing rulings 686 and 794): every act of
//! play, an `isocosm_overlay::eponym` intent or handoff, becomes native
//! commands here and nowhere else. The driven sophont's actuation (232,
//! 233, 779), its asks, agreements, deeds and tellings, the player's acts,
//! the checkpoint answers, and the harm the foreground settled (669).
//! Founding, the host's arrivals, bodies and authored content, goes
//! through [`World::found`].

use isocosm::harm::Wounded;
use isocosm::history::Command;
use isocosm::knowing::{Knowing, Proposition};
use isocosm::schema::{Id, Key};
use isocosm::social::{self, DeedKind, Offer, Social};
use isocosm_overlay::eponym::{
    ActTarget, Actuation, AgreementAct, AgreementHandle, EndReason, EponymHandoff, EponymIntent,
    LifeCheckpoint, LifeChoice, Manner, PlayerActKind, Proposal, StartTime, Successor, Term,
    TermSide, WorkKey,
};
use isocosm_overlay::{EntityHandle, Harm, Pointable, WoundSeverity};
use isometer_space::{Edit, Op, edit::Shape};
use serde::Deserialize;

use crate::founding::{DROP, SPENT};
use crate::{World, WorldError};

mod borrowed;
mod owner;
pub use borrowed::{Borrowed, Reading};

/// The timed act that carves the ground at a region.
pub const CARVE: &str = "eponym:carve";
/// The term keys a proposal carries native work and terms by.
pub const GRADE: &str = "grade";
pub const DANGER: &str = "danger";
pub const SHARE: &str = "share";
pub const DANGER_CAP: &str = "danger-cap";

/// What lowering came to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lowered {
    /// Each command's outcome, in order.
    pub outcomes: Vec<String>,
    /// What each wound did.
    pub wounded: Vec<Wounded>,
    /// The participant a first life joined as.
    pub joined: Option<Id>,
}

/// A telling's claim as the sim reads its bytes: the claim reported and
/// how it reads.
#[derive(Deserialize)]
struct AsTold {
    claim: Key,
    proposition: Proposition,
}

impl Borrowed<'_> {
    /// Lowers one intent. With a participant who plays a sophont, only that
    /// sophont acts (152); with none, the host's scripted bodies do.
    pub fn submit(
        &mut self,
        participant: Option<Id>,
        intent: &EponymIntent,
    ) -> Result<Lowered, WorldError> {
        let mut next = (*self.session).clone();
        let lowered = Borrowed::over_volume(&mut next, self.site, &self.volume)
            .submit_inner(participant, intent)?;
        *self.session = next;
        Ok(lowered)
    }

    fn submit_inner(
        &mut self,
        participant: Option<Id>,
        intent: &EponymIntent,
    ) -> Result<Lowered, WorldError> {
        let played = participant.and_then(|p| self.sim().plays(p));
        let check = |actor: EntityHandle| match (participant, played) {
            (Some(_), None) => Err(WorldError::NotPlayed),
            (_, Some(p)) if p != actor.0 => Err(WorldError::NotPlayed),
            _ => Ok(()),
        };
        let mut out = Lowered::default();
        let social = match intent {
            EponymIntent::Project(act) => {
                check(act.subject)?;
                let participant = participant.ok_or(WorldError::NotPlayed)?;
                let outcome = self.run(Command::Project(isocosm::projects::Change::Act {
                    participant,
                    actor: act.subject.0,
                    target: act.target.map(|target| target.0),
                    process: act.process.0.clone(),
                    project: act.project.map(|project| isocosm::projects::ProjectId(project.0)),
                    cause: None,
                }))?;
                let acted: isocosm::projects::Acted =
                    serde_json::from_str(&outcome).map_err(|_| WorldError::Decode)?;
                if !acted.receipt.accepted() { return Err(WorldError::Refused); }
                out.outcomes.push(outcome);
                return Ok(out);
            },
            EponymIntent::Drive(a) => {
                check(a.body)?;
                self.drive(a, &mut out)?;
                return Ok(out);
            },
            EponymIntent::Ask(p) => {
                check(p.from)?;
                match p.under {
                    None => Social::Consider(offer(p)?),
                    Some(a) => {
                        let held = self
                            .sim()
                            .state()
                            .agreements
                            .get(&a.0)
                            .ok_or(WorldError::Refused)?;
                        if held.asker != p.from.0 || held.holder != p.to.0 {
                            return Err(WorldError::Refused);
                        }
                        let offered = offer(p)?;
                        if held.terms != offered.terms {
                            return Err(WorldError::Refused);
                        }
                        Social::Exercise {
                            agreement: a.0,
                            work: offered.work,
                        }
                    },
                }
            },
            EponymIntent::Agree(act) => {
                let (by, social) = agreement(act)?;
                check(by)?;
                social
            },
            EponymIntent::Deed(d) => {
                check(d.doer)?;
                let kind = DeedKind {
                    key: d.kind.0.clone(),
                    agreement: d.under.map(|a| a.0),
                };
                Social::Deed {
                    doer: d.doer.0,
                    toward: d.toward.map(|t| t.0),
                    kind,
                }
            },
            EponymIntent::Tell(t) => {
                check(t.from)?;
                // The sim weighs no manner yet (241).
                if t.manner != Manner::Plain {
                    return Err(WorldError::Unbuilt("a telling's manner"));
                }
                let told: AsTold =
                    serde_json::from_slice(&t.claim.as_told).map_err(|_| WorldError::Decode)?;
                let Pointable::Event(handle) = t.claim.about else {
                    return Err(WorldError::Refused);
                };
                let named = isocosm_overlay::key_for_handle(
                    self.sim().state().events.keys().map(String::as_str),
                    handle.0,
                );
                if named != Some(told.proposition.event.as_str()) {
                    return Err(WorldError::Refused);
                }
                let (reporter, hearer) = (t.from.0, t.to.0);
                let report = Knowing::Report {
                    reporter,
                    hearer,
                    claim: told.claim,
                    proposition: told.proposition,
                };
                out.outcomes.push(self.run(Command::Know(report))?);
                return Ok(out);
            },
            EponymIntent::Know(k) => {
                check(k.subject)?;
                let record: Knowing =
                    serde_json::from_slice(&k.record).map_err(|_| WorldError::Decode)?;
                let subject = match &record {
                    Knowing::Observe { observer, .. } => *observer,
                    Knowing::Claim { claimant, .. } => *claimant,
                    Knowing::Correct { corrector, .. } => *corrector,
                    Knowing::Report { .. } => {
                        return Err(WorldError::Unbuilt("a report outside a telling"));
                    },
                };
                if subject != k.subject.0 {
                    return Err(WorldError::NotPlayed);
                }
                out.outcomes.push(self.run(Command::Know(record))?);
                return Ok(out);
            },
            EponymIntent::Act(a) => {
                check(a.subject)?;
                let PlayerActKind::Name {
                    of: Pointable::Entity(of),
                    name,
                } = &a.kind
                else {
                    return Err(WorldError::Unbuilt(
                        "a note, or a name for what is not an entity",
                    ));
                };
                let name = Command::Name {
                    by: a.subject.0,
                    of: of.0,
                    name: name.clone(),
                };
                out.outcomes.push(self.run(name)?);
                return Ok(out);
            },
            EponymIntent::Checkpoint(c) => {
                self.answer(participant, played, c, &mut out)?;
                return Ok(out);
            },
            // Control is the game's until dramatis takes it (803).
            EponymIntent::Creative(_) => return Err(WorldError::Unbuilt("creative tag-in")),
        };
        out.outcomes.push(self.run(Command::Social(social))?);
        Ok(out)
    }

    /// Lowers what the foreground settled about `subject` (232, 669).
    pub fn hand_back(
        &mut self,
        subject: Id,
        outcome: &EponymHandoff,
    ) -> Result<Lowered, WorldError> {
        let mut next = (*self.session).clone();
        let lowered = Borrowed::over_volume(&mut next, self.site, &self.volume)
            .hand_back_inner(subject, outcome)?;
        *self.session = next;
        Ok(lowered)
    }

    fn hand_back_inner(
        &mut self,
        subject: Id,
        outcome: &EponymHandoff,
    ) -> Result<Lowered, WorldError> {
        let mut out = Lowered::default();
        match outcome {
            EponymHandoff::Blow(b) => {
                if b.target.0 != subject {
                    return Err(WorldError::Refused);
                }
                self.harm(subject, &b.harm, &mut out)?;
            },
            EponymHandoff::Hurt(h) => self.harm(subject, h, &mut out)?,
            EponymHandoff::Dropped(items) => {
                for item in items {
                    out.outcomes.push(self.act(subject, Some(item.0), DROP)?);
                }
            },
        }
        Ok(out)
    }

    /// The body's timed acts; its motion is the game's, in-site space being
    /// isometer's and the sim site-grained (670).
    fn drive(&mut self, a: &Actuation, out: &mut Lowered) -> Result<(), WorldError> {
        let body = a.body.0;
        if let Some(motion) = a.motion {
            if motion.at.0 != self.site() {
                return Err(WorldError::Refused);
            }
            let places = isometer_space::places::Places::derive(self.volume(), Default::default())
                .map_err(|_| WorldError::Refused)?;
            let patch = places.place_at(motion.position.0.map(i64::from));
            out.outcomes.push(self.run(Command::Patch {
                entity: body,
                patch,
            })?);
        }
        for act in &a.acts {
            let key = act.act.0.as_str();
            let outcome = match act.target {
                Some(ActTarget::Region { centre, radius }) if key == CARVE => {
                    let at = crate::walking::to_site(self.volume(), centre.0);
                    let edit = Edit {
                        op: Op::Carve,
                        shape: Shape::Sphere {
                            centre: at,
                            radius: u64::from(radius),
                        },
                    };
                    self.run(Command::Edit {
                        site: self.site(),
                        edit,
                        by: Some(body),
                    })?
                },
                // An item the body uses up runs out itself.
                Some(ActTarget::Thing(item)) if key == SPENT => self.act(item.0, None, SPENT)?,
                Some(ActTarget::Thing(thing)) => self.act(body, Some(thing.0), key)?,
                None => self.act(body, None, key)?,
                Some(_) => return Err(WorldError::Unbuilt("a timed act at a place or region")),
            };
            out.outcomes.push(outcome);
        }
        Ok(())
    }

    /// Harm as wounds: a severed part through every cell, and vigour
    /// drained as cells lost from a part the sim draws, until the sim keeps
    /// vigour of its own (checkpoint 10).
    fn harm(&mut self, entity: Id, harm: &Harm, out: &mut Lowered) -> Result<(), WorldError> {
        let mut wounds = Vec::new();
        for w in &harm.wounds {
            if w.severity != WoundSeverity::Severed {
                return Err(WorldError::Unbuilt("a wound short of severing"));
            }
            let part =
                isometer_core::PartId(u32::try_from(w.part.0).map_err(|_| WorldError::Decode)?);
            wounds.push((Some(part), u32::MAX));
        }
        if harm.vigour_drained > 0 {
            wounds.push((None, u32::try_from(harm.vigour_drained).unwrap_or(u32::MAX)));
        }
        for (part, cells) in wounds {
            let outcome = self.run(Command::Wound {
                entity,
                part,
                cells,
            })?;
            out.wounded
                .push(serde_json::from_str(&outcome).map_err(|_| WorldError::Decode)?);
            out.outcomes.push(outcome);
        }
        Ok(())
    }

    /// Takes up a life (779): the first joins the sim as a participant.
    fn answer(
        &mut self,
        participant: Option<Id>,
        played: Option<Id>,
        c: &LifeCheckpoint,
        out: &mut Lowered,
    ) -> Result<(), WorldError> {
        let critter = match c {
            LifeCheckpoint::First(f) => {
                if f.start != StartTime::Society {
                    return Err(WorldError::Unbuilt("a first-life start time"));
                }
                match f.begins_as {
                    LifeChoice::Denizen(e) => e.0,
                    _ => return Err(WorldError::Unbuilt("an outsider or a birth: founding")),
                }
            },
            LifeCheckpoint::Death(s) => {
                if played.is_some_and(|p| p != s.of.0) {
                    return Err(WorldError::NotPlayed);
                }
                match s.next {
                    Successor::Companion(e) | Successor::Another(LifeChoice::Denizen(e)) => e.0,
                    _ => return Err(WorldError::Unbuilt("a successor not yet in the world")),
                }
            },
        };
        let participant = match (participant, c) {
            (Some(p), _) => p,
            (None, LifeCheckpoint::First(_)) => {
                let joined = self.run(Command::Join)?;
                let id = joined
                    .strip_prefix("participant:")
                    .and_then(|s| s.parse().ok());
                *out.joined.insert(id.ok_or(WorldError::Decode)?)
            },
            (None, LifeCheckpoint::Death(_)) => return Err(WorldError::NotPlayed),
        };
        out.outcomes.push(self.run(Command::Take {
            participant,
            critter,
        })?);
        Ok(())
    }

    /// `actor` runs `process`; refused unless the sim accepted it.
    fn act(&mut self, actor: Id, target: Option<Id>, process: &str) -> Result<String, WorldError> {
        let outcome = self.run(Command::Act {
            actor,
            target,
            process: process.into(),
            cause: None,
        })?;
        let receipt: isocosm::simulation::Receipt =
            serde_json::from_str(&outcome).map_err(|_| WorldError::Decode)?;
        receipt
            .accepted()
            .then_some(outcome)
            .ok_or(WorldError::Refused)
    }
}

/// A native agreement act and who takes it.
fn agreement(act: &AgreementAct) -> Result<(EntityHandle, Social), WorldError> {
    Ok(match act {
        AgreementAct::Form(p) => {
            if p.under.is_some() {
                return Err(WorldError::Refused);
            }
            (p.from, Social::Form(offer(p)?))
        },
        AgreementAct::Renegotiate {
            agreement,
            by,
            terms: t,
        } => {
            validate_terms(t, false)?;
            let terms = native_terms(t);
            (
                *by,
                Social::Renegotiate {
                    agreement: agreement.0,
                    by: by.0,
                    terms,
                },
            )
        },
        AgreementAct::End { agreement, by, why } => (
            *by,
            Social::End {
                agreement: agreement.0,
                by: by.0,
                why: native_end(*why),
            },
        ),
        AgreementAct::Home { proposal, dwelling } => {
            if proposal.under.is_some() {
                return Err(WorldError::Refused);
            }
            (
                proposal.from,
                Social::OfferHome {
                    offer: offer(proposal)?,
                    dwelling: dwelling.0,
                },
            )
        },
    })
}

fn validate_terms(terms: &[Term], work: bool) -> Result<(), WorldError> {
    let mut seen = std::collections::BTreeSet::new();
    for term in terms {
        let native = matches!(
            (term.side, term.what.as_str()),
            (TermSide::Offered, SHARE) | (TermSide::Asked, DANGER_CAP)
        ) || (work
            && term.side == TermSide::Asked
            && matches!(term.what.as_str(), GRADE | DANGER));
        if !native {
            return Err(WorldError::Unbuilt(
                "a term outside native work and agreement terms",
            ));
        }
        if term.count > u64::from(u8::MAX) || !seen.insert((term.side as u8, term.what.as_str())) {
            return Err(WorldError::Refused);
        }
    }
    Ok(())
}

fn term(terms: &[Term], side: TermSide, key: &str) -> u8 {
    let found = terms.iter().find(|t| t.side == side && t.what == key);
    found.map_or(0, |t| u8::try_from(t.count).unwrap_or(u8::MAX))
}

fn native_terms(t: &[Term]) -> social::Terms {
    social::Terms::new(
        term(t, TermSide::Offered, SHARE),
        term(t, TermSide::Asked, DANGER_CAP),
    )
}

/// A proposal's native offer: its work's grade and danger, and the share
/// and danger cap, ride as terms by key.
fn offer(p: &Proposal) -> Result<Offer, WorldError> {
    validate_terms(&p.terms, true)?;
    let work = social::Work::new(
        p.work.0.clone(),
        term(&p.terms, TermSide::Asked, GRADE),
        term(&p.terms, TermSide::Asked, DANGER),
    );
    Ok(Offer::new(p.from.0, p.to.0, work, native_terms(&p.terms)))
}

fn native_end(why: EndReason) -> social::EndReason {
    match why {
        EndReason::Withdrawn => social::EndReason::Withdrawn,
        EndReason::Resigned => social::EndReason::Resigned,
        EndReason::WorkDone => social::EndReason::WorkDone,
        EndReason::PremisesChanged => social::EndReason::PremisesChanged,
    }
}

/// A native offer as the contract's proposal, the inverse of lowering,
/// for hosts that author in native terms.
pub fn proposal(o: &Offer, under: Option<Id>) -> Proposal {
    Proposal {
        from: EntityHandle(o.asked_by),
        to: EntityHandle(o.asked_of),
        work: WorkKey(o.work.craft.clone()),
        terms: terms(&o.terms, o.work.grade, o.work.danger),
        under: under.map(AgreementHandle),
    }
}

/// Native terms as the contract's, with a work's grade and danger.
pub fn terms(t: &social::Terms, grade: u8, danger: u8) -> Vec<Term> {
    let mut terms = agreement_terms(t);
    terms.extend(
        [(GRADE, grade), (DANGER, danger)].map(|(what, count)| Term {
            side: TermSide::Asked,
            what: what.into(),
            count: u64::from(count),
        }),
    );
    terms
}

/// An agreement's held terms without a work proposal.
pub fn agreement_terms(t: &social::Terms) -> Vec<Term> {
    let term = |side, what: &str, count: u8| Term {
        side,
        what: what.into(),
        count: u64::from(count),
    };
    vec![
        term(TermSide::Offered, SHARE, t.share),
        term(TermSide::Asked, DANGER_CAP, t.danger_cap),
    ]
}

/// A native end reason as the contract's.
pub fn end_reason(why: social::EndReason) -> EndReason {
    match why {
        social::EndReason::Withdrawn => EndReason::Withdrawn,
        social::EndReason::Resigned => EndReason::Resigned,
        social::EndReason::WorkDone => EndReason::WorkDone,
        social::EndReason::PremisesChanged => EndReason::PremisesChanged,
    }
}

#[cfg(test)]
mod tests;
