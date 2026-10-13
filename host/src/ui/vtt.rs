// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual saved-roster, individual time and explicit party controls for this host.

use cambium::{el, text};
use isocosm_overlay::{EntityHandle, EventHandle, Intent, Seat, Tick,
    vtt::{PartyAction, PartyIntent, TimeIntent, VttIntent, VttIntentEnvelope}};
use isometry_campaign::native::{GameBindings, SharedRosterInput, time};
use super::{model::{App, Defaults}, view::{Child, button}};
use crate::products::{Input, Mode};

impl App {
    pub(super) fn vtt_bytes(&mut self, key:u64, bytes:Vec<u8>) -> Result<(), String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let input = Input { mode:Mode::Vtt, bytes, round:0, region:None };
        local.native.submit_product(&input)?;
        self.refresh();
        Ok(())
    }

    pub(super) fn vtt_send(&mut self, key: u64, intent: VttIntent) -> Result<(), String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let access = local.native.projection()?.access;
        let tick = self.world.read(|session| session.sim.state().tick)?;
        let envelope = VttIntentEnvelope { participant:access.participant, tick:Tick(tick),
            intent:Intent::Game(intent) };
        self.vtt_bytes(key, serde_json::to_vec(&envelope).map_err(|e| e.to_string())?)
    }

    fn vtt_roster(&mut self, key:u64, player:Option<u64>) -> Result<(), String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let input = self.world.read_game(Mode::Vtt, |_, bytes| {
            let game = GameBindings::decode(bytes)?;
            let mut players = game.shared_players().cloned().unwrap_or_default();
            if let Some(player) = player { if !players.remove(&player) { players.insert(player); } }
            let input = SharedRosterInput { version:1, expected_roster_revision:game.roster_revision(), players };
            Ok(Input { mode:Mode::Vtt, bytes:serde_json::to_vec(&input).map_err(|e| e.to_string())?,
                round:0, region:None })
        })?;
        local.native.submit_product(&input)?;
        self.refresh();
        Ok(())
    }

    fn vtt_party(&mut self, key:u64, party:String) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let mut defaults:Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|error| error.to_string())?;
        defaults.party = Some(party);
        local.native.set_view_defaults(serde_json::to_vec(&defaults).map_err(|e| e.to_string())?)?;
        local.preferences.modes.insert("vtt".into(), defaults);
        self.refresh();
        Ok(())
    }

    fn vtt_disclose(&mut self, key:u64, party:String, member:u64) -> Result<(), String> {
        let access = self.windows.get(&key).ok_or("window is closed")?.native.projection()?.access;
        let input = self.world.read(|session| {
            isometry_campaign::native::party::disclose_known(session, access.participant.0, party)
        })??;
        if input.member.0 != member { return Err("played body changed before map disclosure".into()); }
        self.vtt_send(key, VttIntent::Party(input))
    }
}

struct Reading {
    seat:Seat, participant:u64, subject:Option<u64>, game:GameBindings,
    players:Vec<u64>, tick:u64, parties:Vec<String>, selected:Option<String>,
    own_members:Vec<u64>, map:Result<isocosm::party::SharedMap,String>,
    proposals:Vec<(EventHandle,u64,String,bool)>, event_lines:Vec<String>,
}

fn read(app:&App, key:u64) -> Result<Reading,String> {
    let local = app.windows.get(&key).ok_or("window is closed")?;
    let access = local.native.projection()?.access;
    let defaults:Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
        .map_err(|error| error.to_string())?;
    app.world.read_game(Mode::Vtt, |session, bytes| {
        let game = GameBindings::decode(bytes)?;
        time::validate_bindings(session, game.downtime(), game.time_holds())?;
        let roster = game.shared_roster();
        let proposals = game.downtime().values().map(|proposal| {
            let ready = roster.as_ref().map_or_else(|| Err("configure the saved table roster".into()),
                |roster| time::ready(session, proposal, roster, game.time_holds()));
            let (label, available) = match ready {
                Ok(Some(_)) => ("Ready for the DM to choose".into(),true),
                Ok(None) => ("Waiting for individual permissions".into(),false),
                Err(reason) => (reason,false),
            };
            (proposal.witness.event, proposal.declaration.ticks, label, available)
        }).collect();
        let selected = defaults.party;
        let own_members = if let Some(party) = &selected {
            isocosm::party::members(&session.sim,party)?.into_iter()
                .filter(|m| m.participant == access.participant.0).map(|m| m.member).collect()
        } else { Vec::new() };
        let map = selected.as_ref().ok_or("Choose an authored party".to_string())
            .and_then(|party| isometry_campaign::native::party::read_party(session,access,party));
        let event_lines = map.as_ref().map(|map| map.events.iter().map(|key| {
            match session.sim.state().events.get(key) {
                Some(event) => {
                    let fact = event.authored.as_ref().map(|line|format!("{}: {}",line.kind,line.text))
                        .unwrap_or_else(|| format!("{}: subject {:?}, object {:?}",event.process,event.subject,event.object));
                    format!("{key} at native tick {}: {fact}",event.tick)
                },
                None => format!("{key}: materialised event unavailable"),
            }
        }).collect()).unwrap_or_default();
        let players = if access.seat == Seat::Dm {
            session.sim.state().population.groups.iter()
                .filter(|(_,group)| isocosm::directing::is_participant(&group.entity))
                .flat_map(|(first,group)| *first..*first+group.count)
                .filter(|id| *id != access.participant.0).collect()
        } else { Vec::new() };
        Ok(Reading { seat:access.seat, participant:access.participant.0, subject:access.subject.map(|id| id.0),
            game, players, tick:session.sim.state().tick,
            parties:session.sim.genesis().rules.party_relations.keys().cloned().collect(), selected, own_members, map, proposals, event_lines })
    })
}

pub(super) fn panel(app:&App, key:u64) -> Child {
    let mut rows:Vec<Child> = vec![Box::new(el("h2",text("Table time and party")))];
    let reading = match read(app,key) { Ok(reading)=>reading, Err(reason)=>{
        rows.push(Box::new(el("p",text(reason)).attr("role","status")));
        return Box::new(el("section",rows).attr("class","panel"));
    }};
    rows.push(Box::new(el("p",text(format!("Native tick {}",reading.tick)))));
    let roster = reading.game.shared_players();
    if reading.seat == Seat::Dm {
        rows.push(Box::new(el("h3",text("Saved players"))));
        if roster.is_none() {
            rows.push(button("table-roster-empty".into(),"Configure an empty player roster".into(),key,
                move |app| app.vtt_roster(key,None)));
        }
        for player in reading.players {
            let bound = roster.is_some_and(|players| players.contains(&player));
            rows.push(button(format!("table-player-{player}"),format!("{} player {player}",if bound {"Unbind"} else {"Bind"}),key,
                move |app| app.vtt_roster(key,Some(player))));
        }
        if roster.is_some() {
            for ticks in [1,10] {
                rows.push(button(format!("table-propose-{ticks}"),format!("Propose {ticks} ticks"),key,
                    move |app| app.vtt_send(key,VttIntent::Time(TimeIntent::Downtime{ticks}))));
            }
        }
    } else if reading.seat == Seat::Player && roster.is_some_and(|players| players.contains(&reading.participant)) {
        let released = reading.game.time_holds().iter().rev().find(|hold| hold.participant==reading.participant)
            .is_some_and(|hold| hold.released);
        let intent = if released {TimeIntent::TakeHold} else {TimeIntent::ReleaseHold};
        rows.push(button("table-time-hold".into(),if released {"Take my time hold back"} else {"Release my time hold"}.into(),key,
            move |app| app.vtt_send(key,VttIntent::Time(intent))));
    }
    for (event,ticks,status,ready) in reading.proposals {
        rows.push(Box::new(el("p",text(format!("Proposal {}: {ticks} ticks. {status}",event.0)))));
        if reading.seat == Seat::Dm && ready {
            rows.push(button(format!("table-advance-{}",event.0),"Advance this proposal".into(),key,
                move |app| app.vtt_send(key,VttIntent::Time(TimeIntent::Advance{downtime:event}))));
        } else if reading.seat == Seat::Player && roster.is_some_and(|players|players.contains(&reading.participant)) {
            rows.push(button(format!("table-consent-{}",event.0),"Consent to this proposal".into(),key,
                move |app| app.vtt_send(key,VttIntent::Time(TimeIntent::Consent{downtime:event}))));
        }
    }
    rows.push(Box::new(el("h3",text("Party map"))));
    for party in reading.parties {
        let select = party.clone();
        rows.push(button(format!("table-party-{party}"),format!("Read party {party}"),key,
            move |app| app.vtt_party(key,select.clone())));
    }
    if let Some(party) = reading.selected {
        if reading.seat != Seat::CreativeObserver {
            if let Some(member) = reading.subject {
                let join = party.clone();
                rows.push(button(format!("table-join-{member}"),format!("Opt body {member} in"),key,
                    move |app| app.vtt_send(key,VttIntent::Party(PartyIntent{party:join.clone(),member:EntityHandle(member),action:PartyAction::Join}))));
                if reading.own_members.contains(&member) {
                    let disclose = party.clone();
                    rows.push(button("table-disclose".into(),"Share this body's known map".into(),key,
                        move |app| app.vtt_disclose(key,disclose.clone(),member)));
                }
            }
            for member in reading.own_members {
                let leave = party.clone();
                rows.push(button(format!("table-leave-{member}"),format!("Withdraw my body {member}"),key,
                    move |app| app.vtt_send(key,VttIntent::Party(PartyIntent{party:leave.clone(),member:EntityHandle(member),action:PartyAction::Leave}))));
            }
        }
        match reading.map {
            Ok(map) => {
                for place in map.places.values() {
                    let label = place.authored.as_ref().map(|place|place.key.as_str()).unwrap_or("Site");
                    rows.push(Box::new(el("p",text(format!("{label} ({})",place.site)))));
                }
                for route in map.routes { rows.push(Box::new(el("p",text(format!("Route {} to {}: {} ticks",route.from,route.to,route.travel))))); }
                rows.push(Box::new(el("p",text(format!("{} recorded disclosures",map.records.len())))));
                for fact in reading.event_lines { rows.push(Box::new(el("p",text(fact)))); }
            },
            Err(reason) => rows.push(Box::new(el("p",text(reason)))),
        }
    }
    rows.push(super::journey::panel(app,key));
    rows.push(super::overmap::panel(app,key));
    rows.push(super::adventure::panel(app,key));
    Box::new(el("section",rows).attr("class","panel"))
}
