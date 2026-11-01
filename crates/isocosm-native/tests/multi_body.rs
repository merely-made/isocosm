// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Several bodies per participant at the table (cases 1062, 1065): a player
//! plays two and acts for each, a window views as one chosen body, a GM plays
//! several and manages the unplayed, and downtime counts the participant once.

use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph::{Command, schema::Id};
use cosmograph_overlay::{EntityHandle, Intent, OrdinaryAccess, ParticipantHandle, Seat, Tick, WorldPoint,
    vtt::{Cell, TableAct, TableActKind, TableBatch, VttIntent, VttIntentEnvelope}};
use cosmograph_host::world::World;
use isocosm_native::{TABLE, VttEndpoint, native::{GameBindings, seat_controls, submit_game_access}};

#[path = "support/table.rs"]
mod table;
#[path = "support/admitted.rs"]
mod admitted;

fn snapshot(endpoint: &mut VttEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections.into_iter().find(|o| o.request.session.0 == TABLE).unwrap().request;
    endpoint.snapshot(request).unwrap()
}

fn ids(endpoint: &mut VttEndpoint) -> Vec<String> {
    let snap = snapshot(endpoint);
    snap.scene.active_items_in_order().into_iter()
        .map(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone()).collect()
}

fn act(endpoint: &mut VttEndpoint, id: &str, intent: &str) {
    let snap = snapshot(endpoint);
    let target = snap.scene.active_items_in_order().into_iter().find(|(_, item)|
        snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id)
        .unwrap_or_else(|| panic!("no card {id}")).0;
    let result = endpoint.invoke(IntentInvocation { session: snap.session.clone(), target,
        observed_epoch: snap.scene.epoch, observed_revision: snap.scene.revision, intent: intent.into(),
        payload: Vec::new() }).unwrap();
    assert_eq!(result, IntentResult::Accepted, "{intent} on {id}");
}

fn access(world: &World, participant: ParticipantHandle, seat: Seat) -> OrdinaryAccess {
    world.read(|s| OrdinaryAccess { participant, seat, subject: s.sim.plays(participant.0).map(EntityHandle) }).unwrap()
}

/// Move `actor` to the cell it stands on, through the table's native batch.
fn stay(world: &World, who: OrdinaryAccess, actor: Id) -> Result<String, String> {
    let (mut session, mut sections) = world.save().map(|save| (cosmograph::Session::load(save.native,
        cosmograph::Execution::Grouped).unwrap(), save.sections)).unwrap();
    let game = sections["vtt"].clone();
    let bindings = GameBindings::decode(&game).unwrap();
    let placement = &bindings.positions().values().find(|p| p.entity == actor).unwrap().placement;
    let at = placement.at.map(|v| i32::try_from(v).unwrap());
    let envelope = VttIntentEnvelope { participant: who.participant, tick: Tick(session.sim.state().tick),
        intent: Intent::Game(VttIntent::Table(TableBatch { acts: vec![TableAct { actor: EntityHandle(actor),
            kind: TableActKind::Move { to: Cell { site: cosmograph_overlay::PlaceHandle(placement.site), at: WorldPoint(at) } } }] })) };
    let (note, next) = submit_game_access(&mut session, game, &serde_json::to_vec(&envelope).unwrap(), who)?;
    sections.insert("vtt".into(), next);
    Ok(note)
}

#[test]
fn one_player_plays_two_bodies_acts_for_each_and_views_as_one_per_window() {
    let t = admitted::admitted_table_controlled([true, true]);
    let [arden, brisa] = t.characters;
    let played = t.world.read(|s| s.sim.played(t.player.0)).unwrap();
    assert!(played.contains(&arden) && played.contains(&brisa), "the player plays both admitted characters");
    let player = access(&t.world, t.player, Seat::Player);
    for (body, key) in [(arden, admitted::KEYS[0]), (brisa, admitted::KEYS[1])] {
        t.world.read(|s| seat_controls(s, player, body)).unwrap().unwrap();
        let note = stay(&t.world, player, body).unwrap();
        assert!(note.starts_with(&format!("moved: {key} at")), "the player moves body {body}: {note}");
    }
    let dm = access(&t.world, t.dm, Seat::Dm);
    assert!(stay(&t.world, dm, arden).is_err(), "a GM window does not move a body another participant plays");

    // A window views as one body, chosen per window; two windows, two views.
    let mut first = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    let mut second = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    let cards = ids(&mut first);
    for body in &played { assert!(cards.contains(&format!("body:{body}")), "each played body has a card: {cards:?}"); }
    act(&mut second, &format!("body:{brisa}"), "vtt.view-as");
    let snap = snapshot(&mut second);
    let viewing = |snap: &ProjectionSnapshot, body: Id| snap.scene.active_items_in_order().into_iter()
        .any(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == format!("body:{body}"));
    assert!(viewing(&snap, brisa));
    // The chosen window offers to view as the others, not as brisa itself.
    let offers = |endpoint: &mut VttEndpoint, body: Id| {
        let snap = snapshot(endpoint);
        let (instance, _) = snap.scene.active_items_in_order().into_iter().find(|(_, item)|
            snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == format!("body:{body}")).unwrap();
        snap.presentation.offers_for(instance).unwrap().iter()
            .any(|o| o.semantics.actions.iter().any(|a| a.intent.0 == "vtt.view-as"))
    };
    assert!(!offers(&mut second, brisa) && offers(&mut second, arden));
    assert!(offers(&mut first, brisa), "the first window still views as its default body");

    // Putting one of the two down (Leave): the window that viewed it views as
    // its default again, the other body is still played and still moves.
    act(&mut second, &format!("body:{brisa}"), "vtt.leave-body");
    let played = t.world.read(|s| s.sim.played(t.player.0)).unwrap();
    assert!(!played.contains(&brisa) && played.contains(&arden));
    assert!(!ids(&mut second).contains(&format!("body:{brisa}")), "the put-down body has no card");
    let player = access(&t.world, t.player, Seat::Player);
    assert!(stay(&t.world, player, brisa).is_err(), "a put-down body is not acted for");
    assert!(stay(&t.world, player, arden).is_ok());
    t.world.read(|s| assert!(s.entries.iter().any(|e| matches!(&e.command,
        Command::Leave { participant, critter } if *participant == t.player.0 && *critter == brisa)))).unwrap();
}

#[test]
fn a_gm_plays_three_bodies_and_still_manages_the_unplayed() {
    let t = table::table();
    let dm = t.dm.0;
    let unplayed: Vec<Id> = t.world.read(|s| s.sim.state().population.groups.iter()
        .filter(|(id, g)| g.entity.method == cosmograph::schema::Method::Deliberative && g.entity.alive
            && !s.sim.state().relations.iter().any(|r| r.kind == cosmograph::directing::PLAYS && r.object == **id))
        .map(|(id, _)| *id).collect()).unwrap();
    assert!(unplayed.len() >= 3, "the fixture has unplayed bodies to take and to manage");
    for body in &unplayed[..2] {
        t.world.command(Command::TakeAlso { participant: dm, critter: *body }).unwrap();
    }
    let played = t.world.read(|s| s.sim.played(dm)).unwrap();
    assert_eq!(played.len(), 3, "the GM plays its own body and two more");
    let gm = access(&t.world, t.dm, Seat::Dm);
    t.world.read(|s| {
        for body in &played { assert!(seat_controls(s, gm, *body).is_ok(), "the GM acts for its body {body}"); }
        assert!(seat_controls(s, gm, unplayed[2]).is_ok(), "and manages an unplayed one");
        assert!(seat_controls(s, gm, t.bodies[1]).is_err(), "but not the player's");
        assert!(isocosm_native::ruleset::controls(s, gm, unplayed[2]));
    }).unwrap();
}

#[test]
fn downtime_counts_a_two_body_player_once() {
    let t = table::table();
    let spare: Id = t.world.read(|s| s.sim.state().population.groups.iter()
        .find(|(id, g)| g.entity.method == cosmograph::schema::Method::Deliberative && g.entity.alive
            && !s.sim.state().relations.iter().any(|r| r.kind == cosmograph::directing::PLAYS && r.object == **id))
        .map(|(id, _)| *id).unwrap()).unwrap();
    t.world.command(Command::TakeAlso { participant: t.player.0, critter: spare }).unwrap();
    let mut gm = VttEndpoint::open(&t.world, t.dm, Seat::Dm).unwrap();
    act(&mut gm, "roster", "vtt.roster-empty");
    act(&mut gm, &format!("player:{}", t.player.0), "vtt.roster-toggle");
    act(&mut gm, "time", "vtt.propose-1");
    let proposal = ids(&mut gm).into_iter().find(|id| id.starts_with("proposal:")).unwrap();
    let mut player = VttEndpoint::open(&t.world, t.player, Seat::Player).unwrap();
    act(&mut player, &proposal, "vtt.consent");
    let before = t.world.read(|s| s.sim.state().tick).unwrap();
    act(&mut gm, &proposal, "vtt.advance");
    assert_eq!(t.world.read(|s| s.sim.state().tick).unwrap(), before + 1,
        "one yes from the participant readies the proposal, however many bodies it plays");
}

/// Telling at the table (case 1066): a split party regroups, and one
/// player's body tells another's everything about a place it heard of; the
/// hearer knows it afterwards or the draw refused, and the world replays alike.
#[test]
fn a_regrouped_body_tells_another_about_a_place_at_the_table() {
    use cosmograph::directing::readings::{Mode, view};
    let mut genesis = cosmograph::Founding { seed: 5, sites: 3, population: 48, cohort_size: 1, lineages: 3,
        ecology: true, bodies: Some(Default::default()),
        played: Some(cosmograph::directing::Played { lineage: 1, region_sites: 1 }), ..Default::default() }
        .generate().unwrap();
    for kind in cosmograph::knowing::NOTE_KINDS { genesis.rules.note_kinds.insert(kind.into()); }
    let mut session = cosmograph::Session::new(genesis, cosmograph::Execution::Individuals).unwrap();
    // Two living bodies at one site, one of which heard of an event at some
    // place that the other missed: the regrouped split party.
    let differing = |s: &cosmograph::Session| {
        let st = s.sim.state();
        let living: Vec<(Id, Id)> = st.population.groups.iter().filter(|(_, g)| g.count == 1 && g.entity.alive
            && g.entity.method == cosmograph::schema::Method::Deliberative)
            .filter_map(|(id, g)| g.entity.resident_site().map(|site| (*id, site))).collect();
        for &(a, site) in &living { for &(b, other) in &living {
            if a == b || site != other { continue; }
            // An event worth telling: unknown to the hearer, near what it knows.
            for (key, _) in s.sim.tellable_events(a, b, s.sim.genesis().rules.telling().event_hops) {
                if let Some(site) = st.events[&key].place.site() { return Some((a, b, key, site)); }
            }
        } }
        None
    };
    while differing(&session).is_none() {
        assert!(session.sim.state().tick < 80, "no two bodies at a site know the world differently");
        session.advance(1).unwrap();
    }
    let (teller, hearer, event, place) = differing(&session).unwrap();
    let mut sections = std::collections::BTreeMap::new();
    sections.insert("vtt".to_string(), GameBindings::decode(&[]).unwrap().encode().unwrap());
    let world = World::with_sections(session, sections);
    let join = || ParticipantHandle(world.command(Command::Join).unwrap().strip_prefix("participant:").unwrap()
        .parse().unwrap());
    let (teller_player, hearer_player) = (join(), join());
    world.command(Command::Take { participant: teller_player.0, critter: teller }).unwrap();
    world.command(Command::Take { participant: hearer_player.0, critter: hearer }).unwrap();
    let mut player = VttEndpoint::open(&world, teller_player, Seat::Player).unwrap();
    // The card offers the event itself: unknown to the hearer, at a place it knows.
    let offered = world.read(|s| s.sim.tellable_events(teller, hearer, s.sim.genesis().rules.telling().event_hops)).unwrap();
    assert!(offered.iter().any(|(e, _)| *e == event), "the untold event is offered: {offered:?}");
    let snap = snapshot(&mut player);
    let (instance, _) = snap.scene.active_items_in_order().into_iter().find(|(_, item)|
        snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == format!("tell:{teller}:{hearer}")).unwrap();
    let event_action = format!("vtt.tell-event-{}", cosmograph_overlay::key_handle(&event));
    assert!(snap.presentation.offers_for(instance).unwrap().iter()
        .any(|o| o.semantics.actions.iter().any(|a| a.intent.0 == event_action)), "the table offers {event_action}");
    act(&mut player, &format!("tell:{teller}:{hearer}"), &format!("vtt.tell-{place}"));
    let (taken, knows, shown) = world.read(|s| {
        let outcome = &s.entries.iter().rev().find(|e| matches!(e.command, Command::Know(_))).unwrap().outcome;
        let telling: cosmograph::knowing::telling::Telling = serde_json::from_str(outcome).unwrap();
        let v = view(&s.sim, hearer, Mode::Survival);
        (telling.taken, s.sim.knows(hearer, &event).unwrap(), v.events.contains(&event) && v.sites.contains(&place))
    }).unwrap();
    assert_eq!(knows, taken, "the hearer knows the event exactly when the draw took the telling");
    assert_eq!(shown, taken, "and its view shows the place only then");
    let save = world.save().unwrap();
    let replayed = cosmograph::Session::load(save.native.clone(), cosmograph::Execution::Individuals).unwrap();
    assert_eq!(cosmograph::digest(&replayed.save()), cosmograph::digest(&save.native), "the telling replays alike");
}
