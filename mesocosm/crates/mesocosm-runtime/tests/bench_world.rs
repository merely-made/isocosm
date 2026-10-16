// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use isocosm::{Execution, directing::interim::Interim};
use mesocosm_runtime::Runtime;

#[test]
fn the_explicit_bench_founds_through_the_core_without_opening_the_head_gate() {
    let founded = worlds::held().founding().clone();
    assert!(Runtime::new(founded.clone(), worlds::RATE).is_err());
    let direct = Interim::found(
        founded.genesis.clone(),
        founded.start,
        founded.mode,
        founded.pace,
        Execution::Grouped,
    )
    .unwrap();
    let mut bench = Runtime::bench(founded, worlds::RATE).unwrap();
    assert_eq!(bench.state_hash(), direct.session.sim.state_hash());
    assert_eq!(bench.save(), direct.session.save());
    assert_eq!(bench.step(1), 1);
    assert!(bench.checkpoint().is_some());
    let saved = bench.save_world().unwrap();
    let restored = Runtime::resume_world(saved.clone(), worlds::RATE, None).unwrap();
    assert_eq!(restored.save_world().unwrap(), saved);
}
