// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn valid_later_cpu_remains_observable_after_an_unavailable_or_rejected_call() {
    let mut total = StageTotal::default();
    total.add(Measured {
        wall_ns: 10,
        thread_cpu_ns: Some(0),
    });
    total.add(Measured {
        wall_ns: 20,
        thread_cpu_ns: None,
    });
    let before = total.clone();
    total.add(Measured {
        wall_ns: 30,
        thread_cpu_ns: Some(7),
    });
    assert_eq!(total.thread_cpu_ns, None);
    assert_eq!(
        total.observed_thread_cpu_ns - before.observed_thread_cpu_ns,
        7
    );
    assert_eq!(
        total.thread_cpu_observed_calls - before.thread_cpu_observed_calls,
        1
    );
    assert_eq!(
        total.thread_cpu_unavailable_calls - before.thread_cpu_unavailable_calls,
        0
    );
    total.add(Measured {
        wall_ns: 5,
        thread_cpu_ns: Some(6),
    });
    assert_eq!(total.wall_ns, 65);
    assert_eq!(total.observed_thread_cpu_ns, 7);
    assert_eq!(total.thread_cpu_observed_calls, 2);
    assert_eq!(total.thread_cpu_unavailable_calls, 1);
    assert_eq!(total.thread_cpu_rejected_calls, 1);
    assert_eq!(total.rejected_thread_cpu_ns, Some(6));
    assert!(!total.overflowed);
}

#[test]
fn counters_do_not_wrap_and_unperformed_scopes_stay_absent() {
    let mut work = PreparationCosts::default();
    assert!(work.stages.is_empty());
    assert!(work.counters.is_empty());
    assert!(work.last_target_extent.is_none());
    work.count("target_texels_created", u64::MAX);
    work.count("target_texels_created", 1);
    assert!(work.counter_overflowed);
    assert_eq!(work.counters["target_texels_created"], u64::MAX);
}

#[test]
fn measuring_refusal_preserves_its_result_and_allows_nested_counter_updates() {
    let costs = super::super::costs::Costs::default();
    let epoch = costs.preparation().cost_epoch;
    let result = costs.measure("resource_setup", || {
        costs.count("section_create_calls", 1);
        Err::<(), _>("controlled refusal")
    });
    assert_eq!(result, Err("controlled refusal"));
    let work = costs.preparation();
    assert_eq!(work.cost_epoch, epoch);
    assert_eq!(work.stages["resource_setup"].calls, 1);
    assert!(work.stages.get("draw_work").is_none());
    assert_eq!(work.counters["section_create_calls"], 1);
    assert!(work.counters.get("sections_created").is_none());
}

#[test]
fn gpu_retirement_keeps_cost_epoch_and_a_new_producer_cannot_alias_it() {
    use cambium_rootstock::TextureProducer;
    let mut first = super::super::SectionProducer::new(super::super::SceneHandle::default());
    let epoch = first
        .preparation_costs()
        .cost_epoch
        .expect("diagnostic epoch");
    first.costs.count("native_stage_calls", 3);
    first.retire();
    assert_eq!(first.preparation_costs().cost_epoch, Some(epoch));
    assert_eq!(first.preparation_costs().counters["native_stage_calls"], 3);
    let other = super::super::SectionProducer::new(super::super::SceneHandle::default());
    assert_ne!(other.preparation_costs().cost_epoch, Some(epoch));
    assert!(
        other
            .preparation_costs()
            .counters
            .get("native_stage_calls")
            .is_none()
    );
}
