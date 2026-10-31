// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "attention/fixture.rs"]
mod fixture;
use fixture::fixture;
use cosmograph::{Command, Execution, Session, attention::{Region, Target, WindowAttention}};
use std::collections::BTreeSet;

fn value(site: u64, pin: u64) -> WindowAttention {
    WindowAttention { pinned: BTreeSet::from([Target::Entity(pin)]), examined: Some(Region::Site(site)) }
}

#[test]
fn native_join_take_two_windows_union_and_close_only_one_contribution() {
    let (mut session, players, subjects) = fixture();
    let body = session.sim.state().population.get(subjects[0]).unwrap().clone();
    assert!(body.body.is_some(), "fixture must bind a native body");
    let first = session.attach_window(players[0], "mesocosm".into(), value(0, subjects[0])).unwrap();
    let second = session.attach_window(players[0], "eponym".into(), value(1, subjects[1])).unwrap();
    session.attach_window(players[1], "vtt".into(), value(2, players[1])).unwrap();
    let set = session.sim.player_attention(players[0]).unwrap();
    assert_eq!(set.played, BTreeSet::from([subjects[0]]));
    assert_eq!(set.pinned, BTreeSet::from([Target::Entity(subjects[0]), Target::Entity(subjects[1])]),
        "native admitted windows must both contribute pins");
    assert_eq!(set.examined, BTreeSet::from([Region::Site(0), Region::Site(1)]),
        "native admitted windows must both contribute close-up regions");
    assert!(set.care.contains(&Target::Lineage(body.lineage.clone())));
    session.close_attention(players[0], first).unwrap();
    let set = session.sim.player_attention(players[0]).unwrap();
    assert_eq!(set.pinned, BTreeSet::from([Target::Entity(subjects[1])]),
        "native close must preserve the other window contribution");
    assert_eq!(set.examined, BTreeSet::from([Region::Site(1)]));
    assert_eq!(session.sim.attention_window(players[0], second.id()).unwrap().0, second);
    assert_eq!(session.sim.plays(players[0]), Some(subjects[0]));
    assert_eq!(session.sim.state().population.get(subjects[0]).unwrap(), &body);
}

#[test]
fn native_foreign_stale_profile_and_typed_target_refusals_are_atomic() {
    let (mut session, players, subjects) = fixture();
    let window = session.attach_window(players[0], "mesocosm".into(), value(0, subjects[0])).unwrap();
    let before = session.save();
    assert!(session.close_attention(players[1], window).is_err(), "native window owner must refuse foreign close");
    assert!(session.replace_attention(players[1], window, "vtt".into(), value(1, subjects[1])).is_err());
    assert!(session.attach_window(subjects[0], "mesocosm".into(), WindowAttention::default()).is_err());
    assert!(session.attach_window(players[0], "unknown-profile".into(), WindowAttention::default()).is_err());
    let unknown = [Target::Entity(u64::MAX), Target::Site(u64::MAX), Target::Location(u64::MAX),
        Target::Lineage("missing-lineage".into()), Target::Faction(u64::MAX), Target::Event("missing-event".into())];
    for target in unknown {
        let invalid = WindowAttention { pinned: BTreeSet::from([target]), ..Default::default() };
        let direct = Command::Window(cosmograph::attention::Change::Replace { participant: players[0],
            window, profile: "mesocosm".into(), attention: invalid.clone() });
        assert!(session.command(direct).is_err(), "direct native typed-target admission must refuse before mutation");
        assert!(session.replace_attention(players[0], window, "mesocosm".into(), invalid).is_err(),
            "native typed-target admission must refuse before mutation");
        assert_eq!(session.save(), before, "native refused admission must preserve history and witness");
    }
    let invalid = WindowAttention { examined: Some(Region::Location(u64::MAX)), ..Default::default() };
    assert!(session.replace_attention(players[0], window, "vtt".into(), invalid).is_err());
    assert_eq!(session.save(), before);
    session.close_attention(players[0], window).unwrap();
    let closed = session.save();
    let reused = Command::Window(cosmograph::attention::Change::Attach { participant: players[0],
        window, profile: "mesocosm".into(), attention: WindowAttention::default() });
    assert!(session.command(reused).is_err(), "an issued native identity cannot be reused after close");
    assert!(session.close_attention(players[0], window).is_err());
    assert!(session.replace_attention(players[0], window, "vtt".into(), WindowAttention::default()).is_err());
    assert_eq!(session.save(), closed);
}

#[test]
fn native_group_care_stays_crowd_but_a_logged_close_up_is_individual() {
    // Reuses directing's declared seed-7, 48-member, four-site crowd fixture.
    let genesis = cosmograph::Founding { seed: 7, sites: 4, population: 48,
        cohort_size: 4, lineages: 3, ecology: true,
        played: Some(cosmograph::directing::Played { lineage: 1, region_sites: 2 }),
        ..Default::default()
    }.generate().unwrap();
    let mut session = Session::new(genesis, Execution::Grouped).unwrap();
    let subject = *session.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.method == cosmograph::schema::Method::Deliberative).unwrap().0;
    let joined = session.command(Command::Join).unwrap();
    let participant = joined.strip_prefix("participant:").unwrap().parse().unwrap();
    session.command(Command::Take { participant, critter: subject }).unwrap();
    let (&first, cohort) = session.sim.state().population.groups.iter()
        .find(|(_, g)| g.count > 1 && g.entity.place == 0).expect("declared crowd fixture needs a grouped row at site zero");
    let lineage = cohort.entity.lineage.clone();
    let group = WindowAttention { pinned: BTreeSet::from([Target::Lineage(lineage)]), ..Default::default() };
    session.attach_window(participant, "vtt".into(), group).unwrap();
    assert!(session.sim.state().population.groups.get(&first).unwrap().count > 1,
        "caring for a group must keep its fungible members as a crowd");
    let close_up = WindowAttention { examined: Some(Region::Site(0)), ..Default::default() };
    session.attach_window(participant, "vtt".into(), close_up).unwrap();
    for cohort in session.sim.state().population.groups.values().filter(|g| g.entity.alive && g.entity.place == 0) {
        assert_eq!(cohort.count, 1, "a logged close-up region must run its members individually");
    }
}

#[test]
fn close_buffer_and_global_inspect_release_remain_separate_keep_reasons() {
    let (mut session, players, subjects) = fixture();
    let pin = *session.sim.state().population.groups.keys().find(|id| !subjects.contains(id) && !players.contains(id)).unwrap();
    let local = WindowAttention { pinned: BTreeSet::from([Target::Entity(pin)]), ..Default::default() };
    let first = session.attach_window(players[0], "mesocosm".into(), local.clone()).unwrap();
    let second = session.attach_window(players[0], "mesocosm".into(), local).unwrap();
    session.command(Command::Inspect(pin)).unwrap();
    session.close_attention(players[0], first).unwrap();
    assert!(session.sim.kept().contains(&pin));
    assert!(session.sim.state().roots.contains(&pin), "window close cannot perform global Release");
    assert!(!session.sim.state().attention.buffered(session.sim.state().tick).contains(&pin));
    session.close_attention(players[0], second).unwrap();
    assert!(session.sim.state().attention.buffered(session.sim.state().tick).contains(&pin),
        "last owned pin removal must retain the native collection buffer");
    assert!(session.sim.state().roots.contains(&pin));
    session.command(Command::Release(pin)).unwrap();
    assert!(session.sim.kept().contains(&pin));
    session.advance(3).unwrap();
    assert!(!session.sim.state().attention.buffered(session.sim.state().tick).contains(&pin));
}

#[test]
fn eponym_care_recomputes_from_native_take_and_vtt_care_from_assertions() {
    let (mut session, players, subjects) = fixture();
    let care = session.sim.profile_care(players[0], "eponym").unwrap();
    assert!(care.contains(&Target::Entity(subjects[0])));
    session.command(Command::Take { participant: players[0], critter: subjects[1] }).unwrap();
    let next = session.sim.profile_care(players[0], "eponym").unwrap();
    assert!(next.contains(&Target::Entity(subjects[1])));
    let visible = cosmograph::directing::readings::view(&session.sim, subjects[1], cosmograph::directing::readings::Mode::Survival);
    for (first, count) in visible.things {
        for id in first..first + count { assert!(next.contains(&Target::Entity(id))); }
    }
    let window = session.attach_window(players[0], "vtt".into(), WindowAttention::default()).unwrap();
    let asserted = session.command(Command::Assert(cosmograph::asserted::Assertion::Character(
        cosmograph::asserted::Character { key: "attention-fixture-character".into(), name: "Witness".into(), ..Default::default() }
    ))).unwrap();
    let authored: u64 = asserted.strip_prefix("entity:").unwrap().parse().unwrap();
    assert!(session.sim.player_attention(players[0]).unwrap().care.contains(&Target::Entity(authored)),
        "admitted VTT care must include subsequently authored native characters");
    assert_eq!(session.sim.attention_window(players[0], window.id()).unwrap().1, "vtt");
}

#[test]
fn native_window_history_witness_current_save_and_fork_replay_exactly() {
    let (mut session, players, subjects) = fixture();
    let window = session.attach_window(players[0], "mesocosm".into(), value(0, subjects[0])).unwrap();
    session.replace_attention(players[0], window, "eponym".into(), value(1, subjects[1])).unwrap();
    session.advance(2).unwrap();
    let saved = session.save();
    assert!(serde_json::to_value(&saved.witness).unwrap().to_string().contains("attention"));
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(Session::load(saved.clone(), mode).unwrap().save(), saved);
    }
    assert_eq!(session.fork_at(session.sim.state().tick, "attention-replay".into()).unwrap().save().witness, saved.witness);
    let mut corrupt = saved;
    corrupt.state_hash ^= 1;
    assert!(Session::load(corrupt, Execution::Individuals).is_err());
}

#[test]
fn independent_branch_attachments_keep_their_native_identities_on_merge() {
    let (session, players, subjects) = fixture();
    let mut first = session.fork_at(0, "attention:first".into()).unwrap();
    let mut second = session.fork_at(0, "attention:second".into()).unwrap();
    let a = first.attach_window(players[0], "mesocosm".into(), value(0, subjects[0])).unwrap();
    let b = second.attach_window(players[0], "mesocosm".into(), value(1, subjects[1])).unwrap();
    assert_ne!(a, b, "branch attachments must not alias native window identities");
    let merged = first.merge(&second).unwrap();
    assert!(merged.changes.is_empty());
    assert!(merged.proposed.sim.attention_window(players[0], a.id()).is_ok());
    assert!(merged.proposed.sim.attention_window(players[0], b.id()).is_ok());
    assert_eq!(merged.proposed.sim.player_attention(players[0]).unwrap().examined,
        BTreeSet::from([Region::Site(0), Region::Site(1)]));
    Session::load(merged.proposed.save(), Execution::Individuals).unwrap();
}

#[test]
fn authentic_v1_v2_archives_load_without_attention_preimage_changes() {
    let archives = [include_str!("data/v1-checkpointed-world.json"), include_str!("data/v2-checkpointed-world.json")];
    for archive in archives {
        for mode in [Execution::Individuals, Execution::Grouped] {
            let session = Session::load_json(archive.as_bytes(), mode).unwrap();
            assert!(session.sim.state().attention.is_empty());
            let saved = session.save();
            assert_eq!(Session::load(saved.clone(), mode).unwrap().save(), saved);
        }
    }
}
