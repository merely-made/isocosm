// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Asks and agreements over the world (rulings 60, 63 and 67): an offer is
//! weighed by the peer asked, and each step of an agreement's life is a
//! deed, so where a peer lives and what it does daily follow from deeds and
//! agreements, never from an order.

use super::*;
use crate::{Result, simulation::Simulation};

impl Simulation {
    /// Where `holder` stands with `toward`, folded from the deeds `toward`
    /// did to it.
    pub fn standing(&self, holder: Id, toward: Id) -> Standing {
        standing_in(&self.genesis.rules, &self.deeds(), holder, toward)
    }

    fn peer(&self, id: Id) -> Result<Entity> {
        let e = self.state.population.get(id).filter(|e| e.alive);
        e.cloned().ok_or_else(|| format!("{id} is no living peer"))
    }

    fn live(&self, id: Id) -> Result<&Agreement> {
        let a = self.state.agreements.get(&id).ok_or("no such agreement")?;
        match a.standing() {
            true => Ok(a),
            false => Err("the agreement has ended".into()),
        }
    }

    /// The peer asked weighs an offer; its answer is a deed.
    pub(crate) fn weigh_offer(&mut self, offer: &Offer) -> Result<Response> {
        self.peer(offer.asked_by)?;
        let peer = self.peer(offer.asked_of)?;
        let deeds = self.deeds();
        let standing = standing_in(&self.genesis.rules, &deeds, offer.asked_of, offer.asked_by);
        let w = willing::weigh(
            &self.genesis.rules,
            &peer,
            offer.asked_by,
            &standing,
            &deeds,
            &offer.work,
            &offer.terms,
        );
        let kind = match w.verdict {
            Verdict::Accept => DeedKind::new(OFFER_ACCEPTED),
            Verdict::Refuse => DeedKind::new(OFFER_REFUSED),
            Verdict::Counteroffer(_) => DeedKind::new(OFFER_COUNTERED),
        };
        let recorded = self.record_deed(offer.asked_of, Some(offer.asked_by), &kind)?;
        let at = self.state.tick;
        Ok(Response {
            by: offer.asked_of,
            at,
            verdict: w.verdict,
            premises: w.premises,
            recorded,
        })
    }

    /// An offer taken as a standing agreement, if the peer accepts it.
    pub(crate) fn form(&mut self, offer: &Offer) -> Result<Ruling> {
        self.peer(offer.asked_by)?;
        let peer = self.peer(offer.asked_of)?;
        let deeds = self.deeds();
        let standing = standing_in(&self.genesis.rules, &deeds, offer.asked_of, offer.asked_by);
        let w = willing::weigh(
            &self.genesis.rules,
            &peer,
            offer.asked_by,
            &standing,
            &deeds,
            &offer.work,
            &offer.terms,
        );
        let (by, at) = (offer.asked_of, self.state.tick);
        if !w.verdict.accepted() {
            let kind = RulingKind::Declined;
            return Ok(Ruling {
                by,
                at,
                kind,
                premises: w.premises,
                recorded: None,
            });
        }
        let id = self
            .state
            .agreements
            .keys()
            .next_back()
            .map_or(0, |k| k + 1);
        let deed = self.record_formation(by, offer.asked_by, id)?;
        let history = vec![AgreementEvent {
            at,
            deed: deed.clone(),
            what: AgreementChange::Formed,
        }];
        self.state.agreements.insert(
            id,
            Agreement {
                id,
                asker: offer.asked_by,
                holder: by,
                work: offer.work.clone(),
                terms: offer.terms,
                formed_at: at,
                state: AgreementState::Standing,
                history,
            },
        );
        let mut premises = w.premises;
        premises.push(Premise::AgreementTerm {
            agreement: id,
            share: offer.terms.share,
            danger_cap: offer.terms.danger_cap,
            covers: true,
        });
        Ok(Ruling {
            by,
            at,
            kind: RulingKind::Formed(id),
            premises,
            recorded: Some(deed),
        })
    }

    /// The holder asked for work under an agreement.
    pub(crate) fn exercise(&mut self, id: Id, work: &Work) -> Result<Ruling> {
        let a = self.live(id)?;
        let (asker, holder, terms, covers) = (a.asker, a.holder, a.terms, a.covers(work));
        let at = self.state.tick;
        let mut premises = vec![Premise::AgreementTerm {
            agreement: id,
            share: terms.share,
            danger_cap: terms.danger_cap,
            covers,
        }];
        if !covers {
            let kind = RulingKind::OutsideTerms;
            return Ok(Ruling {
                by: holder,
                at,
                kind,
                premises,
                recorded: None,
            });
        }
        let peer = self.peer(holder)?;
        let deeds = self.deeds();
        let standing = standing_in(&self.genesis.rules, &deeds, holder, asker);
        let (willing, weighing) =
            willing::weigh_routine(&self.genesis.rules, &peer, asker, &standing, &deeds, work);
        premises.extend(weighing);
        if !willing {
            let kind = RulingKind::Declined;
            return Ok(Ruling {
                by: holder,
                at,
                kind,
                premises,
                recorded: None,
            });
        }
        let deed = self.record_deed(holder, Some(asker), &DeedKind::under(PERFORMED, id))?;
        let what = AgreementChange::Exercised;
        self.change(id, |a| {
            a.history.push(AgreementEvent {
                at,
                deed: deed.clone(),
                what,
            })
        });
        Ok(Ruling {
            by: holder,
            at,
            kind: RulingKind::Performed,
            premises,
            recorded: Some(deed),
        })
    }

    /// A party proposes new terms; the holder weighs a change the asker puts.
    pub(crate) fn propose_change(&mut self, id: Id, by: Id, terms: Terms) -> Result<Ruling> {
        let a = self.live(id)?;
        if !a.party(by) {
            return Err(format!("{by} is no party to the agreement"));
        }
        let (asker, holder, from, other) = (a.asker, a.holder, a.terms, a.other(by));
        let at = self.state.tick;
        let mut premises = vec![Premise::TermChange {
            agreement: id,
            from,
            to: terms,
        }];
        if by == asker {
            let peer = self.peer(holder)?;
            let deeds = self.deeds();
            let standing = standing_in(&self.genesis.rules, &deeds, holder, asker);
            premises.extend(willing::evidence(
                &self.genesis.rules,
                &deeds,
                &standing,
                asker,
            ));
            let caution = caution_of(&peer);
            let limit = willing::bearable(standing.affinity, caution);
            let borne = i16::from(terms.danger_cap) <= limit && terms.share >= from.share;
            premises.push(Premise::DangerWeighed {
                danger: terms.danger_cap,
                bearable: limit,
                affinity: standing.affinity,
                caution,
                borne,
            });
            if !borne {
                let kind = RulingKind::Declined;
                return Ok(Ruling {
                    by: holder,
                    at,
                    kind,
                    premises,
                    recorded: None,
                });
            }
        }
        let deed = self.record_deed(by, Some(other), &DeedKind::under(RENEGOTIATED, id))?;
        let what = AgreementChange::Renegotiated { from, to: terms };
        self.change(id, |a| {
            a.terms = terms;
            a.history.push(AgreementEvent {
                at,
                deed: deed.clone(),
                what,
            });
        });
        let kind = RulingKind::Renegotiated { from, to: terms };
        Ok(Ruling {
            by,
            at,
            kind,
            premises,
            recorded: Some(deed),
        })
    }

    /// A party ends an agreement.
    pub(crate) fn end(&mut self, id: Id, by: Id, why: EndReason) -> Result<Ruling> {
        let a = self.live(id)?;
        if !a.party(by) {
            return Err(format!("{by} is no party to the agreement"));
        }
        let other = a.other(by);
        let at = self.state.tick;
        let deeds = self.deeds();
        let standing = standing_in(&self.genesis.rules, &deeds, by, other);
        let mut premises = willing::evidence(&self.genesis.rules, &deeds, &standing, other);
        premises.push(Premise::Ending { agreement: id, why });
        let deed = self.record_deed(by, Some(other), &DeedKind::under(AGREEMENT_ENDED, id))?;
        self.change(id, |a| {
            a.state = AgreementState::Ended { at, why };
            a.history.push(AgreementEvent {
                at,
                deed: deed.clone(),
                what: AgreementChange::Ended(why),
            });
        });
        Ok(Ruling {
            by,
            at,
            kind: RulingKind::Ended(why),
            premises,
            recorded: Some(deed),
        })
    }

    fn change(&mut self, id: Id, f: impl FnOnce(&mut Agreement)) {
        if let Some(a) = self.state.agreements.get_mut(&id) {
            f(a);
        }
    }

    /// A premise in words, naming peers by the names they were given.
    pub fn say(&self, premise: &Premise) -> String {
        let name = |id: &Id| self.name_of(*id).unwrap_or("someone").to_string();
        let v = super::deed::vocabulary(&self.genesis.rules);
        match premise {
            Premise::Deed {
                at,
                doer,
                kind,
                trust,
                affinity,
                ..
            } => format!(
                "tick {at}: {} {} (trust {trust:+}, liking {affinity:+})",
                name(doer),
                v.phrase(&kind.key)
            ),
            Premise::Standing {
                toward,
                trust,
                affinity,
                from_deeds,
            } => format!(
                "where I stand with {}: trust {trust}, liking {affinity}, out of {} deeds",
                name(toward),
                from_deeds.len()
            ),
            Premise::Confidence {
                craft,
                demanded,
                held,
                margin,
            } => {
                format!(
                    "{} at grade {demanded}; I hold {held} ({margin:+})",
                    craft_name(craft)
                )
            },
            Premise::TrustAsked {
                danger,
                asked,
                held,
                met,
            } => format!(
                "danger {danger} asks trust {asked}; I hold {held} ({})",
                if *met { "enough" } else { "not enough" }
            ),
            Premise::DangerWeighed {
                danger,
                bearable,
                affinity,
                caution,
                borne,
            } => format!(
                "danger {danger} against the {bearable} I would bear for them \
                 (liking {affinity}, caution {caution}): {}",
                if *borne { "within me" } else { "past me" }
            ),
            Premise::AgreementTerm {
                share,
                danger_cap,
                covers,
                ..
            } => format!(
                "our arrangement: share {share}, up to danger {danger_cap} ({})",
                if *covers {
                    "this fits"
                } else {
                    "this is not that"
                }
            ),
            Premise::TermChange { from, to, .. } => format!(
                "terms: share {} -> {}, danger cap {} -> {}",
                from.share, to.share, from.danger_cap, to.danger_cap
            ),
            Premise::Ending { why, .. } => format!("ending: {}", why.phrase()),
        }
    }

    pub fn explain(&self, premises: &[Premise]) -> Vec<String> {
        premises.iter().map(|p| self.say(p)).collect()
    }
}

/// Standing folded from `deeds`: what `toward` did to `holder`.
pub fn standing_in(
    rules: &crate::rules::Rules,
    deeds: &[Deed],
    holder: Id,
    toward: Id,
) -> Standing {
    let v = super::deed::vocabulary(rules);
    let mut s = Standing::default();
    for d in deeds
        .iter()
        .filter(|d| d.doer == toward && d.toward == Some(holder) && holder != toward)
    {
        let (trust, affinity) = v.weight(&d.kind.key);
        s.trust += trust;
        s.affinity += affinity;
        s.from_deeds.push(d.id.clone());
    }
    s
}
