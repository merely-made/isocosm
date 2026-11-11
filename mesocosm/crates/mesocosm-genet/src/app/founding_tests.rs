// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Each world-rule flag through the founding path the binary takes: the
//! A1 candidate, its 60-tick joint witness and the native admission
//! (`startup::configured`, as `--curve` and the window both call it).
//!
//! The bench shortcut (`generated()` with `bench_world`) skips the admission,
//! which is how `--hosting` and the rest founded under test and failed in the
//! binary: the admission asked evidence for every rule the flag added. The
//! rules are now laid over the admission's basis as well, so they are the
//! world's baseline (case 875), and these tests found each one for real.
//! The same run from a shell: `mesocosm-genet --curve 120 --ecology-rates a1
//! --seed 9 --hosting` (seeds in the table below).

use super::*;
use mesocosm_runtime::world_rules::WorldRules;

/// Founds with the flags through the real path and runs `steps` demo steps;
/// returns the runtime and the tick reached.
fn found_and_run(rules: WorldRules, seed: u64, steps: u64) -> (Runtime, u64) {
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let config = HostConfig { seed, world_rules: rules, ecology_rates: Some(cosmograph::ecology::Rates::A1),
        ..Default::default() };
    let mut runtime = startup::configured(&config, &authored)
        .unwrap_or_else(|why| panic!("seed {seed} with {rules:?} did not found: {why}"));
    assert!(runtime.spatial().ecology.is_some(), "founded through the ecology admission, not the bench");
    for step in 0..steps { crate::played::demo_step(&mut runtime, step); }
    let tick = runtime.sim().state().tick;
    (runtime, tick)
}

/// A flag founds, and its world runs the 60 witness ticks and 120 steps more.
fn runs_120(rules: WorldRules, seed: u64) -> Runtime {
    let (runtime, tick) = found_and_run(rules, seed, 120);
    assert!(tick >= 180, "{rules:?} on seed {seed} stopped at tick {tick}");
    runtime
}

#[test] fn regard_founds_and_runs() { runs_120(WorldRules { regard: true, ..Default::default() }, 9); }
#[test] fn exertion_founds_and_runs() { runs_120(WorldRules { exertion: true, ..Default::default() }, 3); }
#[test] fn drift_founds_and_runs() { runs_120(WorldRules { drift: true, ..Default::default() }, 9); }
#[test] fn draft_founds_and_runs() { runs_120(WorldRules { draft: true, ..Default::default() }, 9); }
#[test] fn sheet_founds_and_runs() { runs_120(WorldRules { sheet: true, ..Default::default() }, 9); }
#[test] fn the_rounds_rules_found_and_run() { runs_120(WorldRules::all(), 9); }
#[test] fn learning_founds_and_runs() { runs_120(WorldRules { learning: true, ..Default::default() }, 9); }
#[test] fn mass_law_founds_and_runs() { runs_120(WorldRules { mass_law: true, ..Default::default() }, 9); }
#[test] fn dormancy_founds_and_runs() { runs_120(WorldRules { dormancy: true, ..Default::default() }, 9); }

#[test]
fn hosting_founds_with_its_microbe_and_runs() {
    let runtime = runs_120(WorldRules { hosting: true, ..Default::default() }, 9);
    assert!(runtime.sim().is_surface(mesocosm_runtime::microbe::KEY));
}

/// The default climate kills most A1 lineages inside the witness's 60 ticks,
/// so few seeds found at all (3 of 120 tried), and the played line there dies
/// out by about tick 80 to 120; the tests found and run it as far as it goes.
#[test]
fn the_climate_founds_where_its_witness_passes_and_runs_while_the_line_lives() {
    for (turn, seed) in [(None, 95), (Some(5), 95)] {
        let (runtime, tick) = found_and_run(WorldRules { climate: true, turn, ..Default::default() }, seed, 60);
        assert!(tick > 60, "the admitted world runs on from the witness's tick");
        assert!(runtime.sim().genesis().rules.climate.is_some());
        assert_eq!(runtime.sim().genesis().rules.climate.as_ref().and_then(|c| c.turn), turn);
    }
}

/// Every rule but the climate, which kills the line too soon to run 120 steps.
fn all_rules() -> WorldRules {
    WorldRules { regard: true, exertion: true, hosting: true, learning: true, mass_law: true, dormancy: true,
        ..WorldRules::all() }
}

#[test]
fn every_flag_but_the_climate_founds_and_runs() { runs_120(all_rules(), 5); }

#[test]
fn every_flag_with_the_climate_founds_and_runs_while_the_line_lives() {
    let rules = WorldRules { climate: true, turn: Some(5), ..all_rules() };
    let (_, tick) = found_and_run(rules, 76, 20);
    assert!(tick > 60);
}

/// The admission still refuses a rule it was not told about: the check was
/// not loosened, the flag's rules were declared to it.
#[test]
fn an_undeclared_rule_still_fails_the_extra_noun_evidence() {
    let mut founded = mesocosm_runtime::Founded::generated(9, 256).unwrap();
    founded.genesis.founding.as_mut().unwrap().ecology_rates = cosmograph::ecology::Rates::A1;
    let rules = WorldRules { hosting: true, ..Default::default() };
    let mut setup = startup::a1_setup_with_for_tests(founded, rules).unwrap();
    assert!(!setup.rules.is_off());
    setup.rules = WorldRules::default();
    let why = setup.offer(10).err().expect("an undeclared hosting rule is refused");
    assert!(why.contains("extra-noun evidence"), "{why}");
}
