// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The actual overmap controls use authenticated native participant bindings.

use super::*;
use isometry_views::{PartyRequest, PartyView};
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Intent, IntentEnvelope, Tick,
    vtt::{PartyIntent, PartyAction, VttIntent, VttIntentEnvelope}};
use isonetry::native_table::{TableSender, table_digest};

pub(crate) fn context(snapshot: &GameSnapshot, sender: TableSender)
    -> Result<(isometry_campaign::founding::Campaign, OrdinaryAccess), String> {
    let world = snapshot.native.as_ref().ok_or("party map needs a native campaign")?;
    let table = snapshot.native_table.as_ref().ok_or("party map needs explicit native participant bindings")?;
    let campaign = world.load(isocosm::Execution::Grouped)?;
    let session = campaign.table_session();
    table.validate_history(session)?;
    let participant = table.participant(sender)?;
    let access = OrdinaryAccess { participant: ParticipantHandle(participant),
        seat: if matches!(sender, TableSender::Host) { Seat::Dm } else { Seat::Player },
        subject: session.sim.plays(participant).map(EntityHandle) };
    Ok((campaign, access))
}

pub(crate) fn envelope(snapshot: &GameSnapshot, sender: TableSender, request: PartyRequest)
    -> Result<VttIntentEnvelope, String> {
    let (campaign, access) = context(snapshot, sender)?;
    let session = campaign.table_session();
    let input = match request {
        PartyRequest::Join { party, member } => {
            if access.subject != Some(EntityHandle(member)) { return Err("party opt-in names a changed played body".into()); }
            PartyIntent { party, member: EntityHandle(member), action: PartyAction::Join }
        },
        PartyRequest::Leave { party, member } => PartyIntent { party, member: EntityHandle(member), action: PartyAction::Leave },
        PartyRequest::Disclose { party, member } => {
            if access.subject != Some(EntityHandle(member)) { return Err("party disclosure names a changed played body".into()); }
            isometry_campaign::native::party::disclose_known(session, access.participant.0, party)?
        },
        PartyRequest::Read { .. } => return Err("read-only party maps submit no command".into()),
        PartyRequest::Journey(_) => return Err("joint journey uses its separate actual native admission door".into()),
    };
    Ok(IntentEnvelope { participant: access.participant, tick: Tick(session.sim.state().tick),
        intent: Intent::Game(VttIntent::Party(input)) })
}

pub(crate) fn read(snapshot: &GameSnapshot, sender: TableSender, party: &str) -> Result<PartyView, String> {
    let (campaign, access) = context(snapshot, sender)?;
    let session = campaign.table_session();
    let own = isocosm::party::members(&session.sim, party)?.into_iter()
        .filter(|m| m.participant == access.participant.0).collect::<Vec<_>>();
    let shared = access.seat != Seat::Player || !own.is_empty();
    let map = if shared { isometry_campaign::native::party::read_party(session, access, party)? }
        else { isocosm::party::SharedMap { party: party.into(), members: own,
            places: Default::default(), routes: Default::default(), events: Default::default(), records: Vec::new() } };
    let events = map.events.iter().map(|key|
        (key.clone(), session.sim.state().events.get(key).cloned())).collect();
    let table = snapshot.native_table.as_ref().ok_or("journey read needs its admitted endpoint table")?;
    let journeys = if shared { isometry_campaign::native::joint_journey::read_journeys(session,
        &table.game,access,&table.time_roster()?,party)? } else { Vec::new() };
    let dm = access.seat == Seat::Dm;
    let mut journey_authoring = Vec::new();
    if dm {
        let roster = table.bound_players();
        for member in &map.members {
            if roster.contains(&member.participant) && session.sim.plays(member.participant) == Some(member.member)
                && let Some(body) = session.sim.state().population.get(member.member) {
                journey_authoring.push(format!("Current player {} · body {} · {:?} · resident {:?}",
                    member.participant,member.member,body.method,body.resident_site()));
            }
        }
        for (name,profile) in &session.sim.genesis().rules.pace_profiles {
            journey_authoring.push(format!("Authored profile {name}: {}",
                serde_json::to_string(profile).map_err(|e| e.to_string())?));
        }
        for (name,process) in &session.sim.genesis().rules.processes {
            for effect in process.commitments.iter().chain(&process.effects) {
                if let isocosm::rules::Effect::MovePaced { destination,profile } = effect
                    && session.sim.genesis().rules.paced_movement(name,*destination).is_ok() {
                    journey_authoring.push(format!("Authored process {name} · destination {destination} · profile {profile}"));
                }
            }
        }
    }
    // Only this viewer's actual residence may mark "here". No aggregate party location.
    let here = access.subject.and_then(|id| session.sim.state().population.get(id.0))
        .and_then(|body| body.resident_site()).filter(|site| map.places.contains_key(site));
    let view = PartyView { cursor: snapshot.native.as_ref().ok_or("no native campaign")?.cursor()?,
        source: isocosm::digest(snapshot.native.as_ref().ok_or("no native campaign")?),
        table: table_digest(snapshot.native_table.as_ref()), participant: access.participant.0,
        subject: access.subject.map(|id| id.0), shared, map, here, events,
        dm,journeys,journey_authoring,journey_quote:None };
    Ok(view)
}

fn install(ui: &mut UiState, read: PartyView) -> Result<(), String> {
    if ui.native.party.handle.text().trim() != read.map.party
        || ui.native.world.as_ref().ok_or("party read has no current native world")?.cursor()? != read.cursor
        || isocosm::digest(ui.native.world.as_ref().ok_or("party read has no current native world")?) != read.source
        || table_digest(ui.native.table.as_ref()) != read.table {
        return Err("party map response names a changed handle, native source or participant binding".into());
    }
    ui.native.party.reading = Some(read);
    Ok(())
}

impl App {
    pub(crate) fn pump_native_party(&mut self, ctx: &mut Ctx<'_>) {
        let mut request = None;
        ctx.runner.update(|ui| request = ui.native.party.request.take());
        if let Some(net) = &mut self.net {
            for result in net.take_party_readings() {
                ctx.runner.update(|ui| {
                    ui.status = match result.value.and_then(|view| install(ui, view)) {
                        Ok(()) => "read shared native party map".into(), Err(why) => why,
                    };
                });
            }
            if let Some(request) = request {
                if net.native_party(request).is_none() {
                    ctx.runner.update(|ui| ui.status = "native party authority is unavailable".into());
                }
            }
            return;
        }
        let Some(request) = request else { return; };
        if let PartyRequest::Journey(isometry_views::JourneyRequest::Quote { party,input }) = &request {
            let snapshot = self.snapshot_of(ctx.runner.state());
            let result = crate::native_journey::quote(&snapshot,TableSender::Host,party,input.clone());
            ctx.runner.update(|ui| ui.status = match result.and_then(|view| install(ui,view)) {
                Ok(()) => "read exact native departure stock requests".into(),Err(why) => why,
            });
            return;
        }
        if let PartyRequest::Read { party } = request {
            let snapshot = self.snapshot_of(ctx.runner.state());
            let result = read(&snapshot, TableSender::Host, &party);
            ctx.runner.update(|ui| ui.status = match result.and_then(|view| install(ui, view)) {
                Ok(()) => "read shared native party map".into(), Err(why) => why,
            });
            return;
        }
        let before = self.snapshot_of(ctx.runner.state());
        self.ensure_history_origin(&before);
        let mut authority = HostSession::with_history(before, self.campaign.clone(), self.history.clone());
        let result = match request {
            PartyRequest::Journey(request) => crate::native_journey::envelope(authority.state(),TableSender::Host,request)
                .and_then(|input| authority.native_journey(TableSender::Host,&input)),
            request => envelope(authority.state(),TableSender::Host,request)
                .and_then(|input| authority.native_party(TableSender::Host,&input)),
        };
        let accepted = authority.state().clone();
        self.history = authority.history().clone();
        self.journal = accepted.journal.clone();
        ctx.runner.update(|ui| {
            ui.apply_snapshot(accepted);
            ui.status = match result { Ok(_) => "accepted native party decision; read the shared map".into(), Err(why) => why };
        });
    }
}
