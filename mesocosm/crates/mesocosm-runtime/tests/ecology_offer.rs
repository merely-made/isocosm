// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Offer/save/replay certification for site-grain fixtures and generated physical worlds.

#[path = "../../../../shared/cosmograph/tests/ecology_independence_879/fixture.rs"]
mod native_fixture;

use cosmograph::{Command, Execution, directing::{self, interim::{OnCollapse, Pace, Start}, readings::Mode}};
use mesocosm_runtime::{Founded, GameSave, Runtime, runtime::ecology::Setup, spatial::ECOLOGY_VERSION};

fn setup(execution: Execution) -> Setup {
    let candidate = native_fixture::candidate_with_played("test:offered-cycle", 1);
    Setup { version: 1,
        founded: Founded { genesis: candidate.genesis, start: Start::default(), mode: Mode::Survival,
            pace: Pace { round: 1, scoring: 1 }, on_collapse: OnCollapse::Stay },
        specification: candidate.spec, execution, spatial: None, generated_templates: false }
}

#[test]
fn accepted_run_handoff_matches_public_replay_and_binds_execution() {
    for execution in [Execution::Individuals, Execution::Grouped] {
        let setup = setup(execution);
        let run = || {
            let verified = cosmograph::ecology::run_joint_validated(
                &setup.founded.genesis, &setup.specification, execution,
                &mut |session| session.advance_tick_with_flows()).unwrap();
            assert_eq!(verified.session().save(), verified.witness().native);
            let basis = cosmograph::ecology::GeneratedBasis::new(
                setup.founded.genesis.founding.as_ref().unwrap()).unwrap();
            cosmograph::ecology::admit_validated(verified,
                setup.specification.independence.as_ref().unwrap(), &basis).unwrap()
        };
        let (world, witness, admission) =
            cosmograph_host::founding::from_admitted_run(execution, run()).unwrap();
        let replayed = cosmograph_host::founding::from_admitted(execution,
            cosmograph_host::founding::Offered { witness: &witness, admission: &admission }).unwrap();
        assert_eq!(world.save().unwrap(), replayed.save().unwrap());
        assert_eq!(world.save().unwrap().native, witness.native);
        world.advance(1).unwrap();
        replayed.advance(1).unwrap();
        assert_eq!(world.save().unwrap(), replayed.save().unwrap(),
            "the live handoff and decoded replay continue through the same native clock");
        let other = if execution == Execution::Individuals {
            Execution::Grouped
        } else { Execution::Individuals };
        assert!(cosmograph_host::founding::from_admitted_run(other, run()).is_err());
    }
}

#[test]
fn actual_offer_retains_the_native_role_cut_before_same_tick_join_and_take() {
    for execution in [Execution::Individuals, Execution::Grouped] {
        let setup = setup(execution);
        let runtime = setup.offer(30).unwrap();
        let seal = runtime.spatial().ecology.as_ref().unwrap();
        seal.witness.require_roles().unwrap();
        assert_eq!(seal.witness.execution, execution);
        assert_eq!(runtime.session().sim.mode(), execution);
        assert_eq!(runtime.session().sim.genesis(), &setup.founded.genesis);
        let native = &seal.witness.native;
        let offered = cosmograph_host::founding::Offered { witness: &seal.witness, admission: &seal.admission };
        let core = cosmograph_host::founding::from_admitted(execution, offered).unwrap();
        assert_eq!(core.save().unwrap().native, *native, "the core resumes the actual admitted trajectory");
        let other = if execution == Execution::Individuals { Execution::Grouped } else { Execution::Individuals };
        assert!(cosmograph_host::founding::from_admitted(other, offered).is_err(), "execution is bound to the witness");
        let mut foreign = setup.founded.genesis.clone();
        foreign.seed ^= 1;
        assert!(cosmograph_host::founding::found(foreign, execution,
            cosmograph_host::founding::Purpose::Head, Some(offered)).is_err(), "another generated candidate is refused");
        assert_eq!(runtime.session().prefix(native.entries.len(), native.tick).unwrap().save(), *native);
        assert_eq!(runtime.session().sim.state().tick, setup.specification.horizon);
        assert!(runtime.session().entries.starts_with(&native.entries));
        let suffix = &runtime.session().entries[native.entries.len()..];
        assert_eq!(suffix.len(), 2);
        assert!(matches!(&suffix[0].command, Command::Join));
        assert!(matches!(&suffix[1].command, Command::Take { participant, critter }
            if *participant == runtime.interim().participant && Some(*critter) == runtime.critter()));
        assert!(suffix.iter().all(|entry| entry.tick == native.tick));
        assert!(directing::is_participant(runtime.session().sim.state().population.get(runtime.interim().participant).unwrap()));
        assert_eq!(runtime.interim().handover.span.epochs, 0);
        assert_eq!((runtime.interim().handover.from_tick, runtime.interim().handover.to_tick), (native.tick, native.tick));
        assert!(runtime.spatial().driver.is_none());
        assert!(runtime.spatial().members.is_empty());
    }
}

#[test]
fn game_save8_replay_and_resume_preserve_execution_seal_and_controller_once() {
    for execution in [Execution::Individuals, Execution::Grouped] {
        let runtime = setup(execution).offer(30).unwrap();
        let saved = runtime.save_game();
        assert_eq!(saved.version, 8);
        assert_eq!(saved.version, ECOLOGY_VERSION);
        assert!(saved.spatial.ecology.is_some() && saved.spatial.continuation.is_some());
        let decoded: GameSave = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        let replayed = Runtime::replayed_game(decoded.clone()).unwrap();
        assert_eq!(replayed.session.sim.mode(), execution);
        assert_eq!(replayed.session.save(), saved.native);
        let resumed = Runtime::resume_game_with_mode(decoded, 30, None).unwrap();
        assert_eq!(resumed.session().sim.mode(), execution);
        assert_eq!(resumed.save(), saved.native);
        assert_eq!(resumed.interim().participant, runtime.interim().participant);
        assert_eq!(resumed.critter(), runtime.critter());
        assert_eq!(resumed.interim().handover, runtime.interim().handover);
        assert_eq!(resumed.interim().habitable_at, runtime.interim().habitable_at);
        assert_eq!(resumed.spatial().ecology, saved.spatial.ecology);
        assert_eq!(serde_json::to_value(resumed.save_game()).unwrap(), serde_json::to_value(saved).unwrap());
    }
}

#[test]
fn game_save_downgrade_or_missing_admission_cannot_bypass_saved_offer_gate() {
    let saved = setup(Execution::Individuals).offer(30).unwrap().save_game();
    let mut downgrade = saved.clone();
    downgrade.version = 7;
    assert!(Runtime::resume_game_with_mode(downgrade, 30, None).is_err());
    let mut missing = saved;
    missing.spatial.ecology = None;
    assert!(Runtime::replayed_game(missing).is_err());
}

#[test]
fn sealed_witness_source_or_execution_tampering_refuses_actual_restore() {
    let saved = setup(Execution::Individuals).offer(30).unwrap().save_game();
    let mut branch = saved.clone();
    branch.spatial.ecology.as_mut().unwrap().witness.native.branch = "branch:another-witness".into();
    let error = Runtime::replayed_game(branch).err().expect("another native prefix must refuse");
    assert!(error.contains("another native source or witness prefix"), "{error}");
    let mut execution = saved;
    execution.spatial.ecology.as_mut().unwrap().witness.execution = Execution::Grouped;
    let error = Runtime::resume_game_with_mode(execution, 30, None).err().expect("execution seal must refuse");
    assert!(error.contains("source seals differ"), "{error}");
}

#[test]
fn mechanical_seal_evidence_is_revalidated_on_product_restore() {
    let saved = setup(Execution::Individuals).offer(30).unwrap().save_game();
    let mut json = serde_json::to_value(saved).unwrap();
    json["spatial"]["ecology"]["admission"]["evidence"][0]["accepted_actions"] = serde_json::json!([]);
    let altered: GameSave = serde_json::from_value(json).unwrap();
    let error = Runtime::replayed_game(altered).err().expect("mechanical evidence must be revalidated");
    assert!(error.contains("consumed native evidence"), "{error}");
}

#[test]
fn fresh_generated_runtime_constructor_cannot_skip_joint_admission() {
    let setup = setup(Execution::Individuals);
    let error = Runtime::new(setup.founded.clone(), 30).err().expect("marked generated constructor must refuse");
    assert!(error.contains("must pass its joint witness"), "{error}");
    let mut unmarked = setup.founded;
    unmarked.genesis.rules.ecology_witness = None;
    let error = Runtime::new(unmarked, 30).err().expect("unmarked fresh generated constructor must refuse");
    assert!(error.contains("explicit joint ecology witness"), "{error}");
}

#[test]
fn generated_a1_offer_authors_and_replays_one_physical_joint_witness() {
    for execution in [Execution::Individuals, Execution::Grouped] {
        eprintln!("generated physical witness: {execution:?}, declaring roles");
        let mut founded = Founded::generated(9, 256).unwrap();
        assert_eq!(founded.genesis.founding.as_ref().unwrap().ecology_rates,
            cosmograph::ecology::Rates::Stock);
        let mut founding = founded.genesis.founding.clone().unwrap();
        founding.ecology_rates = cosmograph::ecology::Rates::A1;
        founded.genesis = founding.generate().unwrap();
        assert_eq!(founded.genesis.founding.as_ref().unwrap().ecology_rates,
            cosmograph::ecology::Rates::A1);
        let setup = Setup::generated(founded, execution).unwrap();
        let mut legacy = serde_json::to_value(&setup).unwrap();
        legacy.as_object_mut().unwrap().remove("generated_templates");
        let old_setup: Setup = serde_json::from_value(legacy).unwrap();
        assert!(!old_setup.generated_templates, "older authored setup retains site-grain clock semantics");
        let decoded: Setup = serde_json::from_slice(&serde_json::to_vec(&setup).unwrap()).unwrap();
        assert!(decoded.generated_templates, "the generator-owned clock source survives setup persistence");
        eprintln!("generated physical witness: {execution:?}, assessing 60 ticks");
        let assessment = decoded.assess().unwrap();
        eprintln!("generated physical witness: {execution:?}, assessment complete");
        assessment.witness.require_roles().unwrap();
        assert_eq!(assessment.witness.spec.horizon, 60);
        assert_eq!(assessment.witness.spec.capture.selection.actor_lineage, "lineage:3");
        assert_eq!(assessment.witness.spec.returned.selection.actor_lineage, "lineage:3");
        assert_eq!(assessment.witness.spec.reproduction.selection.actor_lineage, "lineage:3");
        assert_eq!(assessment.witness.spec.counterplay.selection.actor_lineage, "lineage:1");
        assert_eq!(assessment.witness.spec.counterplay.selection.target_lineage.as_deref(), Some("lineage:3"));
        assert!(assessment.spatial.driver.is_some(), "889 templates must be the physical witness source");
        assessment.spatial.validate_replay(
            &cosmograph::Session::load(assessment.witness.native.clone(), execution).unwrap()).unwrap();

        eprintln!("generated physical witness: {execution:?}, offering 60 ticks");
        let runtime = decoded.offer(30).unwrap();
        let saved = runtime.save_game();
        assert!(saved.spatial.driver.is_some());
        let controlled = runtime.session().sim.plays(runtime.interim().participant)
            .expect("the admitted generated runtime controls a native body");
        // Generated stands cover only their declared residents. Other native
        // bodies remain at site grain, including newborns without a birth solver.
        let physical = saved.spatial.members.get(&controlled)
            .map(|placement| placement.realized(&runtime.session().sim));
        eprintln!("generated physical witness: {execution:?}, controlled={controlled}, physical binding={physical:?}");
        let replayed = Runtime::replayed_game(saved.clone()).unwrap();
        assert_eq!(replayed.session.sim.mode(), execution);
        assert_eq!(replayed.session.save(), saved.native);
        eprintln!("generated physical witness: {execution:?}, offer/save/replay complete");
    }
}

#[test]
fn generated_a1_offer_refuses_when_founders_have_no_viable_trophic_pair() {
    let mut founded = Founded::generated(9, 60).unwrap();
    let mut founding = founded.genesis.founding.clone().unwrap();
    founding.ecology_rates = cosmograph::ecology::Rates::A1;
    founded.genesis = founding.generate().unwrap();
    let error = Setup::generated(founded, Execution::Individuals)
        .err()
        .expect("the two seed-9 cohorts contain neither a native producer nor consumer");
    assert!(error.contains("living producer") && error.contains("native feed"), "{error}");
}
