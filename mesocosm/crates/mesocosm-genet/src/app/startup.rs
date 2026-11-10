// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ordinary explicit start/view choices over existing native founding law.

use super::{HostConfig, Runtime};
use mesocosm_runtime::continuation::Rounds;

#[cfg(test)]
fn a1_setup(founded: mesocosm_runtime::Founded,
    affinity: Option<cosmograph::rules::Affinity>)
    -> Result<mesocosm_runtime::runtime::ecology::Setup, mesocosm_runtime::runtime::ecology::GeneratedSetupError> {
    a1_setup_with(founded, affinity, super::config::WorldRules::default())
}

/// `a1_setup` with the chosen world rules set in the regenerated genesis,
/// before the setup classifies it (lineage-rounds D2, D9, D12).
fn a1_setup_with(mut founded: mesocosm_runtime::Founded,
    affinity: Option<cosmograph::rules::Affinity>, rules: super::config::WorldRules)
    -> Result<mesocosm_runtime::runtime::ecology::Setup, mesocosm_runtime::runtime::ecology::GeneratedSetupError> {
    if affinity.is_some() {
        return Err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid("--ecology-rates a1 cannot be combined with the authored pack affinity; remove that affinity before selecting A1".into()));
    }
    let mut founding = founded.genesis.founding.clone()
        .ok_or_else(|| mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid("--ecology-rates a1 requires an ordinary generated founding".into()))?;
    founding.ecology_rates = cosmograph::ecology::Rates::A1;
    if rules.mass_law { founding.mass_law = Some(cosmograph::ecology::MassLaw::default()); }
    founded.genesis = founding.generate().map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
    rules.apply(&mut founded.genesis).map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
    mesocosm_runtime::runtime::ecology::Setup::generated_classified(
        founded, cosmograph::Execution::Individuals)
}

fn candidate_seed(base: u64, attempt: u32) -> u64 {
    let mut value = base.wrapping_add(u64::from(attempt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[derive(Debug)]
struct SelectedA1<T> { setup: T, seed: u64, attempts: u32 }

fn select_a1_candidate<T>(
    base_seed: u64,
    attempt_cap: u32,
    permit_search: bool,
    population: u64,
    mut prepare: impl FnMut(u64) -> Result<T,
        mesocosm_runtime::runtime::ecology::GeneratedSetupError>,
) -> Result<SelectedA1<T>, mesocosm_runtime::runtime::ecology::GeneratedSetupError> {
    use mesocosm_runtime::runtime::ecology::GeneratedSetupError as CandidateError;
    if permit_search && !(1..=128).contains(&attempt_cap) {
        return Err(CandidateError::Invalid("A1 candidate cap must be from 1 to 128".into()));
    }
    let attempts = if permit_search { attempt_cap } else { 1 };
    for index in 0..attempts {
        let seed = if index == 0 { base_seed } else { candidate_seed(base_seed, index) };
        eprintln!("A1 candidate {}/{attempts}: seed {seed}, population {population}.", index + 1);
        match prepare(seed) {
            Ok(setup) => return Ok(SelectedA1 { setup, seed, attempts: index + 1 }),
            Err(CandidateError::MissingViableConsumerFood) => {
                if index + 1 < attempts {
                    eprintln!("A1 candidate seed {seed} lacks a viable consumer and food target; trying the next candidate.");
                } else {
                    eprintln!("A1 candidate seed {seed} lacks a viable consumer and food target; candidate limit reached.");
                }
            }
            Err(CandidateError::MissingPlayedPlacement) => {
                if !permit_search { return Err(CandidateError::MissingPlayedPlacement); }
                eprintln!("A1 candidate seed {seed} has no declared stand for its controlled body.");
            }
            Err(error @ CandidateError::Invalid(_)) => return Err(error),
        }
    }
    Err(CandidateError::Invalid(format!(
        "no A1 candidate met the requested food/placement conditions after {attempts} candidate(s)")))
}

/// A generated world; with `--hosting`, its microbe is seeded before play.
pub(super) fn generated(config: &HostConfig, affinity: Option<cosmograph::rules::Affinity>) -> Result<Runtime, String> {
    let mut runtime = generated_world(config, affinity)?;
    if config.world_rules.hosting { runtime.seed_microbe()?; }
    Ok(runtime)
}

fn generated_world(config: &HostConfig, affinity: Option<cosmograph::rules::Affinity>) -> Result<Runtime, String> {
    if config.ecology_rates.is_some()
        && (config.bench_world || config.ecology_candidate.is_some()
            || config.watch.is_some()
            || config.spread.is_some() || config.spatial.is_some() || config.graft.is_some()) {
        return Err("--ecology-rates requires an ordinary generated world without a bench, candidate, resume, spread, spatial or graft input".into());
    }
    if config.play {
        return play(config, affinity);
    }
    if config.bench_world {
        let mut founded = mesocosm_runtime::Founded::generated_with_affinity(config.seed, config.population, affinity)?;
        founded.genesis.rules.epoch_ticks = config.bench_epoch;
        if let Some(start) = config.start { founded.start = start; }
        if let Some(mode) = config.mode { founded.mode = mode; }
        config.world_rules.apply(&mut founded.genesis)?;
        return Runtime::bench(founded, config.ticks_per_second);
    }
    if let Some(path) = &config.ecology_candidate {
        let setup = read_setup(path)?;
        return setup.offer(config.ticks_per_second);
    }
    let seed = if config.unchosen_seed {
        // Same clock-sourced seed as native cosmograph-bench. This is not OS randomness.
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|why| why.to_string())?.as_nanos() as u64
    } else { config.seed };
    if config.ecology_witness.is_none()
        && config.ordinary_ecology_rates() == cosmograph::ecology::Rates::A1 {
        let began = std::time::Instant::now();
        let selected = select_a1_candidate(seed, config.candidate_attempts, config.unchosen_seed, config.population,
            |candidate| {
            let mut founded = mesocosm_runtime::Founded::generated_with_affinity(candidate, config.population, affinity.clone())
                .map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
            if let Some(start) = config.start { founded.start = start; }
            if let Some(mode) = config.mode { founded.mode = mode; }
            a1_setup_with(founded, affinity.clone(), config.world_rules)
        }).map_err(|error| match error {
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(message) => message,
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingViableConsumerFood =>
                "candidate search ended unexpectedly".into(),
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingPlayedPlacement =>
                "candidate has no declared physical stand".into(),
        })?;
        eprintln!("A1 preflight accepted seed {} after {} candidate(s); running full ecology witness.", selected.seed, selected.attempts);
        let runtime = selected.setup.offer_with_progress(config.ticks_per_second,
            &mut |tick, horizon| {
                if tick % 10 == 0 || tick == horizon {
                    eprintln!("A1 ecology witness: live tick {tick}/{horizon} completed.");
                }
                if tick == horizon { eprintln!("A1 ecology witness: validating native history."); }
            })?;
        eprintln!("A1 ecology witness accepted in {:.3}s.", began.elapsed().as_secs_f64());
        return Ok(runtime);
    }
    let mut founded = mesocosm_runtime::Founded::generated_with_affinity(seed, config.population, affinity.clone())?;
    if let Some(start) = config.start { founded.start = start; }
    if let Some(mode) = config.mode { founded.mode = mode; }
    let path = config.ecology_witness.as_ref()
        .ok_or("new generated world requires --ecology-witness with complete authored witness inputs")?;
    let setup = read_setup(path)?;
    if let Some(rates) = config.ecology_rates {
        let mut founding = founded.genesis.founding.clone()
            .ok_or("--ecology-rates requires a generated founding")?;
        founding.ecology_rates = rates;
        founded.genesis = founding.generate()?;
    }
    // This explicit declaration is installed before the candidate's native Session exists.
    founded.genesis.rules.ecology_witness = Some(setup.specification.clone());
    if setup.founded != founded {
        return Err("ecology witness names another generated candidate, seed, or lifecycle configuration".into());
    }
    setup.offer(config.ticks_per_second)
}

/// A world to play rather than to admit: the A1 candidate search keeps its
/// cheap structural check (a living consumer with food), the full ecology
/// witness is skipped, and the epoch is short enough to reach the board.
fn play(config: &HostConfig, affinity: Option<cosmograph::rules::Affinity>) -> Result<Runtime, String> {
    let seed = if config.unchosen_seed {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|why| why.to_string())?.as_nanos() as u64
    } else { config.seed };
    // A1 was qualified on 60 founders; the generator's default of 256 booms
    // past a thousand within an epoch. An explicit --population still wins.
    let population = if config.population == cosmograph::Founding::default().population { 60 } else { config.population };
    let selected = select_a1_candidate(seed, config.candidate_attempts, config.unchosen_seed, population,
        |candidate| {
            let mut founded = mesocosm_runtime::Founded::generated_with_affinity(candidate, population, affinity.clone())
                .map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
            if let Some(start) = config.start { founded.start = start; }
            if let Some(mode) = config.mode { founded.mode = mode; }
            let bodies = founded.genesis.founding.as_mut().and_then(|f| f.bodies.as_mut())
                .ok_or_else(|| mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(
                    "body roster needs a bodied founding".into()))?;
            if config.body_roster && config.body_founding.is_none() {
                bodies.roster = true;
                bodies.founding = None;
            } else {
                let selection = config.body_founding.as_deref().unwrap_or("SpacedRoster");
                bodies.roster = selection != "Drawn";
                bodies.founding = (selection != "Drawn").then(|| selection.to_string());
            }
            bodies.proportions = config.body_proportions.unwrap_or(
                !config.body_roster && config.body_founding.is_none());
            bodies.limb_scale = if bodies.proportions { [1,2] } else { [0,0] };
            let mut founded = a1_setup_with(founded, affinity.clone(), config.world_rules)?.founded;
            founded.genesis.rules.epoch_ticks = config.play_epoch;
            for site in founded.genesis.sites.values_mut() {
                if let Some(soil) = site.accounts.get_mut("world:soil") { *soil = *soil * config.play_soil / 100; }
            }
            let sun = founded.genesis.rules.sun.clone().unwrap_or_default();
            founded.genesis.rules.sun = Some(cosmograph::light::SunRules { start_hour: config.play_hour, ..sun });
            if config.habitat_site {
                cosmograph::spatial::embodiment::declare_site(&mut founded.genesis)
                    .map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
                preflight_site_starter(&founded)?;
            }
            let runtime = Runtime::bench_placed(founded, config.ticks_per_second)
                .map_err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid)?;
            if runtime.critter().is_none_or(|id| !runtime.spatial().members.contains_key(&id)) {
                return Err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingPlayedPlacement);
            }
            if !placed_food_near_starter(&runtime) {
                return Err(mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingViableConsumerFood);
            }
            Ok(runtime)
        }).map_err(|error| match error {
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::Invalid(message) => message,
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingViableConsumerFood =>
                "no candidate had a living consumer with food".into(),
            mesocosm_runtime::runtime::ecology::GeneratedSetupError::MissingPlayedPlacement =>
                "the controlled body has no generator-declared stand; choose another world seed".into(),
        })?;
    eprintln!("play: seed {}, epoch {} ticks, {} ticks a second, starting at {}:00, soil {}%; no ecology witness.",
        selected.seed, config.play_epoch, config.ticks_per_second, config.play_hour, config.play_soil);
    // One tick at a time, on a worker: a world too big for the clock falls
    // behind it while the window keeps drawing the last tick.
    Ok(selected.setup.with_max_steps(1).with_background())
}

/// Structural diet compatibility alone can put food in a different site.
/// New playable worlds also need an actual nearby meal, using native queries
/// and native occupied geometry. This does not require a walk or choose one.
fn placed_food_near_starter(runtime: &Runtime) -> bool {
    let Some(id) = runtime.critter() else { return false; };
    let sim = runtime.sim();
    let game = runtime.spatial();
    let Some(actor) = game.members.get(&id) else { return false; };
    starter_food_near(&sim,id,actor.site.0,|food| game.members.get(&food)
        .filter(|at| at.site==actor.site).map(|at|at.pose.clone()))
}

fn starter_food_near(sim:&cosmograph::Simulation,id:u64,site:u64,
    mut pose:impl FnMut(u64)->Option<cosmograph::spatial::Pose>) -> bool {
    use cosmograph::spatial::{embodiment, foraging};
    let Some(at)=pose(id) else {return false};
    let Some(doc) = sim.state().population.get(id).and_then(|e|e.body.as_ref()) else { return false; };
    let Ok(occupied) = embodiment::boxes(doc, &at) else { return false; };
    sim.placement_food(id, site).into_iter().any(|(food, range)| {
        let Some(at) = pose(food) else { return false; };
        let Some(doc) = sim.state().population.get(food).and_then(|e|e.body.as_ref()) else { return false; };
        embodiment::boxes(doc, &at).is_ok_and(|boxes|foraging::gap(&occupied, &boxes) <= range)
    })
}

/// Reject the same unsupported starts before deriving every resident's places.
/// The native interim still chooses the controller; no walking outcome is tested.
fn preflight_site_starter(founded:&mesocosm_runtime::Founded)
    -> Result<(),mesocosm_runtime::runtime::ecology::GeneratedSetupError> {
    use mesocosm_runtime::runtime::ecology::GeneratedSetupError as Error;
    if founded.start!=Default::default() || founded.genesis.rules.deep_time.epochs!=0 {return Ok(());}
    let session=cosmograph::Session::new(founded.genesis.clone(),cosmograph::Execution::Grouped).map_err(Error::Invalid)?;
    let interim=cosmograph::directing::interim::Interim::found_session(session,
        founded.start,founded.mode,founded.pace).map_err(Error::Invalid)?;
    let actor=interim.critter().ok_or(Error::MissingPlayedPlacement)?;
    let sim=&interim.session.sim;
    let site=sim.state().population.get(actor).and_then(|e|e.resident_site()).ok_or(Error::MissingPlayedPlacement)?;
    let t=sim.genesis().rules.spatial_templates.as_ref().ok_or(Error::MissingPlayedPlacement)?;
    let poses:std::collections::BTreeMap<_,_>=t.stands.get(&site).into_iter().flatten()
        .filter_map(|s|Some((s.actor?,cosmograph::spatial::Pose {at:s.at,yaw:s.yaw,feet:s.feet.clone()}))).collect();
    if !poses.contains_key(&actor) {return Err(Error::MissingPlayedPlacement);}
    if !starter_food_near(sim,actor,site,|id|poses.get(&id).cloned()) {return Err(Error::MissingViableConsumerFood);}
    Ok(())
}

fn read_setup(path: &std::path::Path) -> Result<mesocosm_runtime::runtime::ecology::Setup, String> {
    let bytes = std::fs::read(path).map_err(|why| why.to_string())?;
    serde_json::from_slice(&bytes).map_err(|why| why.to_string())
}

#[cfg(test)]
mod tests {
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

}

/// One start/resume path for the headed host and headless sampling.
pub(super) fn configured(config: &HostConfig, authored: &mesocosm_runtime::Authored) -> Result<Runtime,String> {
    if config.watch.is_some() && (config.start.is_some() || config.unchosen_seed) {
        return Err("a resumed run already has its native founding and start".into());
    }
    if config.spread.is_some() && (config.start.is_some() || config.mode.is_some() || config.unchosen_seed) {
        return Err("authored spread supplies its own seed, start and viewing mode".into());
    }
    if let Some(why) = config.round_flags_refused(false, false) { return Err(why.into()); }
    let mut runtime = match (&config.watch, &config.spread) {
            (Some(_), Some(_)) => {
                Err("authored spread founding cannot replace a resumed run".into())
            },
            (_, Some(path)) => std::fs::read(path)
                .map_err(|why| why.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<mesocosm_runtime::author_spread::Setup>(&bytes)
                        .map_err(|why| why.to_string())
                })
                .and_then(|setup| Runtime::from_authored_spread(&setup, config.ticks_per_second)),
            (Some(path), None) => {
                crate::played::resume_with_mode(path, config.ticks_per_second, config.mode)
            },
            (None, None) => generated(config, authored.affinity().cloned()),
        }?;
    if config.rounds != Rounds::Capped || config.condition { runtime.set_rounds(config.rounds, config.condition); }
    Ok(runtime)
}
