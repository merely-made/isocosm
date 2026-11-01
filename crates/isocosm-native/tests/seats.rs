// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Seats as grants (case 1059): any number of GMs or none, a GM who also
//! plays, and the GM grant taken only with the yes of those it asks.

use chirograph::{IntentInvocation, IntentResult, PortableCardV1, ProjectionSnapshot, ResourceRequest};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use cosmograph::Command;
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use isocosm_native::{TABLE, VttEndpoint};

#[path = "support/table.rs"]
mod table;
use table::table;

type Card = (String, PortableCardV1, Vec<String>);

fn snapshot(endpoint: &mut VttEndpoint) -> Result<ProjectionSnapshot, String> {
    let request = endpoint.describe().projections.into_iter().find(|offer| offer.request.session.0 == TABLE)
        .unwrap().request;
    endpoint.snapshot(request)
}

fn read(endpoint: &mut VttEndpoint) -> Vec<Card> {
    let snapshot = snapshot(endpoint).unwrap();
    snapshot.scene.active_items_in_order().into_iter().map(|(instance, item)| {
        let id = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(), resource: offer.resource })
            .unwrap().bytes;
        (id, serde_json::from_slice(&bytes).unwrap(), offer.semantics.actions.iter().map(|a| a.intent.0.clone()).collect())
    }).collect()
}

fn card(cards: &[Card], id: &str) -> Card {
    cards.iter().find(|(card, _, _)| card == id).cloned().unwrap_or_else(|| panic!("no card {id} in {:?}",
        cards.iter().map(|(id, _, _)| id).collect::<Vec<_>>()))
}

fn field(card: &Card, label: &str) -> String {
    card.1.values.iter().find(|v| v.label == label).map(|v| v.value.clone()).unwrap()
}

/// Invoke `intent` on card `id`, which must offer it.
fn act(endpoint: &mut VttEndpoint, id: &str, intent: &str) {
    let snapshot = snapshot(endpoint).unwrap();
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id)
        .unwrap_or_else(|| panic!("no card {id}")).0;
    let result = endpoint.invoke(IntentInvocation { session: snapshot.session.clone(), target,
        observed_epoch: snapshot.scene.epoch, observed_revision: snapshot.scene.revision,
        intent: intent.into(), payload: Vec::new() }).unwrap();
    assert_eq!(result, IntentResult::Accepted, "{intent} on {id}");
}

fn offers(endpoint: &mut VttEndpoint, id: &str) -> Vec<String> { card(&read(endpoint), id).2 }

fn join(world: &cosmograph_host::world::World) -> ParticipantHandle {
    let joined = world.command(Command::Join).unwrap();
    ParticipantHandle(joined.strip_prefix("participant:").unwrap().parse().unwrap())
}

fn gms(endpoint: &mut VttEndpoint) -> String { field(&card(&read(endpoint), "seats"), "GM grants") }

#[test]
fn a_second_gm_is_seated_by_the_first_gms_yes_and_both_may_release_to_none() {
    let t = table();
    let mut first = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    assert_eq!(gms(&mut first), "not kept yet");
    assert!(offers(&mut player, "seats").is_empty(), "only the host's GM window starts keeping grants");
    act(&mut first, "seats", "vtt.seats-configure");
    assert_eq!(gms(&mut first), t.dm.0.to_string());

    act(&mut player, "seats", "vtt.seat-claim");
    let claim = format!("claim:{}", t.player.0);
    assert_eq!(field(&card(&read(&mut first), &claim), "Needs"), t.dm.0.to_string());
    assert!(offers(&mut player, &claim).is_empty(), "a claimant gives no yes to itself");
    act(&mut first, &claim, "vtt.seat-consent");
    assert_eq!(gms(&mut first), format!("{}, {}", t.dm.0, t.player.0));

    // The new GM opens a GM window beside its player window and sees the truth there.
    let mut second = VttEndpoint::open(&t.world, t.player, Seat::Dm).unwrap();
    assert_eq!(offers(&mut second, "seats"), ["vtt.seat-release"]);
    act(&mut first, "seats", "vtt.seat-release");
    assert!(snapshot(&mut first).is_err(), "a released grant closes its GM window");
    act(&mut second, "seats", "vtt.seat-release");
    assert_eq!(gms(&mut player), "none");
}

#[test]
fn at_a_table_with_no_gm_every_bound_player_says_yes_and_a_roster_change_stales_the_claim() {
    // Mark (case 1030): "eight to all be players relying on a prewritten
    // campaign, with one choosing to become a gm when needed".
    let t = table();
    let mut host = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    act(&mut host, "seats", "vtt.seats-configure");
    act(&mut host, "roster", "vtt.roster-empty");
    let mut everyone = vec![t.dm, t.player];
    everyone.extend((0..6).map(|_| join(&t.world)));
    for participant in &everyone { act(&mut host, &format!("player:{}", participant.0), "vtt.roster-toggle"); }
    act(&mut host, "seats", "vtt.seat-release");
    let mut players: Vec<VttEndpoint> = everyone.iter()
        .map(|p| VttEndpoint::open(&t.world, *p, Seat::Player).unwrap()).collect();
    assert_eq!(gms(&mut players[0]), "none");
    assert_eq!(field(&card(&read(&mut players[0]), "seats"), "Bound players").split(", ").count(), 8);

    let claimant = everyone[3];
    act(&mut players[3], "seats", "vtt.seat-claim");
    let claim = format!("claim:{}", claimant.0);
    assert_eq!(field(&card(&read(&mut players[0]), &claim), "Needs").split(", ").count(), 7);
    for index in [0, 1, 2, 4, 5] { act(&mut players[index], &claim, "vtt.seat-consent"); }
    assert_eq!(gms(&mut players[0]), "none", "six of seven yeses grant nothing");
    act(&mut players[6], &claim, "vtt.seat-consent");
    assert_eq!(gms(&mut players[0]), "none");
    act(&mut players[7], &claim, "vtt.seat-consent");
    assert_eq!(gms(&mut players[0]), claimant.0.to_string());

    // The new GM unbinds a player; a later claim made before it is stale.
    let mut gm = VttEndpoint::open(&t.world, claimant, Seat::Dm).unwrap();
    act(&mut players[5], "seats", "vtt.seat-claim");
    let late = format!("claim:{}", everyone[5].0);
    act(&mut gm, &format!("player:{}", everyone[7].0), "vtt.roster-toggle");
    assert_eq!(field(&card(&read(&mut gm), &late), "Status"), "stale: claim again");
    assert!(offers(&mut gm, &late).is_empty());
}

#[test]
fn six_gms_are_seated_one_claim_at_a_time_each_asking_every_gm_seated() {
    // Mark (case 1030): "it should even be possible for six people to all be gm".
    let t = table();
    let mut first = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    act(&mut first, "seats", "vtt.seats-configure");
    let mut seated = vec![(t.dm, first)];
    for _ in 0..5 {
        let newcomer = join(&t.world);
        let mut window = VttEndpoint::open(&t.world, newcomer, Seat::Player).unwrap();
        act(&mut window, "seats", "vtt.seat-claim");
        let claim = format!("claim:{}", newcomer.0);
        assert_eq!(field(&card(&read(&mut window), &claim), "Needs").split(", ").count(), seated.len());
        for (_, gm) in &mut seated { act(gm, &claim, "vtt.seat-consent"); }
        seated.push((newcomer, VttEndpoint::open(&t.world, newcomer, Seat::Dm).unwrap()));
    }
    assert_eq!(gms(&mut seated[0].1).split(", ").count(), 6);
}

#[test]
fn a_gm_who_plays_proposes_downtime_from_one_window_and_consents_from_the_other() {
    let t = table();
    let mut gm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    act(&mut gm, "seats", "vtt.seats-configure");
    act(&mut gm, "roster", "vtt.roster-empty");
    act(&mut gm, &format!("player:{}", t.dm.0), "vtt.roster-toggle");
    act(&mut gm, &format!("player:{}", t.player.0), "vtt.roster-toggle");
    act(&mut gm, "time", "vtt.propose-1");
    let mut own = VttEndpoint::open(&t.world, t.dm, Seat::Player).unwrap();
    let mut other = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    let proposal = read(&mut gm).into_iter().find(|(id, _, _)| id.starts_with("proposal:")).unwrap().0;
    act(&mut own, &proposal, "vtt.consent");
    assert!(offers(&mut gm, &proposal).is_empty(), "the other bound player has not said yes");
    act(&mut other, &proposal, "vtt.consent");
    let before = t.world.read(|session| session.sim.state().tick).unwrap();
    act(&mut gm, &proposal, "vtt.advance");
    assert_eq!(t.world.read(|session| session.sim.state().tick).unwrap(), before + 1);
}

#[test]
fn body_control_stays_with_who_plays_it_whatever_the_seat() {
    let t = table();
    t.world.read(|session| {
        let gm = OrdinaryAccess { participant: t.dm, seat: Seat::Dm,
            subject: session.sim.plays(t.dm.0).map(EntityHandle) };
        let player = OrdinaryAccess { participant: t.player, seat: Seat::Player,
            subject: session.sim.plays(t.player.0).map(EntityHandle) };
        let (own, theirs) = (t.bodies[0], t.bodies[1]);
        assert!(isocosm_native::native::seat_controls(session, gm, own).is_ok(), "a GM acts for its own body");
        assert!(isocosm_native::native::seat_controls(session, gm, theirs).is_err(), "not for a played one");
        assert!(isocosm_native::native::seat_controls(session, player, theirs).is_ok());
        assert!(isocosm_native::native::seat_controls(session, player, own).is_err());
        assert_eq!(isocosm_native::ruleset::controls(session, gm, theirs), false);
    }).unwrap();
}
