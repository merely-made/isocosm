// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Standing and asks (rulings 51, 56, 60, 63 and 67; Eponym's social family
//! re-expressed in native nouns with its world move, 755): a peer is an
//! entity, its crafts the skills its world's vocabulary declares (778) and
//! its caution a disposition; a deed is
//! an act leaving a world event; standing is folded from deeds; a standing
//! agreement lives in the world's state; a home is a site held under one.
//! **Nothing here commands anyone**: offers are put, answers come back, and
//! a holder whose premises changed declines under its agreement.

use crate::schema::*;
use serde::{Deserialize, Serialize};

mod answer;
mod asks;
mod deed;
mod settlement;
mod terms;
pub mod willing;

pub use answer::{Premise, Response, Ruling, RulingKind, Standing, Verdict, cited_deeds};
pub use asks::standing_in;
pub use deed::{
    AGREEMENT_ENDED, AGREEMENT_FORMED, Deed, DeedKind, DeedRule, OFFER_ACCEPTED, OFFER_COUNTERED,
    OFFER_REFUSED, PERFORMED, RECORDED, RENEGOTIATED, Vocabulary, processes, vocabulary,
};
pub use settlement::{DailyRound, TENANT};
pub use terms::{
    Agreement, AgreementChange, AgreementEvent, AgreementState, CAUTION, Confidence, EndReason,
    Offer, Terms, Work, caution_of, craft_name, grade_of,
};

/// A social act, as the log carries it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Social {
    Deed { doer: Id, toward: Option<Id>, kind: DeedKind },
    Consider(Offer),
    Form(Offer),
    Exercise { agreement: Id, work: Work },
    Renegotiate { agreement: Id, by: Id, terms: Terms },
    End { agreement: Id, by: Id, why: EndReason },
    OfferHome { offer: Offer, dwelling: Id },
}

/// What a social act came to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    Deed(Key),
    Response(Response),
    Ruling(Ruling),
}

impl crate::simulation::Simulation {
    pub(crate) fn social(&mut self, act: &Social) -> crate::Result<Answer> {
        Ok(match act {
            Social::Deed { doer, toward, kind } => Answer::Deed(self.record_deed(*doer, *toward, kind)?),
            Social::Consider(offer) => Answer::Response(self.weigh_offer(offer)?),
            Social::Form(offer) => Answer::Ruling(self.form(offer)?),
            Social::Exercise { agreement, work } => Answer::Ruling(self.exercise(*agreement, work)?),
            Social::Renegotiate { agreement, by, terms } => {
                Answer::Ruling(self.propose_change(*agreement, *by, *terms)?)
            },
            Social::End { agreement, by, why } => Answer::Ruling(self.end(*agreement, *by, *why)?),
            Social::OfferHome { offer, dwelling } => Answer::Ruling(self.offer_home(offer, *dwelling)?),
        })
    }
}
