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

#[test] fn regard_founds_and_runs() { runs_120(WorldRules { regard: true, ..WorldRules::generated() }, 9); }
#[test] fn exertion_founds_and_runs() { runs_120(WorldRules { exertion: true, ..WorldRules::generated() }, 3); }
#[test] fn drift_founds_and_runs() { runs_120(WorldRules { drift: true, ..WorldRules::generated() }, 9); }
#[test] fn draft_founds_and_runs() { runs_120(WorldRules { draft: true, ..WorldRules::generated() }, 9); }
#[test] fn sheet_founds_and_runs() { runs_120(WorldRules { sheet: true, ..WorldRules::generated() }, 9); }
#[test] fn the_rounds_rules_found_and_run() { runs_120(WorldRules { dormancy: true, ..WorldRules::all() }, 9); }
#[test] fn learning_founds_and_runs() { runs_120(WorldRules { learning: true, ..WorldRules::generated() }, 9); }
#[test] fn mass_law_founds_and_runs() { runs_120(WorldRules { mass_law: true, ..WorldRules::generated() }, 9); }
#[test] fn dormancy_founds_and_runs() { runs_120(WorldRules { dormancy: true, ..WorldRules::generated() }, 9); }

/// A founding rule, so it passes native admission with no declared rules; the
/// flag alone lays the gates on the founded world's consumers, decomposers
/// included (D27). It is on in the generated rules, so this is the default world.
#[test]
fn limits_found_and_run() {
    let runtime = runs_120(WorldRules { limits: true, ..WorldRules::generated() }, 9);
    let sim = runtime.sim();
    let g = sim.genesis();
    assert!(g.founding.as_ref().unwrap().limits.is_some());
    // The generated world founds producer, grazer and decomposer lineages.
    for i in 1..=2 {
        assert!(g.rules.processes[&format!("ecology:feed-{i}")].requires.iter().any(|q| matches!(q, cosmograph::rules::Query::Below { .. })), "feed-{i}");
        assert!(g.rules.processes[&format!("ecology:birth-{i}")].requires.iter().any(|q| matches!(q, cosmograph::rules::Query::MembersBelow { .. })), "birth-{i}");
    }
}

#[test]
fn the_generated_rules_turn_limits_on_and_no_limits_turns_them_off_and_default_stays_nothing() {
    assert!(WorldRules::generated().limits && WorldRules::generated().dormancy);
    assert!(HostConfig::default().world_rules.limits);
    assert!(!WorldRules::default().limits && !WorldRules::all().limits);
    assert!(WorldRules::default().is_off());
    // An older saved rules object, with no `limits`, means off.
    assert!(!serde_json::from_str::<WorldRules>(r#"{"dormancy":true}"#).unwrap().limits);
}

#[test]
fn hosting_founds_with_its_microbe_and_runs() {
    let runtime = runs_120(WorldRules { hosting: true, ..WorldRules::generated() }, 9);
    assert!(runtime.sim().is_surface(mesocosm_runtime::microbe::KEY));
}

/// The climate is tuned to a survival target (ruling D26): of seeds 0 to 39
/// with the defaults on, 39 found and 38 keep the played line 600 ticks. The
/// test founds through the real admission and runs the demo 120 steps past the
/// witness, with and without the world's turn, and the played line is still there.
#[test]
fn the_climate_founds_and_the_played_line_lives_on() {
    for (turn, seed) in [(None, 9), (Some(5), 9)] {
        let (runtime, tick) = found_and_run(WorldRules { climate: true, turn, ..WorldRules::generated() }, seed, 120);
        assert!(tick >= 180, "the admitted world runs on from the witness's tick, stopped at {tick}");
        assert!(runtime.sim().genesis().rules.climate.is_some());
        assert_eq!(runtime.sim().genesis().rules.climate.as_ref().and_then(|c| c.turn), turn);
        let played = runtime.sim().state().population.groups.values()
            .filter(|g| g.entity.alive && g.entity.lineage == "lineage:1").map(|g| g.count).sum::<u64>();
        assert!(played > 0, "the played line is alive at tick {tick}");
    }
}

/// Every rule but the climate.
fn all_rules() -> WorldRules {
    WorldRules { regard: true, exertion: true, hosting: true, learning: true, mass_law: true, dormancy: true, limits: true,
        ..WorldRules::all() }
}

#[test]
fn every_flag_but_the_climate_founds_and_runs() { runs_120(all_rules(), 5); }

// Rerecorded 2026-11-11: seed 76 stopped founding when A1's decomposers took
// their own upkeep, feeding and meal-before-move rates (the witness's 60 ticks
// run differently); seed 3 founds (as do 2, 7, 13, 14, 18, 19, 21, 22, 24 and
// more).
#[test]
fn every_flag_with_the_climate_founds_and_runs() {
    let rules = WorldRules { climate: true, turn: Some(5), ..all_rules() };
    let (_, tick) = found_and_run(rules, 3, 120);
    assert!(tick >= 180);
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

// The crowding brake is the product default (2026-11-11): `HostConfig::default()`
// carries `WorldRules::generated()`, while `WorldRules::default()` stays "nothing
// chosen", which is what an absent field in a saved setup meant.

fn found_default(rules: WorldRules, seed: u64) -> Runtime {
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    startup::configured(&HostConfig { seed, world_rules: rules, ..Default::default() }, &authored)
        .unwrap_or_else(|why| panic!("seed {seed} with {rules:?} did not found: {why}"))
}

fn digest_of(runtime: &Runtime) -> String { cosmograph::digest(&runtime.founded().genesis) }

#[test]
#[cfg_attr(debug_assertions, ignore = "founds dormant worlds through the 60-tick witness; slow in debug, run with --release")]
fn a_new_generated_world_founds_with_the_crowding_brake_by_default() {
    assert!(HostConfig::default().world_rules.dormancy);
    let runtime = found_default(HostConfig::default().world_rules, 9);
    let genesis = &runtime.founded().genesis;
    assert!(genesis.founding.as_ref().unwrap().dormancy.is_some());
    assert!(genesis.rules.dormancy.is_some());
    assert!(runtime.spatial().ecology.is_some(), "founded through the ecology admission");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "founds dormant worlds through the 60-tick witness; slow in debug, run with --release")]
fn no_dormancy_founds_the_world_the_old_default_founded_and_dormancy_is_the_default() {
    let off = found_default(WorldRules { dormancy: false, limits: false, ..HostConfig::default().world_rules }, 9);
    let genesis = &off.founded().genesis;
    assert!(genesis.founding.as_ref().unwrap().dormancy.is_none() && genesis.rules.dormancy.is_none());
    let json = serde_json::to_string(genesis).unwrap();
    assert!(!json.contains("dormancy") && !json.contains("ecology:wake"), "an unset world never mentions it");
    // What the default founded before it changed: A1 over the same seed, no rule chosen.
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let founded = mesocosm_runtime::Founded::generated_with_affinity(9, HostConfig::default().population,
        authored.affinity().cloned()).unwrap();
    let before = startup::a1_setup_with_for_tests(founded, WorldRules::default()).unwrap();
    assert_eq!(digest_of(&off), cosmograph::digest(&before.founded.genesis));
    // `--dormancy` and `--limits` are the default spelled out.
    let on = found_default(WorldRules { dormancy: true, limits: true, ..Default::default() }, 9);
    assert_eq!(digest_of(&on), digest_of(&found_default(HostConfig::default().world_rules, 9)));
    assert_ne!(digest_of(&on), digest_of(&off));
    // `--no-limits` alone is the world the brake default founded (2026-11-11).
    let braked = found_default(WorldRules { limits: false, ..HostConfig::default().world_rules }, 9);
    assert!(braked.founded().genesis.founding.as_ref().unwrap().limits.is_none());
    assert_ne!(digest_of(&braked), digest_of(&on));
}

/// The cheap half of the compatibility pin, which runs in debug too: an older
/// setup file (no `rules`) and an older rules object (no `dormancy`) mean off.
#[test]
fn an_older_setup_and_older_rules_mean_no_brake_and_the_default_is_a_choice_of_the_host() {
    assert_eq!(serde_json::from_str::<WorldRules>("{}").unwrap(), WorldRules::default());
    assert!(!serde_json::from_str::<WorldRules>(r#"{"drift":true}"#).unwrap().dormancy);
    assert!(HostConfig::default().world_rules == WorldRules::generated() && WorldRules::generated().dormancy);
    assert!(!WorldRules::default().dormancy && !WorldRules::all().dormancy);
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let founded = mesocosm_runtime::Founded::generated_with_affinity(9, 256, authored.affinity().cloned()).unwrap();
    let off = startup::a1_setup_with_for_tests(founded.clone(), WorldRules::default()).unwrap();
    let value: serde_json::Value = serde_json::to_value(&off).unwrap();
    assert!(value.get("rules").is_none(), "a setup with no rule chosen saves as older files did");
    let loaded: mesocosm_runtime::runtime::ecology::Setup = serde_json::from_value(value).unwrap();
    assert!(!loaded.rules.dormancy && loaded.founded.genesis.founding.as_ref().unwrap().dormancy.is_none());
    let on = startup::a1_setup_with_for_tests(founded, WorldRules::generated()).unwrap();
    let back: mesocosm_runtime::runtime::ecology::Setup = serde_json::from_str(&serde_json::to_string(&on).unwrap()).unwrap();
    assert!(back.rules.dormancy && back == on);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "founds dormant worlds through the 60-tick witness; slow in debug, run with --release")]
fn a_setup_saved_without_the_brake_still_loads_admits_and_a_world_saved_without_it_carries_on() {
    assert_eq!(serde_json::from_str::<WorldRules>("{}").unwrap(), WorldRules::default());
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let founded = mesocosm_runtime::Founded::generated_with_affinity(9, 256, authored.affinity().cloned()).unwrap();
    let setup = startup::a1_setup_with_for_tests(founded, WorldRules::default()).unwrap();
    let json = serde_json::to_string(&setup).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value.get("rules").is_none(), "a setup with no rule chosen saves as older files did");
    let loaded: mesocosm_runtime::runtime::ecology::Setup = serde_json::from_str(&json).unwrap();
    assert!(!loaded.rules.dormancy && loaded.founded.genesis.founding.as_ref().unwrap().dormancy.is_none());
    let mut runtime = loaded.offer(10).expect("an older setup is admitted without the brake");
    runtime.step(5);
    let saved = runtime.save_world().unwrap();
    let mut resumed = Runtime::resume_world(saved, 10, None).unwrap();
    resumed.step(5);
    assert!(resumed.sim().genesis().founding.as_ref().unwrap().dormancy.is_none());
    // A candidate file for the new default round-trips its rules.
    let on = startup::a1_setup_with_for_tests(
        mesocosm_runtime::Founded::generated_with_affinity(9, 256, authored.affinity().cloned()).unwrap(),
        WorldRules::generated()).unwrap();
    let back: mesocosm_runtime::runtime::ecology::Setup = serde_json::from_str(&serde_json::to_string(&on).unwrap()).unwrap();
    assert!(back.rules.dormancy && back == on);
}

/// How many of seeds 0..120 pass the 60-tick founding witness, by default
/// (the brake on) and with `--no-dormancy`; run explicitly, release:
/// `cargo test --release -p mesocosm-genet --lib witness_census -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement: 240 founding witnesses"]
fn witness_census() {
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let count = |rules: WorldRules| {
        let found = std::sync::atomic::AtomicU64::new(0);
        let next = std::sync::atomic::AtomicU64::new(0);
        std::thread::scope(|scope| for _ in 0..8 { scope.spawn(|| loop {
            let seed = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if seed >= 120 { break; }
            let config = HostConfig { seed, world_rules: rules, ..Default::default() };
            if startup::configured(&config, &authored).is_ok() { found.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        }); });
        found.into_inner()
    };
    let (on, off) = (count(WorldRules::generated()), count(WorldRules::default()));
    eprintln!("CENSUS founded of 120: default (brake, limits) {on}, --no-dormancy --no-limits {off}");
}

/// The played line through the real startup path with default flags, as far
/// as `PLAY_TICKS` (default 600); prints living individuals per lineage.
/// Run explicitly, release, `--ignored --nocapture`.
#[test]
#[ignore = "a measurement: five worlds to tick 600"]
fn the_default_world_lives_to_tick_600() {
    let ticks: u64 = std::env::var("PLAY_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    for seed in [3, 5, 9, 11, 17] {
        let mut runtime = match std::panic::catch_unwind(|| found_default(HostConfig::default().world_rules, seed)) {
            Ok(runtime) => runtime,
            Err(_) => { eprintln!("SURVIVAL seed {seed}: did not found by default"); continue; }
        };
        let mut step = 0;
        while runtime.sim().state().tick < ticks { crate::played::demo_step(&mut runtime, step); step += 1; }
        let mut living: std::collections::BTreeMap<String, u64> = Default::default();
        for group in runtime.sim().state().population.groups.values().filter(|g| g.entity.alive) {
            *living.entry(group.entity.lineage.to_string()).or_default() += group.count;
        }
        eprintln!("SURVIVAL seed {seed}: tick {} living {living:?}", runtime.sim().state().tick);
    }
}
