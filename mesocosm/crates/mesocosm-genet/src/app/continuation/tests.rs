// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn reading(label: &str) -> LoopReading {
    LoopReading {
        cut: continuation::LoopCut {
            native: label.into(),
            game: label.into(),
            question: None,
        },
        settings: None,
        phase: LoopPhase::Watching,
        checkpoint: None,
        offers: vec![],
        happenings: vec![],
        lineages: vec![],
        personal: None,
        world_ended_at: None,
        ending: Default::default(),
        sheet: None,
    }
}
#[test]
fn continuation_acceptance_keeps_the_prepared_cut_instead_of_a_later_reading() {
    let mut host = Host::new(super::super::HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        ..Default::default()
    });
    let a = reading("prepared-a");
    let b = reading("desired-b");
    host.continuation.reading = Some(a.clone());
    let prepared = host.prepared_continuation();
    host.continuation.reading = Some(b.clone());
    host.mark_continuation_presented(prepared);
    assert_eq!(host.continuation.presented, Some(a));
    assert_eq!(host.continuation.reading, Some(b));
    host.mark_continuation_presented(None);
    assert!(host.continuation.presented.is_none());
}

fn host(rounds: continuation::Rounds, condition: bool) -> Host {
    Host::new(super::super::HostConfig { bench_world: true, seed: 9, population: 256, rounds, condition, ..Default::default() })
}

fn press(host: &mut Host, key: &str) -> bool { host.try_continuation_key(&Key::Character(key.into())) }

#[test]
fn the_round_keys_send_their_answers_and_a_refusal_shows() {
    use continuation::Rounds;
    let mut host = host(Rounds::Capped, false);
    assert!(press(&mut host, "a"), "A is the round's key");
    assert!(host.runtime.interim().ending.extend, "an extension is pending");
    assert!(host.round_notice(host.runtime.tick()).is_none());
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim).0.as_deref(), Some("capped, extended"));
    drop(sim);
    // A second extension, and a call in a capped round, are refused: shown, not swallowed.
    assert!(press(&mut host, "A"));
    assert!(host.round_notice(host.runtime.tick()).is_some());
    host.round_notice = None;
    assert!(press(&mut host, "d"));
    assert!(host.round_notice(host.runtime.tick()).is_some());
    assert!(!host.runtime.interim().ending.call);
    assert!(host.events.iter().any(|event| event.starts_with("refused")));
    // Nobody is dead, so there is nothing to forfeit.
    host.round_notice = None;
    assert!(press(&mut host, "h"));
    assert!(host.round_notice(host.runtime.tick()).is_some());
    assert!(!host.runtime.interim().ending.forfeit);
}

#[test]
fn a_continuous_round_takes_its_call_and_a_waiting_one_waits_for_the_span() {
    use continuation::Rounds;
    let mut host = host(Rounds::Continuous, false);
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim).0.as_deref(), Some("continuous"));
    drop(sim);
    assert!(press(&mut host, "d"));
    assert!(host.runtime.interim().ending.call, "the call was admitted");
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim).0.as_deref(), Some("continuous, call pending"));
    drop(sim);
    assert!(press(&mut host, "a"), "extending is a capped round's");
    assert!(!host.runtime.interim().ending.extend);

    let mut host = self::host(Rounds::Waiting, false);
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim).0.as_deref(), Some("waiting, span running"));
    drop(sim);
    assert!(press(&mut host, "d"));
    assert!(host.round_notice(host.runtime.tick()).is_some(), "the span has not passed");
    assert!(!host.runtime.interim().ending.call);
}

#[test]
fn a_condition_shows_its_par_once_there_is_one() {
    let host = host(continuation::Rounds::Capped, true);
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim), (None, None), "no par before the first step");
    drop(sim);
    let mut host = host;
    host.runtime.step(1);
    let par = host.runtime.interim().ending.par.expect("the first step reads the par");
    let sim = host.runtime.sim().clone();
    assert_eq!(host.round_state(&sim), (None, Some(format!("{par} mg to beat"))));
}
