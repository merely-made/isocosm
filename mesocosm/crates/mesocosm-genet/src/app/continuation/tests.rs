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
