// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "attention/fixture.rs"]
mod fixture;
use isocosm::{Command, Execution, Session, attention::{Region, WindowAttention},
    directing::readings::{self, Mode}, rules::{Causation, Process}, schema::Id};
use std::collections::BTreeSet;

/// Existing seed-11 body fixture, with one explicitly declared inert noted
/// process and isolated reach routes. All bodies/participants are founded,
/// Joined, and Taken through the existing native doors, never patched state.
fn world() -> (Session, [Id; 2], [Id; 2]) {
    let (base, participants, subjects) = fixture::fixture();
    let mut genesis = base.sim.genesis().clone();
    for site in genesis.sites.values_mut() {
        for route in &mut site.routes { route.transmission = 0; }
    }
    genesis.rules.field.strength = 1_000_000;
    genesis.rules.field.decay_per_tick = 1;
    let key = "process:host-stream-fixture";
    genesis.rules.processes.insert(key.into(), Process { id: key.into(), causation: Causation::Choice,
        requires: vec![], commitments: vec![], effects: vec![], risk: None, target: None,
        period: None, priority: 0, need_account: None, need_below: 0, glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(), note: true });
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    for entry in base.entries { session.command(entry.command).unwrap(); }
    for index in 0..2 { assert_eq!(session.sim.plays(participants[index]), Some(subjects[index])); }
    (session, participants, subjects)
}

fn act(session: &mut Session, actor: Id) -> String {
    let outcome = session.command(Command::Act { actor, target: None,
        process: "process:host-stream-fixture".into(), cause: None }).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(receipt.accepted(), "declared inert note process must admit through native Act");
    assert!(session.sim.state().events.contains_key(&receipt.id));
    receipt.id
}

fn outsider(session: &Session, subject: Id, same_site: bool) -> Id {
    let played = session.sim.state().population.get(subject).unwrap();
    *session.sim.state().population.groups.iter().find(|(_, group)| group.entity.alive
        && !isocosm::directing::is_participant(&group.entity)
        && (group.entity.place == played.place) == same_site
        && group.entity.lineage != played.lineage).expect("declared fixture supplies another native lineage").0
}

#[test]
fn stream_retains_a_played_reaching_record_outside_every_attention_target() {
    let (mut session, participants, subjects) = world();
    session.attach_window(participants[0], "mesocosm".into(), WindowAttention::default()).unwrap();
    let actor = outsider(&session, subjects[0], true);
    let event = act(&mut session, actor);
    let attention = session.sim.player_attention(participants[0]).unwrap();
    assert!(attention.pinned.is_empty() && attention.examined.is_empty());
    assert_ne!(session.sim.state().population.get(actor).unwrap().lineage,
        session.sim.state().population.get(subjects[0]).unwrap().lineage);
    assert!(session.sim.attended_events(participants[0], Mode::Survival).unwrap().iter()
        .any(|e| e.id == event), "213 retains records reaching played despite no attended target");
}

#[test]
fn stream_union_close_and_survival_cut_use_the_same_saved_native_events() {
    let (mut session, participants, subjects) = world();
    let actor = outsider(&session, subjects[0], false);
    let site = session.sim.state().population.get(actor).unwrap().place;
    let event = act(&mut session, actor);
    let first = session.attach_window(participants[0], "mesocosm".into(), WindowAttention::default()).unwrap();
    let second = session.attach_window(participants[0], "mesocosm".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    session.attach_window(participants[1], "vtt".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    assert!(session.sim.attended_events(participants[0], Mode::Creative).unwrap().iter().any(|e| e.id == event));
    assert!(!readings::view(&session.sim, subjects[0], Mode::Survival).events.contains(&event));
    assert!(!session.sim.attended_events(participants[0], Mode::Survival).unwrap().iter()
        .any(|e| e.id == event), "the stream cannot publish an unknown watched event in Survival");
    session.close_attention(participants[0], second).unwrap();
    assert!(!session.sim.attended_events(participants[0], Mode::Creative).unwrap().iter()
        .any(|e| e.id == event), "closing one contribution withdraws its unshared region only");
    assert!(session.sim.attention_window(participants[0], first.id()).is_ok());
    let saved = session.save();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(saved.clone(), execution).unwrap();
        assert_eq!(replay.save(), saved, "ordinary stream filtering never deletes native replay history");
        assert_eq!(replay.sim.attended_events(participants[0], Mode::Creative).unwrap(),
            session.sim.attended_events(participants[0], Mode::Creative).unwrap());
        assert!(replay.sim.state().events.contains_key(&event));
    }
}

#[test]
fn stream_refuses_a_nonparticipant_without_changing_native_history() {
    let (session, _, subjects) = world();
    let before = session.save();
    assert!(session.sim.attended_events(subjects[0], Mode::Creative).is_err());
    assert_eq!(session.save(), before);
}
