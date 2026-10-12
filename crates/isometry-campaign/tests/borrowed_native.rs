// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{Execution, Founding, Session, asserted::{Assertion as Native, Place}};
use isocosm_overlay::{PlaceHandle, WorldPoint, vtt::{Assertion, Cell, NewCharacter}};
use isometry_campaign::native::{Replay, submit_contract_assertions};

// The already declared campaign seed-7/site-2/pop-4/cohort-2 founding.
fn base() -> Replay {
    let genesis = Founding { seed: 7, sites: 2, population: 4, lineages: 2, cohort_size: 2,
        ..Default::default() }.generate().unwrap();
    Replay::from_assertions(genesis, &[Native::Place(Place { key: "ford".into(), name: "Ford".into(),
        ..Default::default() })], Execution::Grouped).unwrap()
}

fn character(key: &str, site: u64) -> Assertion {
    Assertion::Character(NewCharacter { key: key.into(), name: key.into(), faction: None,
        at: Cell { site: PlaceHandle(site), at: WorldPoint([-7, 2, 13]) }, owner: None })
}

#[test]
fn borrowed_vtt_matches_existing_native_replay_and_allocated_identity() {
    let base = base();
    let site = base.reading().authored_site("ford").unwrap().0;
    let batch = [character("mira", site)];
    let expected = base.with_contract_assertions(&batch).unwrap();
    let mut native = Session::load(base.save(), Execution::Grouped).unwrap();
    let allocated = native.sim.state().population.next_id;
    submit_contract_assertions(&mut native, &batch).unwrap();
    assert_eq!(native.sim.authored_entity("mira").unwrap().0, allocated);
    assert_eq!(native.save(), expected.save(), "borrowed VTT retains native history and assertion identities");
}

#[test]
fn borrowed_vtt_late_lowering_refusal_publishes_none_of_the_batch() {
    let base = base();
    let site = base.reading().authored_site("ford").unwrap().0;
    let mut native = Session::load(base.save(), Execution::Grouped).unwrap();
    let before = native.save();
    let batch = [character("mira", site), character("elsewhere", u64::MAX)];
    assert!(submit_contract_assertions(&mut native, &batch).is_err());
    assert_eq!(native.save(), before, "borrowed VTT does not publish its accepted prefix after a later refusal");
}
