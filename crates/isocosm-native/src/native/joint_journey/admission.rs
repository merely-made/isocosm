// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::super::{GameBindings, time::Roster};
use cosmograph::{schema::Method, projects::{OwnedActed, DebitSelection}};
use cosmograph_overlay::key_for_handle;

pub(super) fn identity(session: &Session, roster: &Roster, party: &str, input: &DepartureInput)
    -> Result<DepartureKind, String> {
    if !roster.players.contains(&input.participant)
        || session.sim.plays(input.participant) != Some(input.body)
        || !cosmograph::party::members(&session.sim,party)?.iter()
            .any(|m| m.participant == input.participant && m.member == input.body) {
        return Err("journey participant/body is not its current bound player's opted-in played body".into());
    }
    let body = session.sim.state().population.get(input.body).ok_or("journey body is absent")?;
    if body.resident_site() != Some(input.from) { return Err("journey body has left its approved residence".into()); }
    let (profile, _) = session.sim.genesis().rules.paced_movement(&input.process,input.to)?;
    if profile != &input.profile { return Err("journey process names another authored pace profile".into()); }
    match body.method {
        Method::Normative => Ok(DepartureKind::Normative),
        Method::Deliberative if session.sim.bond(input.participant,input.body).is_some() => {
            if !input.debits.is_empty() || input.target.is_some() {
                return Err("a critter's site goal uses its own chosen target and scheduled stock rules, not immediate selections".into());
            }
            if session.sim.genesis().rules.processes[&input.process].causation != cosmograph::rules::Causation::Choice {
                return Err("critter travel goal must name a native Choice process".into());
            }
            Ok(DepartureKind::Goal)
        },
        _ => Err("joint travel needs a driven normative body or its actually bonded deliberative critter".into()),
    }
}

/// Real native debit requests for explicit author selection. The caller still
/// supplies named-lot portions/unowned amounts; this read invents none.
pub fn preview_departure(session: &Session, access: OrdinaryAccess, roster: &Roster,
    party: &str, input: &DepartureInput) -> Result<cosmograph::projects::DebitPreview,String> {
    super::super::movement::current_access(session,access)?;
    if access.seat != Seat::Dm || roster.players.contains(&access.participant.0) {
        return Err("only the admitted distinct DM authors joint departure quotes".into());
    }
    roster.validate(session)?;
    if identity(session,roster,party,input)? != DepartureKind::Normative {
        return Err("critter goal costs are admitted when its native scheduled process is chosen".into());
    }
    session.sim.prepare_owned_act(input.body,input.target,&input.process,None)
}

fn selections(session: &Session, input: &DepartureInput) -> Result<Vec<DebitSelection>, String> {
    let quote = session.sim.prepare_owned_act(input.body,input.target,&input.process,None)?;
    if let Some(why) = quote.refusal { return Err(format!("native paced departure refused: {why}")); }
    if !quote.receipt.as_ref().is_some_and(|r| r.accepted()) || quote.requests.len() != input.debits.len() {
        return Err("journey approval lacks the exact actual native debit quote".into());
    }
    input.debits.iter().zip(quote.requests).map(|(term,request)| term.selection(request)).collect()
}

fn admit(session: &mut Session, roster: &Roster, party: &str, departure: &Departure)
    -> Result<DepartureResult, String> {
    let input = &departure.input;
    if identity(session,roster,party,input)? != departure.kind
        || session.sim.paced_journey(input.body,&input.process,input.to)? != departure.journey {
        return Err("native method, route or pace changed since journey approval".into());
    }
    match departure.kind {
        DepartureKind::Normative => {
            let selected = selections(session,input)?;
            let encoded = session.command(Command::ActOwned { participant:input.participant,actor:input.body,
                target:input.target,process:input.process.clone(),cause:None,selections:selected })?;
            let acted: OwnedActed = serde_json::from_str(&encoded).map_err(|e| e.to_string())?;
            if !acted.receipt.accepted() || acted.receipt.actor != input.body
                || acted.receipt.chosen != input.process || acted.receipt.target != input.target
                || session.sim.state().population.get(input.body).and_then(|b| b.journey) != Some(departure.journey) {
                return Err("accepted process did not produce this exact native journey".into());
            }
            Ok(DepartureResult::Departed { participant:input.participant,body:input.body,
                entry:session.entries.last().ok_or("native departure entry is absent")?.id.clone(),journey:departure.journey })
        },
        DepartureKind::Goal => {
            let before = session.sim.state().population.get(input.body).and_then(|b| b.journey);
            session.command(Command::Nudge { participant:input.participant,critter:input.body,
                aim:cosmograph::directing::Aim::Act,toward:cosmograph::directing::Toward::Site(input.to),
                act:Some(input.process.clone()) })?;
            if session.sim.state().population.get(input.body).and_then(|b| b.journey) != before {
                return Err("critter travel goal unexpectedly forced a departure".into());
            }
            Ok(DepartureResult::Goal { participant:input.participant,body:input.body,
                entry:session.entries.last().ok_or("native travel goal entry is absent")?.id.clone() })
        },
    }
}

/// Read-only finite quote on a disposable candidate, including exact ordered
/// native debit admission. No proposal, approval, stock spend or clock publishes.
pub fn quote_departures(session: &Session, roster: &Roster, party: &str, inputs: &[DepartureInput])
    -> Result<Vec<Departure>, String> {
    roster.validate(session)?;
    if inputs.is_empty() { return Err("joint departure group is empty".into()); }
    let mut candidate = session.clone();
    let mut players = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut departures = Vec::new();
    for input in inputs {
        if !players.insert(input.participant) || !bodies.insert(input.body) {
            return Err("joint departure lists a participant or body more than once".into());
        }
        let departure = Departure { input:input.clone(),kind:identity(&candidate,roster,party,input)?,
            journey:candidate.sim.paced_journey(input.body,&input.process,input.to)? };
        admit(&mut candidate,roster,party,&departure)?;
        departures.push(departure);
    }
    Ok(departures)
}

/// Only actual host-authenticated seats supply access/roster. Native history
/// and the mode bytes publish together after the entire private candidate accepts.
pub fn submit_journey(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess, roster: &Roster,
    input: &JourneyEnvelope) -> Result<(String, Vec<u8>), String> {
    super::super::movement::current_access(session,access)?;
    roster.validate(session)?;
    if input.participant != access.participant.0 || input.tick != session.sim.state().tick {
        return Err("journey request names another authenticated participant or native tick".into());
    }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    if roster.revision != bindings.roster_revision() || roster.preimage != cosmograph::digest(&roster.players) {
        return Err("journey roster differs from the accepted native players/revision".into());
    }
    let dm = access.seat == Seat::Dm && !roster.players.contains(&input.participant);
    let mut candidate = session.clone();
    let accepted = match &input.intent {
        JourneyInput::Propose { party,departures } => {
            if !dm { return Err("only the distinct DM declares this explicit joint departure".into()); }
            let declaration = Declaration { proposer:input.participant,tick:input.tick,party:party.clone(),roster:roster.clone(),
                departures:quote_departures(&candidate,roster,party,departures)? };
            let witness = super::records::append(&mut candidate,&Record::Proposed(declaration.clone()))?;
            let key = witness.event_key.clone();
            bindings.record_journey(key.clone(),Proposal { declaration,witness,approvals:Vec::new(),completed:None });
            format!("joint journey proposed: {key}")
        },
        JourneyInput::Approve { proposal } | JourneyInput::Apply { proposal } => {
            let key = key_for_handle(candidate.sim.state().events.keys().map(String::as_str),proposal.0)
                .ok_or("joint journey handle has no unique actual native event")?.to_owned();
            let mut held = bindings.journeys().get(&key).ok_or("event is not a recorded joint journey proposal")?.clone();
            held.require_current(&candidate,roster)?;
            held.validate(&candidate,&key)?;
            let accepted: String = if matches!(&input.intent,JourneyInput::Approve { .. }) {
                if access.seat != Seat::Player || !roster.players.contains(&input.participant) {
                    return Err("only the authenticated player approves their own currently played body".into());
                }
                let departure = held.declaration.departures.iter().find(|d| d.input.participant == input.participant)
                    .ok_or("player is not explicitly listed in this departure")?;
                if identity(&candidate,roster,&held.declaration.party,&departure.input)? != departure.kind
                    || candidate.sim.paced_journey(departure.input.body,&departure.input.process,departure.input.to)?
                        != departure.journey {
                    return Err("body method, residence, route or authored pace changed before approval".into());
                }
                if held.approvals.iter().any(|a| a.participant == input.participant) {
                    return Err("this player's departure approval is already recorded".into());
                }
                let body = departure.input.body;
                let witness = super::records::append(&mut candidate,&Record::Approved { participant:input.participant,
                    body,proposal:key.clone(),event:*proposal })?;
                held.approvals.push(Approval { participant:input.participant,body,witness });
                "your current body's exact departure or travel goal is approved".into()
            } else {
                if !dm || held.declaration.proposer != input.participant { return Err("only the proposing DM applies this group".into()); }
                if held.declaration.departures.iter().any(|d| !held.approvals.iter()
                    .any(|a| a.participant == d.input.participant && a.body == d.input.body)) {
                    return Err("each explicitly listed player must approve their current body".into());
                }
                let mut results = Vec::new();
                for departure in &held.declaration.departures {
                    results.push(admit(&mut candidate,roster,&held.declaration.party,departure)?);
                }
                let witness = super::records::append(&mut candidate,&Record::Applied { participant:input.participant,
                    proposal:key.clone(),event:*proposal,departures:results.clone() })?;
                held.completed = Some(Completion { participant:input.participant,witness,departures:results });
                "approved native departures and critter goals published together; native time unchanged".into()
            };
            bindings.record_journey(key,held);
            accepted
        },
    };
    bindings.validate(&candidate)?;
    let game = bindings.encode()?;
    *session = candidate;
    Ok((accepted,game))
}
