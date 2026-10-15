// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Individual native journey approvals and explicit DM publication.

use crate::mode_sections::Games as _;
use crate::products::Submit as _;
use cambium::{el, text};
use isometry_campaign::native::{GameBindings,
    joint_journey::{self, JourneyEnvelope, JourneyInput, JourneyView, DepartureKind, DepartureResult}};
use super::{model::{App, Defaults}, view::{Child,button}};
use crate::products::{Input,Mode};

impl App {
    fn vtt_journey(&mut self,key:u64,intent:JourneyInput) -> Result<(),String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        let tick=self.world.read(|session|session.sim.state().tick)?;
        let envelope=JourneyEnvelope { participant:access.participant.0,tick,intent };
        let input=Input { mode:Mode::Vtt,bytes:serde_json::to_vec(&envelope).map_err(|error|error.to_string())?,
            round:0,region:None };
        local.native.submit_product(&input)?;
        self.refresh();
        Ok(())
    }
}

fn read(app:&App,key:u64) -> Result<Vec<JourneyView>,String> {
    let local=app.windows.get(&key).ok_or("window is closed")?;
    let access=local.native.projection()?.access;
    let defaults:Defaults=serde_json::from_slice(&local.native.binding().view_defaults)
        .map_err(|error|error.to_string())?;
    let party=defaults.party.ok_or("Choose an authored party to read journey proposals")?;
    app.world.read_game(Mode::Vtt,|session,bytes| {
        let game=GameBindings::decode(bytes)?;
        let roster=game.shared_roster().ok_or("Configure the saved table player roster")?;
        joint_journey::read_journeys(session,bytes,access,&roster,&party)
    })
}

pub(super) fn panel(app:&App,key:u64) -> Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h3",text("Joint journeys")))];
    match read(app,key) {
        Ok(views)=> {
            if views.is_empty() { rows.push(Box::new(el("p",text("No recorded proposals for this party.")))); }
            for view in views {
                let proposal=view.proposal;
                let event=proposal.witness.event;
                rows.push(Box::new(el("p",text(format!("Proposal {} for {} at native tick {}",
                    event.0,proposal.declaration.party,proposal.declaration.tick)))));
                for departure in &proposal.declaration.departures {
                    let input=&departure.input;
                    let action=match departure.kind { DepartureKind::Normative=>"Depart",DepartureKind::Goal=>"Travel goal" };
                    rows.push(Box::new(el("p",text(format!("{action}: player {}, body {}, {} to {}, process {}, pace {}",
                        input.participant,input.body,input.from,input.to,input.process,input.profile)))));
                    for term in &input.debits {
                        let lots=term.lots.iter().map(|lot|format!("{}: {}",lot.key,lot.amount))
                            .collect::<Vec<_>>().join(", ");
                        rows.push(Box::new(el("p",text(format!("{} units of {} from {}; lots {lots}; unowned {}",
                            term.amount,term.account,source(term.source),term.unowned)))));
                    }
                }
                rows.push(Box::new(el("p",text(format!("{} recorded approvals",proposal.approvals.len())))));
                if let Some(completed)=&proposal.completed {
                    for result in &completed.departures {
                        let accepted=match result {
                            DepartureResult::Departed {body,journey,..}=>format!(
                                "Body {body} departed {} for {}; arrival at native tick {}",
                                journey.from,journey.to,journey.arrives),
                            DepartureResult::Goal {body,..}=>format!(
                                "Body {body} received its travel goal; its own choice determines departure"),
                        };
                        rows.push(Box::new(el("p",text(accepted))));
                    }
                } else {
                    if let Some(body)=view.can_approve {
                        rows.push(button(format!("journey-approve-{}",event.0),format!("Approve my body {body}"),key,
                            move |app|app.vtt_journey(key,JourneyInput::Approve{proposal:event})));
                    }
                    if view.ready_for_dm {
                        rows.push(button(format!("journey-apply-{}",event.0),"Apply approved journey".into(),key,
                            move |app|app.vtt_journey(key,JourneyInput::Apply{proposal:event})));
                    }
                    if let Some(reason)=view.refusal { rows.push(Box::new(el("p",text(reason)))); }
                }
            }
        },
        Err(reason)=>rows.push(Box::new(el("p",text(reason)))),
    }
    rows.push(Box::new(el("p",text("The DM proposes an explicit journey using the chosen table intent file. Native time advances through the separate downtime controls."))));
    Box::new(el("section",rows).attr("class","panel"))
}

fn source(source:isocosm::projects::StockSource) -> String {
    use isocosm::projects::StockSource;
    match source {
        StockSource::Entity(id)=>format!("body {id}"),
        StockSource::Site(id)=>format!("site {id}"),
        StockSource::Place(id)=>format!("place in site {} at ({}, {}, {})",id.site,id.cell[0],id.cell[1],id.cell[2]),
        StockSource::Transit(id)=>format!("transfer {}",id.0),
    }
}
