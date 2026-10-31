// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Command, Execution, Founding, Session,
    directing::{self, Played, interim::{Interim, Pace, Start}, readings::Mode},
    rules::{DeepTimeSpan, Effect, Query}, schema::Id, simulation::Genesis};

const PACE: Pace = Pace { round: 1, scoring: 1 };

fn genesis(wait_for_habitat: bool) -> Genesis {
    let mut genesis = Founding { seed: 71, sites: 1, population: 2,
        cohort_size: 1, lineages: 1, stock_min: 20, stock_max: 20,
        played: Some(Played { lineage: 0, region_sites: 1 }),
        ..Default::default() }.generate().unwrap();
    genesis.rules.epoch_ticks = 2;
    genesis.rules.deep_time = DeepTimeSpan { epochs: 1 };
    if wait_for_habitat {
        genesis.sites.get_mut(&0).unwrap().conditions.insert("world:habitable".into(), 0);
        let restoration = genesis.rules.processes.get_mut("world:weather").unwrap();
        restoration.period = Some(1);
        restoration.requires = vec![Query::Age { at_least: 4 }];
        restoration.effects = vec![Effect::Condition { key: "world:habitable".into(), delta: 1 }];
    }
    genesis.validate().unwrap();
    genesis
}

fn clock(session: &mut Session) -> Result<(), String> {
    session.command(Command::AdvanceGeometry { ticks: 1, resolutions: vec![] })?;
    Ok(())
}

fn witnessed(mode: Execution, wait_for_habitat: bool) -> Session {
    let mut session = Session::new(genesis(wait_for_habitat), mode).unwrap();
    session.branch = "branch:actual-witness".into();
    clock(&mut session).unwrap();
    // The read uses the actual saved native witness, not an advisory epoch/tick reconstruction.
    Session::load(session.save(), mode).unwrap()
}

fn first_life(session: &Session) -> Id {
    *session.sim.state().population.groups.iter()
        .find(|(_, group)| group.entity.lineage == "lineage:0" && group.entity.alive)
        .unwrap().0
}

fn join(session: &mut Session) -> Id {
    session.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap()
}

#[test]
fn exact_prefix_excludes_later_same_tick_join_and_take_without_forking() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = witnessed(mode, false);
        let accepted = session.save();
        let count = session.entries.len();
        let critter = first_life(&session);
        let participant = join(&mut session);
        session.command(Command::Take { participant, critter }).unwrap();
        assert_eq!(session.sim.state().tick, accepted.tick);
        let prefix = session.prefix(count, accepted.tick).unwrap();
        assert_eq!(prefix.save(), accepted);
        assert_eq!(prefix.branch, "branch:actual-witness");
        assert_eq!(prefix.sim.mode(), mode);
        assert_eq!(prefix.sim.genesis(), session.sim.genesis());
        assert!(prefix.sim.state().population.get(participant).is_none());
        assert_eq!(prefix.sim.plays(participant), None);
        assert_eq!(session.sim.plays(participant), Some(critter));
        assert_eq!(session.entries.len(), count + 2);
    }
}

#[test]
fn prefix_refuses_a_clock_crossing_the_requested_cut_or_outside_source() {
    let mut session = Session::new(genesis(false), Execution::Individuals).unwrap();
    session.command(Command::AdvanceGeometry { ticks: 1, resolutions: vec![] }).unwrap();
    assert_eq!(session.sim.state().tick, 1);
    assert!(session.prefix(1, 0).unwrap_err().contains("splits"));
    assert!(session.prefix(2, 1).unwrap_err().contains("exceeds the accepted source"));
    assert!(session.prefix(1, 2).unwrap_err().contains("exceeds the accepted source"));
    assert!(session.prefix(0, 1).unwrap_err().contains("requested tick disagree"));
    assert_eq!(session.prefix(1, 1).unwrap().save(), session.save());
}

#[test]
fn begin_using_runs_deep_wait_and_start_once_on_the_witness_ended_owner() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let session = witnessed(mode, true);
        let accepted = session.save();
        let mut calls = 0;
        let interim = Interim::begin_using(session, Start { within: 1, epochs: 1 },
            Mode::Survival, PACE, &mut |session| { calls += 1; clock(session) }).unwrap();
        assert_eq!(calls, 5, "tick1 -> deep boundary2 -> habitable boundary4 -> added boundary6");
        assert_eq!(interim.session.sim.state().tick, 6);
        assert_eq!(interim.habitable_at, 4);
        assert_eq!(interim.handover.span.epochs, 3);
        assert_eq!((interim.handover.from_tick, interim.handover.to_tick), (1, 6));
        assert_eq!((interim.handover.from_epoch, interim.handover.to_epoch), (0, 3));
        assert_eq!(interim.session.sim.mode(), mode);
        assert_eq!(interim.session.branch, accepted.branch);
        assert_eq!(interim.session.sim.genesis(), &accepted.genesis);
        assert_eq!(interim.session.prefix(accepted.entries.len(), accepted.tick).unwrap().save(), accepted);
        assert!(interim.session.entries.starts_with(&accepted.entries));
        assert_eq!(interim.session.entries.len(), accepted.entries.len() + calls + 2);
        assert!(matches!(interim.session.entries[interim.session.entries.len() - 2].command, Command::Join));
        let critter = interim.critter().unwrap();
        assert!(matches!(interim.session.entries.last().unwrap().command,
            Command::Take { participant, critter: taken } if participant == interim.participant && taken == critter));
        assert!(directing::is_participant(interim.session.sim.state().population.get(interim.participant).unwrap()));
        assert_eq!(interim.session.sim.plays(interim.participant), Some(critter));
        let restored = Session::load(interim.session.save(), mode).unwrap();
        assert_eq!(restored.save(), interim.session.save());
    }
}

#[test]
fn begin_using_refuses_an_existing_controller_before_any_callback() {
    let mut session = witnessed(Execution::Individuals, false);
    join(&mut session);
    let mut calls = 0;
    let result = Interim::begin_using(session, Start::default(), Mode::Creative, PACE,
        &mut |session| { calls += 1; clock(session) });
    assert!(result.unwrap_err().contains("existing controller must be resumed"));
    assert_eq!(calls, 0);
}

#[test]
fn begin_using_rejects_a_callback_that_skips_the_one_tick_boundary() {
    let session = witnessed(Execution::Individuals, false);
    let result = Interim::begin_using(session, Start::default(), Mode::Creative, PACE,
        &mut |session| { session.advance(2)?; Ok(()) });
    assert!(result.unwrap_err().contains("one-tick boundary"));
}

#[test]
fn begin_using_rejects_callback_prefix_loss_or_a_different_branch() {
    for replace_branch in [false, true] {
        let session = witnessed(Execution::Individuals, false);
        let result = Interim::begin_using(session, Start::default(), Mode::Creative, PACE,
            &mut |session| {
                clock(session)?;
                if replace_branch { session.branch = "branch:another-owner".into(); }
                else { session.entries.remove(0); }
                Ok(())
            });
        assert!(result.unwrap_err().contains("native source, prefix or one-tick boundary"));
    }
}

#[test]
fn begin_using_refuses_non_clock_callback_entries() {
    let session = witnessed(Execution::Individuals, false);
    let result = Interim::begin_using(session, Start::default(), Mode::Creative, PACE,
        &mut |session| {
            clock(session)?;
            session.command(Command::Join)?;
            Ok(())
        });
    assert!(result.unwrap_err().contains("permits only clocks and physical patch inputs"));
}
