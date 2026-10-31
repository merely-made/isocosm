// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::super::{GameBindings,time::Roster};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyView {
    pub proposal: Proposal, pub can_approve: Option<Id>,
    pub ready_for_dm: bool, pub refusal: Option<String>,
}

/// Same admitted native Session and player-roster source as submission.
/// Retained records grant neither geography knowing nor new body control.
pub fn read_journeys(session: &Session, game: &[u8], access: OrdinaryAccess,
    roster: &Roster, party: &str) -> Result<Vec<JourneyView>,String> {
    super::super::party::read_party(session,access,party)?;
    roster.validate(session)?;
    let bindings = GameBindings::decode(game)?;
    bindings.validate(session)?;
    if roster.revision != bindings.roster_revision() || roster.preimage != cosmograph::digest(&roster.players) {
        return Err("joint journey read names another saved player-roster revision".into());
    }
    let mut views = Vec::new();
    for proposal in bindings.journeys().values().filter(|p| p.declaration.party == party) {
        let current = proposal.require_current(session,roster);
        let can_approve = if current.is_ok() && access.seat == Seat::Player
            && !proposal.approvals.iter().any(|a| a.participant == access.participant.0) {
            proposal.declaration.departures.iter().find(|d| d.input.participant == access.participant.0)
                .filter(|d| super::admission::identity(session,roster,party,&d.input).ok() == Some(d.kind))
                .map(|d| d.input.body)
        } else { None };
        let all = proposal.declaration.departures.iter().all(|d| proposal.approvals.iter()
            .any(|a| a.participant == d.input.participant && a.body == d.input.body));
        let readiness = current.and_then(|_| {
            if !all { return Err("awaiting each listed player's own approval".into()); }
            let inputs = proposal.declaration.departures.iter().map(|d| d.input.clone()).collect::<Vec<_>>();
            if quote_departures(session,roster,party,&inputs)? != proposal.declaration.departures {
                return Err("current native departure quote differs from the signed proposal".into());
            }
            Ok(())
        });
        let ready_for_dm = access.seat == Seat::Dm && access.participant.0 == proposal.declaration.proposer
            && readiness.is_ok();
        views.push(JourneyView { proposal:proposal.clone(),can_approve,ready_for_dm,refusal:readiness.err() });
    }
    Ok(views)
}
