// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm_overlay::{EntityHandle, Intent, IntentEnvelope, ParticipantHandle, Tick,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, MesocosmIntent}};
use std::time::Instant;

/// Reuses the qualified generated A1 save. Never writes another full capture.
#[cfg_attr(test, test)]
#[cfg_attr(test, ignore = "explicit saved-world profiling receipt")]
pub(super) fn saved_active_update() {
    let path = std::env::var("MESOCOSM_ACTIVE_SAVE").expect("explicit qualified save");
    let bytes = std::fs::read(path).unwrap();
    let started = Instant::now();
    let saved = Runtime::decode_world_save(&bytes).unwrap();
    let mut runtime = Runtime::resume_world(saved, 10, None).unwrap();
    eprintln!("[active-case] restore_ms={}", started.elapsed().as_millis());
    let before = runtime.tick();
    let reading = crate::profile::measure("case.reading", || runtime.loop_reading()).unwrap();
    assert!(reading.checkpoint.is_some());
    runtime.queue(IntentEnvelope { tick: Tick(runtime.tick()),
        participant: ParticipantHandle(runtime.interim().participant),
        intent: Intent::Game(MesocosmIntent::Checkpoint(CheckpointAnswer {
            entity: EntityHandle(runtime.critter().unwrap()),
            kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent),
        })) });
    crate::profile::measure("case.answer", || runtime.advance(0));
    assert!(runtime.checkpoint().is_none(), "exact saved birth answer accepted");
    let elapsed = Instant::now();
    assert_eq!(runtime.step(1), 1);
    let step_us = elapsed.elapsed().as_micros();
    assert_eq!(runtime.tick(), before + 1);
    assert!(runtime.fault().is_none());
    let reading = crate::profile::measure("case.after_reading", || runtime.loop_reading()).unwrap();
    let receipt = serde_json::json!({"input_bytes":bytes.len(), "before_tick":before,
        "tick":runtime.tick(), "step_us":step_us, "native_hash":runtime.state_hash(),
        "native_digest":isocosm::digest(&runtime.session().save()),
        "section_digest":isocosm::digest(&runtime.world().section(MODE).unwrap()),
        "world_revision":runtime.revision().unwrap(), "entries":runtime.session().entries.len(),
        "trace":isocosm::digest(&runtime.trace()), "loop_reading":isocosm::digest(&reading)});
    eprintln!("[active-case] {receipt}");
}
