// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The willingness rule's three gates (directing weighed by opinion, ruling
//! 60): whether the work is within the peer's craft, whether it trusts the
//! asker enough for the danger, and whether the danger is within what it
//! would bear for them. Nothing here commands anyone.

use super::{Confidence, Deed, Premise, Standing, Terms, Verdict, Work, terms::caution_of};
use crate::{rules::Rules, schema::{Entity, Id}};

pub fn trust_asked(danger: u8) -> i16 {
    i16::from(danger)
}

pub fn bearable(affinity: i16, caution: i16) -> i16 {
    3 + affinity / 2 - caution
}

pub struct Weighed {
    pub verdict: Verdict,
    pub premises: Vec<Premise>,
}

/// The deeds behind a standing, then the standing itself.
pub fn evidence(rules: &Rules, deeds: &[Deed], standing: &Standing, toward: Id) -> Vec<Premise> {
    let v = super::deed::vocabulary(rules);
    let mut premises: Vec<Premise> = standing
        .from_deeds
        .iter()
        .filter_map(|id| deeds.iter().find(|d| &d.id == id))
        .map(|deed| {
            let (trust, affinity) = v.weight(&deed.kind.key);
            Premise::Deed {
                deed: deed.id.clone(),
                at: deed.at,
                doer: deed.doer,
                kind: deed.kind.clone(),
                trust,
                affinity,
            }
        })
        .collect();
    premises.push(Premise::Standing {
        toward,
        trust: standing.trust,
        affinity: standing.affinity,
        from_deeds: standing.from_deeds.clone(),
    });
    premises
}

fn confidence(peer: &Entity, work: &Work) -> (bool, Premise) {
    let read = Confidence::read(peer, work);
    let premise = Premise::Confidence {
        craft: read.craft.clone(),
        demanded: read.demanded,
        held: read.held,
        margin: read.margin(),
    };
    (read.sufficient(), premise)
}

fn trust(standing: &Standing, work: &Work) -> (bool, Premise) {
    let asked = trust_asked(work.danger);
    let met = standing.trust >= asked;
    let premise = Premise::TrustAsked {
        danger: work.danger,
        asked,
        held: standing.trust,
        met,
    };
    (met, premise)
}

pub fn weigh(rules: &Rules, peer: &Entity, toward: Id, standing: &Standing, deeds: &[Deed], work: &Work, terms: &Terms) -> Weighed {
    let (able, conf) = confidence(peer, work);
    if !able {
        // Nothing about the asker is in play, so nothing about it is cited.
        let premises = vec![conf];
        return Weighed { verdict: Verdict::Refuse, premises };
    }
    let mut premises = evidence(rules, deeds, standing, toward);
    premises.push(conf);
    let (met, asked) = trust(standing, work);
    premises.push(asked);
    if !met {
        return Weighed { verdict: Verdict::Refuse, premises };
    }
    let caution = caution_of(peer);
    let limit = bearable(standing.affinity, caution);
    let borne = i16::from(work.danger) <= limit;
    premises.push(Premise::DangerWeighed {
        danger: work.danger,
        bearable: limit,
        affinity: standing.affinity,
        caution,
        borne,
    });
    let verdict = match borne {
        true => Verdict::Accept,
        false => Verdict::Counteroffer(Terms::new(
            terms.share.saturating_add(1),
            u8::try_from(limit.clamp(0, 255)).unwrap_or(0),
        )),
    };
    Weighed { verdict, premises }
}

/// Whether a holder does routine work under an agreement: craft and trust,
/// no danger weighed, since the agreement already bounds it.
pub fn weigh_routine(rules: &Rules, peer: &Entity, toward: Id, standing: &Standing, deeds: &[Deed], work: &Work) -> (bool, Vec<Premise>) {
    let (able, conf) = confidence(peer, work);
    let mut premises = evidence(rules, deeds, standing, toward);
    premises.push(conf);
    if !able {
        return (false, premises);
    }
    let (met, asked) = trust(standing, work);
    premises.push(asked);
    (met, premises)
}
