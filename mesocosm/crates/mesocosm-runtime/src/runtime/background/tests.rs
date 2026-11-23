// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founded, runtime::keymap::nudge};
use cosmograph_overlay::{AttentionChange, Intent, ParticipantHandle, Tick, mesocosm::NudgeMeaning};

fn running() -> Runtime {
    let mut runtime = Runtime::bench(Founded::generated(9, 256).unwrap(), 1).unwrap().with_background();
    assert_eq!(runtime.advance(1_000_000), 0);
    assert!(runtime.product().pending.is_some());
    runtime
}

fn queue_attention(runtime: &mut Runtime) {
    runtime.queue(crate::Envelope {
        tick: Tick(runtime.tick()),
        participant: ParticipantHandle(runtime.interim().participant),
        intent: Intent::Attention(AttentionChange::StopExamining),
    });
}

fn queue_nudge(runtime: &mut Runtime) {
    let actor = runtime.critter().unwrap();
    let site = runtime.played().unwrap().resident_site().unwrap();
    runtime.queue(runtime.envelope(nudge(actor, site, None, NudgeMeaning::Attend)));
}

fn finish_authorised_round(runtime: &mut Runtime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while runtime.advance(0) == 0 {
        assert!(runtime.fault().is_none(), "{:?}", runtime.fault());
        assert!(std::time::Instant::now() < deadline, "lost the authorised round");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(runtime.tick(), 1);
    assert_eq!(runtime.advance(0), 0);
}

#[test]
fn a_pending_round_keeps_a_same_cut_native_fork() {
    let mut runtime = running();
    let before = runtime.world().cut().unwrap();
    runtime.world().product_current(|session| {
        *session = session.fork_at(session.sim.state().tick, "branch:current".into())?;
        Ok(())
    }).unwrap();
    assert_eq!(runtime.world().cut().unwrap(), before);
    queue_nudge(&mut runtime);
    finish_authorised_round(&mut runtime);
    assert_eq!(runtime.world().session().unwrap().branch, "branch:current");
    assert_eq!(runtime.trace().len(), 1);
    assert_eq!(runtime.trace()[0].1, Ok("nudged".into()));
    let reopened = Runtime::resume_world(runtime.save_world().unwrap(), 1, None).unwrap();
    assert_eq!(runtime.sim().state_hash(), reopened.sim().state_hash());
}

#[test]
fn an_unrelated_section_and_a_local_queue_publication_keep_the_running_round() {
    let mut runtime = running();
    queue_attention(&mut runtime);
    runtime.publish();
    runtime.world().put_section("another-mode", vec![7]).unwrap();
    finish_authorised_round(&mut runtime);
    assert_eq!(runtime.queued_len(), 1, "a spurious retry drains the future queue");
    assert!(runtime.trace().is_empty());
    assert_eq!(runtime.world().section("another-mode").unwrap(), Some(vec![7]));
}
