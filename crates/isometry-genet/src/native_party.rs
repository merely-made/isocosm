// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The actual overmap controls use authenticated native participant bindings.

use super::*;
use isometry_views::{PartyRequest, PartyView};
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Intent, IntentEnvelope, Tick,
    vtt::{PartyIntent, PartyAction, VttIntent, VttIntentEnvelope}};
use isonetry::native_table::{TableSender, table_digest};

pub(crate) fn context(snapshot: &GameSnapshot, sender: TableSender)
    -> Result<(isocosm_vtt::founding::Campaign, OrdinaryAccess), String> {
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
            isocosm_vtt::native::party::disclose_known(session, access.participant.0, party)?
        },
        PartyRequest::Decision { party, member, action } => {
            if access.subject != Some(EntityHandle(member)) { return Err("knowledge input names a changed played body".into()); }
            PartyIntent { party, member: EntityHandle(member), action }
        },
        PartyRequest::AuthorStatement(_) | PartyRequest::AuthorLocation(_) => return Err("sources use the actual DM author admission door".into()),
        PartyRequest::Read { .. } | PartyRequest::ReadAt { .. } => return Err("read-only party maps submit no command".into()),
        PartyRequest::Journey(_) => return Err("joint journey uses its separate actual native admission door".into()),
        PartyRequest::Presentation(_) => return Err("overmap presentation uses the explicit host metadata door".into()),
        PartyRequest::Hook(_) => return Err("hook decisions use the actual DM authoring door".into()),
    };
    Ok(IntentEnvelope { participant: access.participant, tick: Tick(session.sim.state().tick),
        intent: Intent::Game(VttIntent::Party(input)) })
}

pub(crate) fn statement_admission(snapshot: &GameSnapshot,
    input: &isocosm_vtt::native::party::StatementInput) -> Result<isonetry::NativeTableAdmissionInput, String> {
    let (campaign, access) = context(snapshot, TableSender::Host)?;
    let table = snapshot.native_table.as_ref().ok_or("statement source needs the admitted native table")?;
    let batch = isocosm_vtt::native::party::statement_admission(campaign.table_session(), &table.game, access, input)?;
    Ok(isonetry::NativeTableAdmissionInput {
        expected_native: snapshot.native.as_ref().ok_or("statement source needs its actual native world")?.cursor()?,
        expected_table: table_digest(Some(table)),
        participant: isocosm_vtt::native::ParticipantInput::Existing(access.participant.0), batch,
    })
}

pub(crate) fn read(snapshot: &GameSnapshot, sender: TableSender, party: &str) -> Result<PartyView, String> {
    read_with_settings(snapshot,sender,party,None)
}

pub(crate) fn read_with_settings(snapshot:&GameSnapshot,sender:TableSender,party:&str,
    settings:Option<isocosm_vtt::native::overmap::ViewSettings>) -> Result<PartyView,String> {
    let (campaign, access) = context(snapshot, sender)?;
    let session = campaign.table_session();
    let own = isocosm::party::members(&session.sim, party)?.into_iter()
        .filter(|m| m.participant == access.participant.0).collect::<Vec<_>>();
    let shared = access.seat != Seat::Player || !own.is_empty();
    let mut map = if shared { isocosm_vtt::native::party::read_party(session, access, party)? }
        else { isocosm::party::SharedMap { party: party.into(), members: own,
            places: Default::default(), routes: Default::default(), events: Default::default(), records: Vec::new(), details: Default::default() } };
    let table = snapshot.native_table.as_ref().ok_or("journey read needs its admitted endpoint table")?;
    let knowledge = isocosm_vtt::native::party::read_knowledge(session, &table.game, access)?;
    let overmap = if shared { Some(isocosm_vtt::native::overmap::read_overmap(session,
        &table.game,access,party,settings)?) } else { None };
    if let Some(read) = &overmap { map = read.map.clone(); }
    let events = map.events.iter().map(|key|
        (key.clone(), session.sim.state().events.get(key).cloned())).collect();
    let journeys = if shared { isocosm_vtt::native::joint_journey::read_journeys(session,
        &table.game,access,&table.time_roster()?,party)? } else { Vec::new() };
    let dm = access.seat == Seat::Dm;
    let hooks=if dm {isocosm_vtt::native::goal_hooks(session,access.subject.map_or(u64::MAX,|id|id.0),
        isocosm::directing::readings::Mode::Creative)?} else {Vec::new()};
    let polities=if dm {isocosm_vtt::native::polity_readings(session,access)?} else {Vec::new()};
    let places=if dm {isocosm_vtt::native::place_readings(session,access)?} else {Vec::new()};
    let mut journey_authoring = Vec::new();
    if dm {
        let roster = table.bound_players();
        for member in &map.members {
            if roster.contains(&member.participant) && session.sim.plays(member.participant) == Some(member.member) {
                if let Some(body) = session.sim.state().population.get(member.member) {
                    journey_authoring.push(format!("Current player {} · body {} · {:?} · resident {:?}",
                        member.participant,member.member,body.method,body.resident_site()));
                }
            }
        }
        for (name,profile) in &session.sim.genesis().rules.pace_profiles {
            journey_authoring.push(format!("Authored profile {name}: {}",
                serde_json::to_string(profile).map_err(|e| e.to_string())?));
        }
        for (name,process) in &session.sim.genesis().rules.processes {
            for effect in process.commitments.iter().chain(&process.effects) {
                if let isocosm::rules::Effect::MovePaced { destination,profile } = effect {
                    if session.sim.genesis().rules.paced_movement(name,*destination).is_ok() {
                        journey_authoring.push(format!("Authored process {name} · destination {destination} · profile {profile}"));
                    }
                }
            }
        }
    }
    // Only this viewer's actual residence may mark "here". No aggregate party location.
    let here = access.subject.filter(|_| overmap.as_ref().is_none_or(|read| read.disclosure.is_none()))
        .and_then(|id| session.sim.state().population.get(id.0))
        .and_then(|body| body.resident_site()).filter(|site| map.places.contains_key(site));
    let view = PartyView { cursor: snapshot.native.as_ref().ok_or("no native campaign")?.cursor()?,
        source: isocosm::digest(snapshot.native.as_ref().ok_or("no native campaign")?),
        table: table_digest(snapshot.native_table.as_ref()), participant: access.participant.0,
        subject: access.subject.map(|id| id.0), shared, map, here, events,
        dm,journeys,journey_authoring,journey_quote:None,overmap,hooks,polities,places,knowledge,
        location_authoring: if dm { session.sim.state().locations.clone() } else { Default::default() } };
    Ok(view)
}

pub(crate) fn presentation_input(snapshot:&GameSnapshot,
    input:&isocosm_vtt::native::overmap::OvermapInput)
    -> Result<isonetry::NativeTableBindingInput,String> {
    let (campaign,access) = context(snapshot,TableSender::Host)?;
    let mut session = campaign.table_session().clone();
    let mut table = snapshot.native_table.as_ref().ok_or("map presentation needs native table bindings")?.clone();
    let (_,game) = isocosm_vtt::native::overmap::submit_presentation(&mut session,table.game.clone(),access,input)?;
    table.game = game;
    Ok(isonetry::NativeTableBindingInput {
        expected_native:snapshot.native.as_ref().ok_or("map presentation needs native source")?.cursor()?,
        expected_table:table_digest(snapshot.native_table.as_ref()),table,
    })
}

pub(crate) fn hook_envelope(snapshot:&GameSnapshot,intent:isocosm_overlay::vtt::HookIntent)
    -> Result<VttIntentEnvelope,String> {
    let (campaign,access)=context(snapshot,TableSender::Host)?;
    Ok(IntentEnvelope {participant:access.participant,tick:Tick(campaign.table_session().sim.state().tick),
        intent:Intent::Game(VttIntent::Hook(intent))})
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
        if let PartyRequest::AuthorLocation(input) = request {
            let before = self.snapshot_of(ctx.runner.state());
            self.ensure_history_origin(&before);
            let mut authority = HostSession::with_history(before, self.campaign.clone(), self.history.clone());
            let result = authority.native_location(&input);
            let accepted = authority.state().clone();
            self.history = authority.history().clone();
            self.journal = accepted.journal.clone();
            ctx.runner.update(|ui| {
                ui.apply_snapshot(accepted);
                ui.native.party.reading = None;
                ui.status = match result { Ok(_) => "admitted native Location; read its actual identity before authoring a learning source".into(), Err(why) => why };
            });
            return;
        }
        if let PartyRequest::AuthorStatement(input) = request {
            let before = self.snapshot_of(ctx.runner.state());
            self.ensure_history_origin(&before);
            let mut authority = HostSession::with_history(before, self.campaign.clone(), self.history.clone());
            let result = statement_admission(authority.state(), &input)
                .and_then(|input| authority.commit_native_table_admission(&input));
            let accepted = authority.state().clone();
            self.history = authority.history().clone();
            self.journal = accepted.journal.clone();
            ctx.runner.update(|ui| {
                ui.apply_snapshot(accepted);
                ui.native.party.reading = None;
                ui.status = match result { Ok(_) => "admitted native statement source; its holder must explicitly learn it".into(), Err(why) => why };
            });
            return;
        }
        if let PartyRequest::ReadAt { party,settings } = request {
            let snapshot = self.snapshot_of(ctx.runner.state());
            let result = read_with_settings(&snapshot,TableSender::Host,&party,Some(settings));
            ctx.runner.update(|ui| ui.status = match result.and_then(|read| install(ui,read)) {
                Ok(())=>"read exact disclosed history; current native authority retained".into(),Err(why)=>why,
            });
            return;
        }
        if let PartyRequest::Presentation(input) = request {
            let before = self.snapshot_of(ctx.runner.state());
            self.ensure_history_origin(&before);
            let mut authority = HostSession::with_history(before,self.campaign.clone(),self.history.clone());
            let result = presentation_input(authority.state(),&input)
                .and_then(|input| authority.commit_native_table_binding(&input));
            let accepted = authority.state().clone();
            self.history = authority.history().clone();
            self.journal = accepted.journal.clone();
            ctx.runner.update(|ui| {
                ui.apply_snapshot(accepted);
                ui.native.party.reading=None;
                ui.status=match result { Ok(_) => "saved native map presentation; read the party map".into(),Err(why)=>why };
            });
            return;
        }
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
            PartyRequest::Hook(intent)=>hook_envelope(authority.state(),intent).and_then(|input|authority.native_hook(&input)),
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
