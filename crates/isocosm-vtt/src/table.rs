// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Table time and party: the saved roster, downtime proposals and holds,
//! and the chosen party's members and shared map, at this window's seat.

use chirograph::IntentEffect;
use isocosm_host::endpoint::disclosure::{Item, NULL, action, card, value};
use isocosm_overlay::{EntityHandle, Seat, vtt::{PartyAction, PartyIntent, TimeIntent, VttIntent}};
use isometry_campaign::native::{GameBindings, SharedRosterInput, WindowReading, time};

use crate::{Act, Built, Cx, VttEndpoint};

const DOMAIN: IntentEffect = IntentEffect::DomainTruth;

/// The roster input that binds or unbinds `player`, or configures an empty roster.
pub(crate) fn roster(endpoint: &VttEndpoint, player: Option<u64>) -> Result<Vec<u8>, String> {
    endpoint.read(|_, bytes| {
        let game = GameBindings::decode(bytes)?;
        let mut players = game.shared_players().cloned().unwrap_or_default();
        if let Some(player) = player && !players.remove(&player) { players.insert(player); }
        let input = SharedRosterInput { version: 1, expected_roster_revision: game.roster_revision(), players };
        serde_json::to_vec(&input).map_err(|e| e.to_string())
    })
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let access = cx.cut.access;
    let (seat, participant, subject) = (access.seat, access.participant.0, access.subject.map(|id| id.0));
    let party = cx.party.map(str::to_owned);
    let mut built = Built::new();
    endpoint.read(|session, bytes| {
        let game = GameBindings::decode(bytes)?;
        time::validate_bindings(session, game.downtime(), game.time_holds())?;
        let roster = game.shared_roster();
        let players = game.shared_players();
        let rostered = players.is_some_and(|p| p.contains(&participant));
        let mut time_actions = Vec::new();
        let mut time_acts = Vec::new();
        if seat == Seat::Dm && players.is_some() {
            for ticks in [1u64, 10] {
                time_actions.push(action(&format!("vtt.propose-{ticks}"), &format!("Propose {ticks} ticks"),
                    "Propose downtime for the table's consent.", NULL, DOMAIN));
                time_acts.push(Act::Send(VttIntent::Time(TimeIntent::Downtime { ticks })));
            }
        } else if seat == Seat::Player && rostered {
            let released = game.time_holds().iter().rev().find(|hold| hold.participant == participant)
                .is_some_and(|hold| hold.released);
            let (label, intent) = if released { ("Take my time hold back", TimeIntent::TakeHold) }
                else { ("Release my time hold", TimeIntent::ReleaseHold) };
            time_actions.push(action("vtt.hold", label, "Hold or release native time at this table.", NULL, DOMAIN));
            time_acts.push(Act::Send(VttIntent::Time(intent)));
        }
        let reading = WindowReading::new(session, &cx.cut.visible);
        let mut values: Vec<_> = reading.characters().into_iter().map(|(key, id)| value("Character", format!("{key} ({id})")))
            .collect();
        values.extend(reading.sites().into_iter().map(|(key, id)| value("Site", format!("{key} ({id})"))));
        built.push(Item { id: "seat".into(), card: card("Seat cut", values, &["seat"]), actions: Vec::new() }, Vec::new());
        built.push(Item { id: "time".into(), card: card("Table time", vec![value("Native tick", session.sim.state().tick)],
            &["time"]), actions: time_actions }, time_acts);
        if seat == Seat::Dm {
            if players.is_none() {
                built.push(Item { id: "roster".into(), card: card("Saved players", vec![value("Roster", "not configured")], &["roster"]),
                    actions: vec![action("vtt.roster-empty", "Configure an empty player roster",
                        "Start the saved table roster.", NULL, DOMAIN)] }, vec![Act::Roster(None)]);
            }
            let others = session.sim.state().population.groups.iter()
                .filter(|(_, group)| isocosm::directing::is_participant(&group.entity))
                .flat_map(|(first, group)| *first..*first + group.count).filter(|id| *id != participant);
            for player in others {
                let bound = players.is_some_and(|p| p.contains(&player));
                let label = format!("{} player {player}", if bound { "Unbind" } else { "Bind" });
                built.push(Item { id: format!("player:{player}"), card: card(format!("Player {player}"),
                    vec![value("Saved", bound)], &["player"]),
                    actions: vec![action("vtt.roster-toggle", &label, "Bind or unbind this player in the saved roster.",
                        NULL, DOMAIN)] }, vec![Act::Roster(Some(player))]);
            }
        }
        for proposal in game.downtime().values() {
            let event = proposal.witness.event;
            let ready = roster.as_ref().map_or_else(|| Err("configure the saved table roster".into()),
                |roster| time::ready(session, proposal, roster, game.time_holds()));
            let (status, available) = match ready {
                Ok(Some(_)) => ("Ready for the DM to choose".to_string(), true),
                Ok(None) => ("Waiting for individual permissions".to_string(), false),
                Err(reason) => (reason, false),
            };
            let values = vec![value("Ticks", proposal.declaration.ticks), value("Status", status)];
            let (actions, acts) = if seat == Seat::Dm && available {
                (vec![action("vtt.advance", "Advance this proposal", "Advance native time by this proposal.", NULL, DOMAIN)],
                    vec![Act::Send(VttIntent::Time(TimeIntent::Advance { downtime: event }))])
            } else if seat == Seat::Player && rostered {
                (vec![action("vtt.consent", "Consent to this proposal", "Consent to this downtime.", NULL, DOMAIN)],
                    vec![Act::Send(VttIntent::Time(TimeIntent::Consent { downtime: event }))])
            } else { (Vec::new(), Vec::new()) };
            built.push(Item { id: format!("proposal:{}", event.0), card: card(format!("Proposal {}", event.0), values,
                &["proposal"]), actions }, acts);
        }
        for name in session.sim.genesis().rules.party_relations.keys() {
            let selected = party.as_deref() == Some(name.as_str());
            built.push(Item { id: format!("party:{name}"), card: card(name,
                vec![value("Selected", selected)], &["party"]),
                actions: vec![action("vtt.read-party", "Read this party", "Choose this party for this window.", NULL,
                    IntentEffect::Curation)] }, vec![Act::SelectParty(name.clone())]);
        }
        let Some(party) = party else { return Ok(()) };
        if seat != Seat::CreativeObserver {
            let own: Vec<u64> = isocosm::party::members(&session.sim, &party)?.into_iter()
                .filter(|m| m.participant == participant).map(|m| m.member).collect();
            if let Some(member) = subject {
                let intent = |action| VttIntent::Party(PartyIntent { party: party.clone(), member: EntityHandle(member), action });
                let mut actions = vec![action("vtt.join", &format!("Opt body {member} in"), "Opt this body into the party.", NULL, DOMAIN)];
                let mut acts = vec![Act::Send(intent(PartyAction::Join))];
                if own.contains(&member) {
                    actions.push(action("vtt.disclose", "Share this body's known map", "Disclose what this body knows.", NULL, DOMAIN));
                    acts.push(Act::Disclose { party: party.clone(), member });
                }
                built.push(Item { id: format!("body:{member}"), card: card(format!("Body {member}"), Vec::new(), &["played"]),
                    actions }, acts);
            }
            for member in own {
                built.push(Item { id: format!("member:{member}"), card: card(format!("My body {member}"), Vec::new(), &["member"]),
                    actions: vec![action("vtt.leave", &format!("Withdraw my body {member}"), "Withdraw this body from the party.",
                        NULL, DOMAIN)] },
                    vec![Act::Send(VttIntent::Party(PartyIntent { party: party.clone(), member: EntityHandle(member),
                        action: PartyAction::Leave }))]);
            }
        }
        match isometry_campaign::native::party::read_party(session, access, &party) {
            Err(reason) => built.push(Item { id: "map".into(), card: card("Party map", vec![value("Refused", reason)], &["map"]),
                actions: Vec::new() }, Vec::new()),
            Ok(map) => {
                for place in map.places.values() {
                    let label = place.authored.as_ref().map(|p| p.key.as_str()).unwrap_or("Site");
                    built.push(Item { id: format!("place:{}", place.site), card: card(format!("{label} ({})", place.site),
                        Vec::new(), &["place"]), actions: Vec::new() }, Vec::new());
                }
                for route in &map.routes {
                    built.push(Item { id: format!("route:{}-{}", route.from, route.to), card: card(
                        format!("Route {} to {}", route.from, route.to), vec![value("Ticks", route.travel)], &["route"]),
                        actions: Vec::new() }, Vec::new());
                }
                let mut values = vec![value("Recorded disclosures", map.records.len())];
                values.extend(map.events.iter().map(|key| match session.sim.state().events.get(key) {
                    Some(event) => value(key.as_str(), format!("tick {}: {}", event.tick, event.authored.as_ref()
                        .map(|line| format!("{}: {}", line.kind, line.text))
                        .unwrap_or_else(|| format!("{}: subject {:?}, object {:?}", event.process, event.subject, event.object)))),
                    None => value(key.as_str(), "materialised event unavailable"),
                }));
                built.push(Item { id: "map".into(), card: card("Party map", values, &["map"]), actions: Vec::new() }, Vec::new());
            },
        }
        Ok(())
    })?;
    Ok(built)
}
