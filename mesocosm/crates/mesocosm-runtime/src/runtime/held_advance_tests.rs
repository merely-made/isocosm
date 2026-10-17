// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{Execution, directing::interim::{Interim, Pace, Start}, directing::readings::Mode};
use isocosm_overlay::{EntityHandle, Intent, IntentEnvelope, ParticipantHandle, Tick,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, MesocosmIntent}};

use crate::{Founded, Runtime};

#[path = "../../../../../shared/isocosm/tests/ecology_independence_879/fixture.rs"]
#[allow(dead_code)]
mod native_fixture;

const PACE: Pace = Pace { round: 1, scoring: 1 };

fn held() -> Runtime {
    let mut candidate = native_fixture::candidate_with_played("test:offered-cycle", 1);
    candidate.genesis.rules.epoch_ticks = 6;
    let interim = Interim::found(candidate.genesis, Start::default(), Mode::Survival, PACE,
        Execution::Grouped).unwrap();
    let mut runtime = Runtime::resume(interim.session.save(), Mode::Survival, PACE, 30).unwrap();
    assert_eq!(runtime.step(1), 1);
    assert!(runtime.checkpoint().is_some());
    runtime
}

#[test]
fn held_advance_discards_time_but_drains_a_queued_answer() {
    let mut runtime = held();
    let tick = runtime.tick();
    assert_eq!(runtime.advance(60_000_000), 0);
    assert_eq!(runtime.tick(), tick);

    let envelope = IntentEnvelope {
        tick: Tick(runtime.tick()),
        participant: ParticipantHandle(runtime.interim().participant),
        intent: Intent::Game(MesocosmIntent::Checkpoint(CheckpointAnswer {
            entity: EntityHandle(runtime.critter().expect("played critter")),
            kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent),
        })),
    };
    runtime.queue(envelope);
    assert_eq!(runtime.queued_len(), 1);
    assert_eq!(runtime.advance(0), 0);
    assert_eq!(runtime.queued_len(), 0);
    assert!(runtime.checkpoint().is_none());
    assert_eq!(runtime.tick(), tick);
    assert_eq!(runtime.advance(0), 0);
    assert_eq!(runtime.tick(), tick, "discarded held time must not be banked");
}

#[test]
fn held_advance_adopts_external_world_revision() {
    let mut runtime = held();
    let before = runtime.seen.expect("runtime published initial state");
    assert!(runtime.current_loop_phase().is_some());
    runtime.world().put_section("held-advance-control", vec![1]).unwrap();
    assert_eq!(runtime.current_loop_phase(), None);
    let changed = runtime.world().revision().unwrap();
    assert_ne!(changed, before);

    assert_eq!(runtime.advance(0), 0);
    assert_eq!(runtime.seen, Some(runtime.world().revision().unwrap()));
    assert_eq!(runtime.seen, Some(changed));
    assert!(runtime.checkpoint().is_some());
    assert!(runtime.current_loop_phase().is_some());
}

#[test]
fn failed_publish_does_not_mark_world_revision_clean() {
    use std::{cell::{Cell, RefCell}, collections::BTreeMap, rc::Rc};
    use isocosm::Session;
    use isocosm_host::world::CommitGate;

    let runtime = Rc::new(RefCell::new(Runtime::bench(Founded::generated(9, 256).unwrap(), 30).unwrap()));
    let world = runtime.borrow().world().share();
    assert!(runtime.borrow().seen.is_some());
    let before = world.section(super::MODE).unwrap();
    let rejected = Rc::new(Cell::new(true));
    let calls = Rc::new(Cell::new(0));
    struct RejectPublication { rejected: Rc<Cell<bool>>, calls: Rc<Cell<u32>> }
    impl CommitGate for RejectPublication {
        fn before_publish(&self, _: &Session, _: &BTreeMap<String, Vec<u8>>, _: u64,
            _: &Session, _: &BTreeMap<String, Vec<u8>>, _: u64) -> Result<(), String> {
            self.calls.set(self.calls.get() + 1);
            if self.rejected.get() { Err("test publication refusal".into()) } else { Ok(()) }
        }
    }
    world.install_commit_gate(Rc::new(RejectPublication {
        rejected: rejected.clone(), calls: calls.clone(),
    })).unwrap();
    // An identical section bypasses the gate. Change actual saved product
    // state so the refusal exercises the publication path.
    runtime.borrow_mut().driver.product_mut().fault = Some("publication control".into());
    runtime.borrow_mut().publish();
    assert_eq!(calls.get(), 1);
    assert_eq!(runtime.borrow().seen, None);
    assert_eq!(runtime.borrow().current_loop_phase(), None);
    assert_eq!(world.section(super::MODE).unwrap(), before);
    rejected.set(false);
    runtime.borrow_mut().publish();
    assert_eq!(calls.get(), 2);
    assert_eq!(runtime.borrow().seen, Some(world.revision().unwrap()));
    assert_ne!(world.section(super::MODE).unwrap(), before);
}
