// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Tests of the start paths in `startup.rs`.

use super::*;

/// An A1 candidate founded without producers, which setup refuses:
/// under case 1106's floor every generated candidate is otherwise viable.
fn barren(seed: u64) -> Result<mesocosm_runtime::runtime::ecology::Setup, mesocosm_runtime::runtime::ecology::GeneratedSetupError> {
    let mut founded = mesocosm_runtime::Founded::generated(seed, 256).unwrap();
    let mut founding = founded.genesis.founding.clone().unwrap();
    founding.ecology_rates = cosmograph::ecology::Rates::A1;
    founded.genesis = founding.generate().unwrap();
    for group in founded.genesis.population.groups.values_mut() {
        if group.entity.traits.contains("life:producer") {
            std::sync::Arc::make_mut(&mut group.entity).alive = false;
        }
    }
    mesocosm_runtime::runtime::ecology::Setup::generated_classified(founded, cosmograph::Execution::Individuals)
}

#[test]
fn explicit_a1_regenerates_an_unmarked_foundation_before_setup_authors_it() {
    let founded = mesocosm_runtime::Founded::generated(9, 256).unwrap();
    let setup = a1_setup(founded, None).unwrap();
    assert_eq!(setup.founded.genesis.founding.as_ref().unwrap().ecology_rates,
        cosmograph::ecology::Rates::A1);
    assert_eq!(setup.specification.horizon, 60);
    assert_eq!(setup.founded.genesis.rules.ecology_witness.as_ref(), Some(&setup.specification));
    assert_eq!(setup.execution, cosmograph::Execution::Individuals);
}

#[test]
fn world_rules_reach_the_founded_genesis_and_default_leaves_it_alone() {
    let found = |world_rules| generated(&HostConfig { bench_world: true, seed: 9, population: 256, world_rules,
        ..Default::default() }, None).unwrap().founded().genesis.rules.directing();
    let plain = found(super::super::config::WorldRules::default());
    assert!(!plain.drift && plain.draft.is_none() && !plain.sheet);
    let all = found(super::super::config::WorldRules::all());
    assert!(all.drift && all.sheet);
    assert_eq!(all.draft, Some(cosmograph::rules::Draft::default()));
    let setup = a1_setup_with(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None,
        super::super::config::WorldRules::all()).unwrap();
    assert!(setup.founded.genesis.rules.directing().sheet, "set after the A1 regeneration");
}

#[test]
fn hosting_seeds_a_microbe_before_play_and_leaves_the_world_matter_alone() {
    let found = |hosting| generated(&HostConfig { bench_world: true, seed: 9, population: 256,
        world_rules: super::super::config::WorldRules { hosting, ..Default::default() },
        ..Default::default() }, None).unwrap();
    let (plain, hosted) = (found(false), found(true));
    assert_eq!(plain.sim().matter(), hosted.sim().matter());
    assert!(!plain.sim().is_surface(mesocosm_runtime::microbe::KEY));
    assert!(hosted.sim().is_surface(mesocosm_runtime::microbe::KEY));
    let line = cosmograph::directing::readings::held(&hosted.sim(), mesocosm_runtime::microbe::KEY).unwrap();
    assert_eq!(line.sites.total, mesocosm_runtime::microbe::SEED_AMOUNT);
}

#[test]
fn a_learning_world_founds_its_policies_runs_and_a_member_learns() {
    // The A1 world, as the window founds it, run as a session for sixty
    // ticks: the round's questions are the runtime's, and a member's
    // learning is the sim's.
    let run = |learning| {
        let rules = super::super::config::WorldRules { learning, ..Default::default() };
        let setup = a1_setup_with(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None, rules).unwrap();
        let mut session = cosmograph::Session::new(setup.founded.genesis, cosmograph::Execution::Individuals).unwrap();
        session.advance(60).unwrap();
        session
    };
    let (learning, plain) = (run(true), run(false));
    let learned = |s: &cosmograph::Session| s.sim.state().population.groups.values()
        .filter(|g| g.entity.alive && g.entity.learned.is_some()).count();
    assert_eq!(learning.sim.state().tick, 60);
    assert!(learned(&learning) > 0, "someone learned in sixty ticks");
    assert_eq!(learned(&plain), 0, "no flag, no learning");
    assert!(plain.sim.state().lineages.values().all(|l| l.policy.is_none()));
    assert!(learning.sim.state().lineages.values().any(|l| l.policy.is_some()));
}

#[test]
fn founding_a_policy_alone_changes_how_nobody_chooses() {
    // The founding policy scores as the hand-written score does, so
    // with the learning rule taken off again the world runs as it did
    // (measured the same over 200 ticks of seeds 9 and 3); learning is
    // what makes the run differ.
    let run = |learning: bool, keep: bool| {
        let rules = super::super::config::WorldRules { learning, ..Default::default() };
        let setup = a1_setup_with(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None, rules).unwrap();
        let mut genesis = setup.founded.genesis;
        if !keep { genesis.rules.directing.as_mut().unwrap().learning = None; }
        let mut session = cosmograph::Session::new(genesis, cosmograph::Execution::Individuals).unwrap();
        session.advance(60).unwrap();
        (session.sim.state().population.groups.clone(), session.sim.state().relations.clone())
    };
    let (plain, policy, full) = (run(false, false), run(true, false), run(true, true));
    assert!(plain == policy, "a founded policy and no learning chooses as before");
    assert!(plain != full, "learning changes the run");
}

#[test]
fn mass_law_sets_the_founding_rule_and_default_flags_leave_the_genesis_alone() {
    let setup = |mass_law| a1_setup_with(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None,
        super::super::config::WorldRules { mass_law, ..Default::default() }).unwrap().founded.genesis;
    let (off, on) = (setup(false), setup(true));
    assert!(off.founding.as_ref().unwrap().mass_law.is_none());
    assert_eq!(cosmograph::digest(&off), cosmograph::digest(&setup(false)));
    let law = on.founding.as_ref().unwrap().mass_law.clone().expect("the flag sets the rule");
    assert!(law.reference.is_some_and(|r| r > 0), "the reference is stated at founding");
    assert_ne!(cosmograph::digest(&off), cosmograph::digest(&on));
    assert!(!serde_json::to_string(&off).unwrap().contains("mass_law"));
    assert!(super::super::config::WorldRules { mass_law: true, ..Default::default() }
        .apply(&mut off.clone()).is_err(), "a world founded without it refuses the late flag");
}

#[test]
fn dormancy_sets_the_founding_rule_and_default_flags_leave_the_genesis_alone() {
    let setup = |dormancy| a1_setup_with(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None,
        super::super::config::WorldRules { dormancy, ..Default::default() }).unwrap().founded.genesis;
    let (off, on) = (setup(false), setup(true));
    assert!(off.founding.as_ref().unwrap().dormancy.is_none() && off.rules.dormancy.is_none());
    assert_eq!(cosmograph::digest(&off), cosmograph::digest(&setup(false)));
    assert_eq!(cosmograph::digest(&off), cosmograph::digest(&setup_plain()), "default flags found as they did");
    let rule = on.rules.dormancy.clone().expect("the flag sets the rule through the A1 regeneration");
    assert_eq!(on.founding.as_ref().unwrap().dormancy.as_ref().map(|d| d.season_ticks), Some(Some(40)));
    assert!(!rule.seeds.is_empty(), "the producers' births seed");
    assert_ne!(cosmograph::digest(&off), cosmograph::digest(&on));
    let json = serde_json::to_string(&off).unwrap();
    assert!(!json.contains("dormancy") && !json.contains("ecology:wake"), "an unset world never mentions it");
    assert!(super::super::config::WorldRules { dormancy: true, ..Default::default() }
        .apply(&mut off.clone()).is_err(), "a world founded without it refuses the late flag");
}

fn setup_plain() -> cosmograph::simulation::Genesis {
    a1_setup(mesocosm_runtime::Founded::generated(9, 256).unwrap(), None).unwrap().founded.genesis
}

#[test]
fn explicit_a1_reports_authored_pack_affinity_instead_of_stripping_it() {
    let founded = mesocosm_runtime::Founded::generated(9, 256).unwrap();
    let error = a1_setup(founded, Some(cosmograph::rules::Affinity::default())).unwrap_err();
    assert!(matches!(error, mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(message)
        if message.contains("authored pack affinity")));
}

#[test]
fn invalid_search_bounds_refuse_before_generation() {
    for cap in [0, 129, u32::MAX] {
        assert!(select_a1_candidate::<mesocosm_runtime::runtime::ecology::Setup>(9, cap, true, 256,
            |_| panic!("invalid cap must refuse before generation")).is_err());
    }
}

#[test]
fn explicit_bad_seed_stops_after_one_candidate_even_with_search_cap() {
    let mut tried = Vec::new();
    let error = select_a1_candidate(1_792_230_697_480_612_000, 12, false, 256, |seed| {
        tried.push(seed);
        barren(seed)
    }).unwrap_err();
    assert_eq!(tried, [1_792_230_697_480_612_000]);
    assert!(matches!(error, mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(_)));
}

#[test]
fn unchosen_search_selects_first_viable_derived_candidate_and_cap_one_exhausts() {
    let base = 1_792_230_697_480_612_000;
    let mut tried = Vec::new();
    // The first two candidates are barren, so the search must go on.
    let selected = select_a1_candidate(base, 12, true, 256, |seed| {
        tried.push(seed);
        if tried.len() <= 2 { barren(seed) }
        else { a1_setup(mesocosm_runtime::Founded::generated(seed, 256).unwrap(), None) }
    }).unwrap();
    assert!(tried.len() > 1);
    assert_eq!(selected.seed, tried[tried.len() - 1]);
    assert_eq!(selected.seed, candidate_seed(base, selected.attempts - 1));
    assert_ne!(selected.seed, base);

    let mut capped = Vec::new();
    let error = select_a1_candidate(base, 1, true, 256, |seed| {
        capped.push(seed);
        barren(seed)
    }).unwrap_err();
    assert_eq!(capped, [base]);
    assert!(matches!(error, mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(_)));
}

#[test]
fn authored_affinity_is_terminal_and_never_hidden_by_retry() {
    let mut tried = Vec::new();
    let error = select_a1_candidate(1_792_230_697_480_612_000, 12, true, 256, |seed| {
        tried.push(seed);
        a1_setup(mesocosm_runtime::Founded::generated(seed, 256).unwrap(),
            Some(cosmograph::rules::Affinity::default()))
    }).unwrap_err();
    assert_eq!(tried, [1_792_230_697_480_612_000]);
    assert!(matches!(error, mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(message)
        if message.contains("authored pack affinity")));
}

#[test]
fn only_unchosen_search_can_skip_a_missing_physical_stand() {
    use mesocosm_runtime::runtime::ecology::GeneratedSetupError as Error;
    let mut attempts = 0;
    let explicit = select_a1_candidate::<()>(7, 8, false, 60, |_| {
        attempts += 1;
        Err(Error::MissingPlayedPlacement)
    });
    assert!(matches!(explicit, Err(Error::MissingPlayedPlacement)));
    assert_eq!(attempts, 1);
    let mut attempts = 0;
    let selected = select_a1_candidate(7, 8, true, 60, |_| {
        attempts += 1;
        if attempts == 1 { Err(Error::MissingPlayedPlacement) } else { Ok(()) }
    }).unwrap();
    assert_eq!(selected.attempts, 2);
    assert_eq!(selected.seed, candidate_seed(7, 1));
}

#[test]
fn structurally_edible_food_in_another_site_is_not_a_playable_meal() {
    // Second headed regression: the old preflight admitted this world,
    // then its controlled consumer migrated immediately without walking.
    let mut founded = mesocosm_runtime::Founded::generated(15_553_628_298_003_140_480, 60).unwrap();
    let b = founded.genesis.founding.as_mut().unwrap().bodies.as_mut().unwrap();
    b.roster = true;
    b.founding = Some("SpacedRoster".into());
    let founded = a1_setup(founded, None).unwrap().founded;
    let runtime = Runtime::bench_placed(founded, 2).unwrap();
    assert!(!placed_food_near_starter(&runtime));
}
#[test]
fn default_search_recovers_the_observed_eight_candidate_exhaustion() {
    use mesocosm_runtime::runtime::ecology::GeneratedSetupError as Error;
    let selected = select_a1_candidate(1_792_522_317_758_004_600,
        HostConfig::default().candidate_attempts, true, 60, |seed| {
            let founded = mesocosm_runtime::Founded::generated(seed, 60).map_err(Error::Invalid)?;
            let founded = a1_setup(founded, None)?.founded;
            let runtime = Runtime::bench_placed(founded, 2).map_err(Error::Invalid)?;
            if runtime.critter().is_none_or(|id| !runtime.spatial().members.contains_key(&id)) {
                return Err(Error::MissingPlayedPlacement);
            }
            Ok(runtime)
        }).unwrap();
    // It took more than eight attempts before case 1106's founding floor; with
    // every lineage founded, the second candidate places the critter.
    assert_eq!(selected.attempts, 2);
    assert!(selected.attempts <= HostConfig::default().candidate_attempts);
}

