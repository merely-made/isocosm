// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The VTT endpoint's six projections over a table world: one authored
//! party, a DM and a player, each playing one embodied subject.

use chirograph::{IntentInvocation, IntentResult, PortableCardV1, ProjectionSnapshot, ResourceRequest};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm::{Command, Execution, Founding, Session, directing::Played, schema::{Id, Method}};
use isocosm_host::world::World;
use isocosm_overlay::{ParticipantHandle, Seat};
use isocosm_vtt::{ADVENTURE, AUTHORING, JOURNEYS, KNOWLEDGE, OVERMAP, TABLE, VttEndpoint};

const BAND: &str = "party:band";
const MEMBER: &str = "relation:member-of-band";

struct Table { world: World, dm: ParticipantHandle, player: ParticipantHandle, bodies: [Id; 2] }

fn table() -> Table {
    let mut genesis = Founding { seed: 11, sites: 3, population: 24, cohort_size: 1, lineages: 3, ecology: true,
        bodies: Some(Default::default()), played: Some(Played { lineage: 1, region_sites: 1 }), ..Default::default() }
        .generate().unwrap();
    genesis.rules.relations.insert(MEMBER.into());
    genesis.rules.party_relations.insert(BAND.into(), MEMBER.into());
    genesis.validate().unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let bodies: Vec<Id> = session.sim.state().population.groups.iter()
        .filter(|(_, group)| group.entity.method == Method::Deliberative).map(|(first, _)| *first).take(2).collect();
    let mut participants = Vec::new();
    for body in &bodies {
        let joined = session.command(Command::Join).unwrap();
        let participant: Id = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        session.command(Command::Take { participant, critter: *body }).unwrap();
        participants.push(ParticipantHandle(participant));
    }
    Table { world: World::new(session), dm: participants[0], player: participants[1], bodies: [bodies[0], bodies[1]] }
}

type Card = (String, PortableCardV1, Vec<String>);

fn snapshot(endpoint: &mut VttEndpoint, session: &str) -> ProjectionSnapshot {
    let request = endpoint.describe().projections.into_iter().find(|offer| offer.request.session.0 == session)
        .unwrap().request;
    endpoint.snapshot(request).unwrap()
}

fn cards(endpoint: &mut VttEndpoint, snapshot: &ProjectionSnapshot) -> Vec<Card> {
    snapshot.scene.active_items_in_order().into_iter().map(|(instance, item)| {
        let id = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(), resource: offer.resource })
            .unwrap().bytes;
        (id, serde_json::from_slice(&bytes).unwrap(), offer.semantics.actions.iter().map(|a| a.intent.0.clone()).collect())
    }).collect()
}

fn card<'a>(cards: &'a [Card], id: &str) -> &'a Card {
    cards.iter().find(|(card, _, _)| card == id).unwrap_or_else(|| panic!("no card {id} in {:?}",
        cards.iter().map(|(id, _, _)| id).collect::<Vec<_>>()))
}

fn field(card: &Card, label: &str) -> Option<String> {
    card.1.values.iter().find(|v| v.label == label).map(|v| v.value.clone())
}

fn invoke(snapshot: &ProjectionSnapshot, id: &str, intent: &str, payload: &[u8]) -> IntentInvocation {
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id).unwrap().0;
    IntentInvocation { session: snapshot.session.clone(), target, observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision, intent: intent.into(), payload: payload.to_vec() }
}

/// Snapshot `session`, invoke `intent` on card `id` and return the result.
fn act(endpoint: &mut VttEndpoint, session: &str, id: &str, intent: &str, payload: &[u8]) -> IntentResult {
    let snapshot = snapshot(endpoint, session);
    endpoint.invoke(invoke(&snapshot, id, intent, payload)).unwrap()
}

fn read(endpoint: &mut VttEndpoint, session: &str) -> Vec<Card> {
    let snapshot = snapshot(endpoint, session);
    cards(endpoint, &snapshot)
}

#[test]
fn six_projections_answer_at_both_seats_and_dm_panels_refuse_a_player() {
    let t = table();
    let mut dm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    assert_eq!(dm.describe().projections.len(), 6);
    for offer in dm.describe().projections {
        let session = offer.request.session.0;
        assert!(!read(&mut dm, &session).is_empty() && !read(&mut player, &session).is_empty(), "{session} reads");
    }
    for session in [ADVENTURE, AUTHORING] {
        let cards = read(&mut player, session);
        assert!(cards.len() == 1 && cards[0].2.is_empty() && field(&cards[0], "Note").is_some(),
            "{session} explains itself to a player and offers nothing");
    }
    let authoring = read(&mut dm, AUTHORING);
    assert_eq!(card(&authoring, "authoring").2.len(), 8, "six previews, apply and assert");
    assert_eq!(field(card(&read(&mut dm, KNOWLEDGE), "knowledge"), "Note").unwrap(),
        "The player's body learns and discloses these records.");
    let seat = read(&mut dm, TABLE);
    assert!(card(&seat, "seat").2.is_empty());
}

#[test]
fn the_dm_keeps_the_roster_and_proposes_time_and_a_rostered_player_consents() {
    let t = table();
    let mut dm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    assert!(card(&read(&mut player, TABLE), "time").2.is_empty(), "an unrostered player holds no time");
    assert_eq!(act(&mut dm, TABLE, "roster", "vtt.roster-empty", b""), IntentResult::Accepted);
    let cards = read(&mut dm, TABLE);
    assert!(!cards.iter().any(|(id, _, _)| id == "roster"), "a configured roster needs no configuring");
    let seat = format!("player:{}", t.player.0);
    assert_eq!(field(card(&cards, &seat), "Saved").unwrap(), "false");
    assert_eq!(act(&mut dm, TABLE, &seat, "vtt.roster-toggle", b""), IntentResult::Accepted);
    assert_eq!(field(card(&read(&mut dm, TABLE), &seat), "Saved").unwrap(), "true");
    assert_eq!(act(&mut dm, TABLE, "time", "vtt.propose-1", b""), IntentResult::Accepted);
    let cards = read(&mut player, TABLE);
    assert_eq!(card(&cards, "time").2, ["vtt.hold"]);
    let (proposal, _, intents) = cards.iter().find(|(id, _, _)| id.starts_with("proposal:"))
        .expect("the DM's proposal is recorded");
    assert_eq!(intents, &["vtt.consent"]);
    let proposal = proposal.clone();
    assert_eq!(act(&mut player, TABLE, &proposal, "vtt.consent", b""), IntentResult::Accepted);
}

#[test]
fn choosing_a_party_curates_every_projection_and_stales_their_snapshots() {
    let t = table();
    let mut dm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let journeys = snapshot(&mut dm, JOURNEYS);
    let cards = cards(&mut dm, &journeys);
    assert_eq!(field(card(&cards, "journeys"), "Note").unwrap(), "Choose an authored party to read journey proposals");
    let party = format!("party:{BAND}");
    assert_eq!(act(&mut dm, TABLE, &party, "vtt.read-party", b""), IntentResult::Accepted);
    assert_eq!(dm.party(), Some(BAND));
    assert!(matches!(dm.invoke(invoke(&journeys, "journeys", "vtt.journey-apply", b"")).unwrap(),
        IntentResult::Stale { .. }), "a curation on one projection stales the others");
    assert_eq!(field(card(&read(&mut dm, TABLE), &party), "Selected").unwrap(), "true");
    assert_ne!(field(card(&read(&mut dm, JOURNEYS), "journeys"), "Note").unwrap(),
        "Choose an authored party to read journey proposals");
    let overmap = read(&mut dm, OVERMAP);
    assert!(field(card(&overmap, "overmap"), "Note").is_none_or(|note| note != "Choose an authored party"),
        "the overmap reads the chosen party");
}

#[test]
fn a_player_opts_the_played_body_into_the_party_and_withdraws_it() {
    let t = table();
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    assert_eq!(act(&mut player, TABLE, &format!("party:{BAND}"), "vtt.read-party", b""), IntentResult::Accepted);
    let body = format!("body:{}", t.bodies[1]);
    assert_eq!(card(&read(&mut player, TABLE), &body).2, ["vtt.join"]);
    assert_eq!(act(&mut player, TABLE, &body, "vtt.join", b""), IntentResult::Accepted);
    let cards = read(&mut player, TABLE);
    assert_eq!(card(&cards, &body).2, ["vtt.join", "vtt.disclose"], "a member body can share its known map");
    let member = format!("member:{}", t.bodies[1]);
    assert_eq!(card(&cards, &member).2, ["vtt.leave"]);
    assert_eq!(act(&mut player, TABLE, &member, "vtt.leave", b""), IntentResult::Accepted);
    assert!(!read(&mut player, TABLE).iter().any(|(id, _, _)| *id == member));
    assert!(!read(&mut player, KNOWLEDGE).is_empty());
}

#[test]
fn authoring_refuses_malformed_operations_and_previews_nothing() {
    let t = table();
    let mut dm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let before = t.world.cut().unwrap();
    for intent in ["vtt.preview-0", "vtt.preview-5", "vtt.apply", "vtt.assert"] {
        assert!(matches!(act(&mut dm, AUTHORING, "authoring", intent, b"{}"), IntentResult::Rejected { .. }),
            "{intent} refuses an empty object");
    }
    assert!(!read(&mut dm, AUTHORING).iter().any(|(id, _, _)| id == "quote"), "a refused preview quotes nothing");
    assert_eq!(t.world.cut().unwrap(), before, "refusals leave the world alone");
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    let authoring = snapshot(&mut player, AUTHORING);
    assert!(matches!(player.invoke(invoke(&authoring, "authoring", "vtt.apply", b"{}")).unwrap(),
        IntentResult::Rejected { .. }), "a player is offered no authoring");
}

#[test]
fn the_endpoint_survives_the_wire_encoding() {
    use chirograph::{Carrier, CarrierRequestBody, CarrierResponseBody};
    use graphshell_endpoint::local::LocalCarrier;
    let t = table();
    let endpoint = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let mut carrier = LocalCarrier::new(endpoint, |_, _| Err("resume is not used".to_string()));
    let CarrierResponseBody::Descriptor(descriptor) = carrier.request(CarrierRequestBody::Discover).unwrap()
        else { panic!("discovery answers a descriptor") };
    assert_eq!(descriptor.projections.len(), 6);
    let request = descriptor.projections.into_iter().find(|offer| offer.request.session.0 == TABLE).unwrap().request;
    let CarrierResponseBody::Snapshot(first) = carrier.request(CarrierRequestBody::Snapshot(request)).unwrap()
        else { panic!("a snapshot request answers a snapshot") };
    let CarrierResponseBody::Intent(result) = carrier.request(CarrierRequestBody::Intent(
        invoke(&first, "roster", "vtt.roster-empty", b""))).unwrap() else { panic!("an intent answers a result") };
    assert_eq!(result, IntentResult::Accepted);
}
