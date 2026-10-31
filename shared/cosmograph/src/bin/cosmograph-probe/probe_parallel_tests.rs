// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{BodyFounding, HarmFounding, report::{ARMS, Verdicts}};
use cosmograph::probe::check::{self, Settings};

fn options(workers: usize) -> Options {
    Options {
        master: 2026101003, draws: 5, permutations: 1999, workers,
        output: None, trace: None, trace_entries: None, project: None, members: None,
        density: false, crowds: false, water: false, predators: false,
        approximate: false, bodies: true, harm: true,
    }
}

fn small_domain() -> Domain {
    Domain::Bodies(Box::new(BodyFounding {
        ticks: 4, sites: [1, 1], producers: [4, 4], grazers: [2, 2],
        harm: Some(HarmFounding::default()), ..Default::default()
    }))
}

fn canonical(o: &Options, draws: &[Completed]) -> serde_json::Value {
    let infos = &draws[0].infos;
    let bounds = infos.iter().map(|r| (r.key.clone(), r.bound_per_mille)).collect::<Vec<_>>();
    let rows = |index: usize| draws.iter().map(|d| {
        let a = d.draw.arms.iter().find(|a| a.arm == ARMS[index]).unwrap();
        a.refused.is_none().then(|| a.readings.clone())
    }).collect::<Vec<_>>();
    let settings = Settings {
        alpha: 0.05, permutations: o.permutations,
        seed: cosmograph::draw(o.master, "probe-check", &[]),
    };
    let comparisons = [
        ("exact against crowd", 2, true),
        ("exact against exact (positive control)", 1, false),
        ("exact against averaged crowd (negative control)", 3, false),
        ("exact against unweighted crowd (draw control)", 5, false),
    ].into_iter().map(|(name, right, bounded)|
        check::compare_runs(name, &bounds, &rows(0), &rows(right), settings, bounded))
        .collect::<Vec<_>>();
    let starvation = infos.iter().filter(|r| r.starvation).map(|r| r.key.clone()).collect();
    let verdicts = Verdicts::new(&comparisons, starvation);
    let mut value = serde_json::json!({ "draws": draws, "comparisons": comparisons, "verdicts": verdicts });
    // Arm wall times are the only nondeterministic fields in these raw draws.
    for draw in value["draws"].as_array_mut().unwrap() {
        for arm in draw["draw"]["arms"].as_array_mut().unwrap() {
            arm.as_object_mut().unwrap().remove("micros");
        }
    }
    value
}

#[test]
fn sequential_and_parallel_draws_preserve_raw_readings_and_controls() {
    // Small worlds exercise the real harm arms and per-tick matter auditor;
    // this is a worker equivalence control, not a statistical certificate.
    let domain = small_domain();
    let arms = [0, 1, 2, 3, 5];
    let mut sequential = Vec::new();
    let mut parallel = Vec::new();
    let o = options(1);
    run(&o, &domain, &arms, |d| { sequential.push(d); Ok(()) }).unwrap();
    let parallel_options = options(4);
    run(&parallel_options, &domain, &arms, |d| { parallel.push(d); Ok(()) }).unwrap();
    assert_eq!(parallel.iter().map(|d| d.draw.k).collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
    for (k, d) in parallel.iter().enumerate() {
        assert_eq!(d.draw.seed, cosmograph::draw(o.master, "probe-world", &[k as u64]));
        for (&index, arm) in arms.iter().zip(&d.draw.arms) {
            assert_eq!(arm.dynamics, cosmograph::draw(o.master, "probe-dynamics", &[k as u64, index as u64]));
        }
        assert_eq!(d.checks.is_some(), k == 0, "only draw zero runs the sanity controls");
    }
    let checks = parallel[0].checks.as_ref().unwrap();
    assert!(checks.exact_rerun_identical && checks.collect_changes_no_outcome);
    assert_eq!(canonical(&o, &sequential), canonical(&parallel_options, &parallel),
        "worker count must not change raw native readings, refusals, matter, seeds or control verdicts");
}

#[test]
fn an_emit_error_drains_the_real_native_draw_pool() {
    let o = options(4);
    let domain = small_domain();
    assert_eq!(Execution::new(o.workers, o.draws).workers, 4);
    let mut delivered = Vec::new();
    let result = run(&o, &domain, &[0, 1, 2, 3, 5], |draw| {
        delivered.push(draw.draw.k);
        if draw.draw.k == 1 {
            // Draw zero has finished its native sanity runs; all four pool
            // jobs have been submitted before this first parallel delivery.
            return Err("deliberate probe emit error".into());
        }
        Ok(())
    });
    // Returning from run crosses thread::scope's join boundary, including
    // workers blocked on either channel. The stopped consumer sees no tail.
    assert_eq!(result, Err("deliberate probe emit error".into()));
    assert_eq!(delivered, [0, 1]);
    drop(domain);
}

#[test]
fn an_emit_panic_unwinds_and_drains_the_real_native_draw_pool() {
    let o = options(4);
    let domain = small_domain();
    assert_eq!(Execution::new(o.workers, o.draws).workers, 4);
    let mut delivered = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&o, &domain, &[0, 1, 2, 3, 5], |draw| {
            delivered.push(draw.draw.k);
            if draw.draw.k == 1 { panic!("deliberate probe emit panic"); }
            Ok(())
        })
    }));
    // The catcher is outside the scope, so it can receive this exact panic
    // only after its native workers have drained and joined during unwind.
    let panic = result.expect_err("the consumer panic must leave the pool");
    let message = panic.downcast_ref::<&str>().copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str));
    assert_eq!(message, Some("deliberate probe emit panic"));
    assert_eq!(delivered, [0, 1]);
    drop(domain);
}

#[test]
fn workers_are_bounded_and_do_not_create_empty_draw_jobs() {
    assert!(requested(0).is_err());
    assert!(requested(65).is_err());
    assert_eq!(requested(1).unwrap(), 1);
    assert_eq!(requested(64).unwrap(), 64);
    assert_eq!(Execution::new(4, 1).workers, 1);
    assert_eq!(Execution::new(4, 3).workers, 2);
    let mut o = options(4);
    o.draws = 0;
    run(&o, &Domain::Probe(Box::default()), &[0], |_| panic!("no draw exists")).unwrap();
}
