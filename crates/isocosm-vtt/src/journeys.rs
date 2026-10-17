// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Joint journeys: each recorded proposal for the chosen party, approval by
//! a player's own body and the DM's publication once all have approved.

use chirograph::IntentEffect;
use isocosm::projects::StockSource;
use isocosm_host::endpoint::disclosure::{Item, NULL, action, card, value};
use crate::native::{GameBindings, joint_journey::{self, DepartureKind, DepartureResult, JourneyInput}};

use super::{Act, Built, Cx, VttEndpoint};

fn source(source: StockSource) -> String {
    match source {
        StockSource::Entity(id) => format!("body {id}"),
        StockSource::Site(id) => format!("site {id}"),
        StockSource::Place(id) => format!("place in site {} at ({}, {}, {})", id.site, id.cell[0], id.cell[1], id.cell[2]),
        StockSource::Transit(id) => format!("transfer {}", id.0),
    }
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let mut built = Built::new();
    let note = |built: &mut Built, id: &str, text: String| built.push(Item { id: id.into(),
        card: card("Joint journeys", vec![value("Note", text)], &["note"]), actions: Vec::new() }, Vec::new());
    let Some(party) = cx.party else {
        note(&mut built, "journeys", "Choose an authored party to read journey proposals".into());
        return Ok(built);
    };
    let access = cx.cut.access;
    let views = endpoint.read(|session, bytes| {
        let game = GameBindings::decode(bytes)?;
        let roster = game.shared_roster().ok_or("Configure the saved table player roster")?;
        joint_journey::read_journeys(session, bytes, access, &roster, party)
    });
    let views = match views {
        Ok(views) => views,
        Err(reason) => { note(&mut built, "journeys", reason); return Ok(built); },
    };
    if views.is_empty() { note(&mut built, "journeys", "No recorded proposals for this party.".into()); }
    for view in views {
        let proposal = &view.proposal;
        let event = proposal.witness.event;
        let mut values = vec![value("Party", &proposal.declaration.party), value("Native tick", proposal.declaration.tick),
            value("Approvals", proposal.approvals.len())];
        for departure in &proposal.declaration.departures {
            let input = &departure.input;
            let kind = match departure.kind { DepartureKind::Normative => "Depart", DepartureKind::Goal => "Travel goal" };
            values.push(value(kind, format!("player {}, body {}, {} to {}, process {}, pace {}",
                input.participant, input.body, input.from, input.to, input.process, input.profile)));
            for term in &input.debits {
                let lots = term.lots.iter().map(|lot| format!("{}: {}", lot.key, lot.amount)).collect::<Vec<_>>().join(", ");
                values.push(value("Debit", format!("{} units of {} from {}; lots {lots}; unowned {}",
                    term.amount, term.account, source(term.source), term.unowned)));
            }
        }
        let (mut actions, mut acts) = (Vec::new(), Vec::new());
        if let Some(completed) = &proposal.completed {
            for result in &completed.departures {
                values.push(value("Result", match result {
                    DepartureResult::Departed { body, journey, .. } => format!(
                        "Body {body} departed {} for {}; arrival at native tick {}", journey.from, journey.to, journey.arrives),
                    DepartureResult::Goal { body, .. } => format!(
                        "Body {body} received its travel goal; its own choice determines departure"),
                }));
            }
        } else {
            if let Some(body) = view.can_approve {
                actions.push(action("vtt.journey-approve", &format!("Approve my body {body}"),
                    "Approve this journey for your body.", NULL, IntentEffect::DomainTruth));
                acts.push(Act::Journey(JourneyInput::Approve { proposal: event }));
            }
            if view.ready_for_dm {
                actions.push(action("vtt.journey-apply", "Apply approved journey", "Publish the approved journey.",
                    NULL, IntentEffect::DomainTruth));
                acts.push(Act::Journey(JourneyInput::Apply { proposal: event }));
            }
            if let Some(reason) = &view.refusal { values.push(value("Refused", reason)); }
        }
        built.push(Item { id: format!("journey:{}", event.0), card: card(format!("Proposal {}", event.0), values, &["journey"]),
            actions }, acts);
    }
    Ok(built)
}
