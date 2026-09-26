use isocosm::{
    Execution, Founding, Session, Simulation, history::Command, rules::*, schema::*,
    simulation::Outcome,
};
use std::collections::{BTreeMap, BTreeSet};

fn founding(seed: u64) -> Founding {
    Founding {
        seed,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Default::default()
    }
}

#[test]
fn drawn_worlds_have_exact_foreground_background_parity() {
    for seed in 0..12 {
        let comparison = isocosm::aggregate::compare(&founding(seed), 20).unwrap();
        assert_eq!(
            comparison.individuals.represented,
            comparison.grouped.represented
        );
        assert_eq!(comparison.individuals.accepted, comparison.grouped.accepted);
        assert!(comparison.grouped.evaluations < comparison.individuals.evaluations);
    }
}

#[test]
fn lift_restrict_preserves_every_identity_and_heterogeneous_reserve() {
    let mut population = founding(77).generate().unwrap().population;
    let before = population.normalized();
    for id in 0..population.next_id {
        population.lift(id).unwrap();
    }
    population.restrict(&BTreeSet::new());
    assert_eq!(population, before);
    let id = 5;
    population
        .lift(id)
        .unwrap()
        .accounts
        .insert("world:soil".into(), 999);
    population.restrict(&BTreeSet::new());
    assert_eq!(population.get(id).unwrap().accounts["world:soil"], 999);
    assert_eq!(population.get(id + 1).unwrap().accounts["world:soil"], 4);
}

#[test]
fn failed_second_effect_rolls_back_first_effect_and_identity_split() {
    let f = founding(8);
    let mut genesis = f.generate().unwrap();
    let p = genesis.rules.processes.get_mut("sim:give").unwrap();
    p.effects.push(Effect::Transfer {
        from: Binding::Actor,
        to: Binding::Target,
        account: "world:soil".into(),
        amount: 999,
    });
    let mut sim = Simulation::new(genesis, Execution::Grouped).unwrap();
    let before = sim.state_hash();
    let groups = sim.state().population.groups.len();
    let r = sim.execute(3, Some(4), "sim:give", None);
    assert!(matches!(r.outcome, Outcome::Blocked(_)));
    assert_eq!(sim.state_hash(), before);
    assert_eq!(sim.state().population.groups.len(), groups);
}

#[test]
fn shared_transfer_conserves_counts_and_prevents_negative_stock() {
    let mut sim = Simulation::new(founding(9).generate().unwrap(), Execution::Grouped).unwrap();
    let mass = sim.matter();
    for _ in 0..4 {
        assert_eq!(
            sim.execute(3, Some(4), "sim:give", None).outcome,
            Outcome::Accepted
        );
    }
    assert_eq!(
        sim.state().population.get(3).unwrap().accounts["world:soil"],
        0
    );
    assert_eq!(
        sim.state().population.get(4).unwrap().accounts["world:soil"],
        8
    );
    let before = sim.state_hash();
    assert!(matches!(
        sim.execute(3, Some(4), "sim:give", None).outcome,
        Outcome::Blocked(_)
    ));
    assert_eq!(sim.state_hash(), before);
    assert_eq!(sim.matter(), mass);
}

#[test]
fn offspring_have_paid_matter_and_pointable_parentage() {
    let mut sim = Simulation::new(founding(33).generate().unwrap(), Execution::Grouped).unwrap();
    let mass = sim.matter();
    let next = sim.state().population.next_id;
    assert_eq!(
        sim.execute(3, None, "sim:birth", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(
        sim.state().population.get(next).unwrap().accounts["world:soil"],
        2
    );
    assert_eq!(
        sim.state().population.get(3).unwrap().accounts["world:soil"],
        2
    );
    assert!(sim.state().relations.contains(&Relation {
        subject: next,
        kind: "sim:parent".into(),
        object: 3
    }));
    assert_eq!(sim.matter(), mass);
}

#[test]
fn known_events_follow_reachable_routes_and_notes_survive_decay() {
    let mut genesis = founding(1).generate().unwrap();
    // Directed disconnected site; no invented reverse edges.
    genesis.sites.get_mut(&0).unwrap().routes = vec![Route {
        to: 1,
        travel: 4,
        transmission: 1_000_000,
    }];
    genesis.sites.get_mut(&1).unwrap().routes.clear();
    genesis.sites.get_mut(&2).unwrap().routes.clear();
    genesis.population.lift(3).unwrap().place = 0;
    genesis.population.lift(4).unwrap().place = 1;
    genesis.population.lift(5).unwrap().place = 2;
    let mut sim = Simulation::new(genesis, Execution::Grouped).unwrap();
    let r = sim.execute(3, None, "sim:remember", None);
    assert!(!sim.knows(4, &r.id).unwrap());
    sim.advance(4).unwrap();
    assert!(sim.learn(4, &r.id).unwrap());
    assert!(!sim.knows(5, &r.id).unwrap());
    let event = sim.state().events.get(&r.id).unwrap();
    assert_eq!(sim.state().reach.learning_path(event, 1), vec![1, 0]);
    sim.advance(150).unwrap();
    assert!(sim.knows(4, &r.id).unwrap());
}

#[test]
fn rules_admission_rejects_unknown_and_unbalanced_transforms() {
    let mut g = founding(4).generate().unwrap();
    let before = g.rules.revision();
    g.rules.field.legend_floor += 1;
    assert_ne!(before, g.rules.revision());
    g.rules.processes.get_mut("sim:give").unwrap().effects = vec![Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([("world:soil".into(), 1)]),
    }];
    assert!(g.validate().unwrap_err().contains("unbalanced"));
    g.rules.processes.get_mut("sim:give").unwrap().effects = vec![Effect::Condition {
        key: "absent:condition".into(),
        delta: 1,
    }];
    assert!(g.validate().unwrap_err().contains("unknown condition"));
}

#[test]
fn saved_history_roundtrips_and_tampering_is_refused() {
    let mut session = Session::new(founding(19).generate().unwrap(), Execution::Grouped).unwrap();
    session.advance(17).unwrap();
    session.command(Command::Inspect(3)).unwrap();
    session
        .command(Command::Act {
            actor: 3,
            target: Some(4),
            process: "sim:give".into(),
            cause: None,
        })
        .unwrap();
    session.advance(31).unwrap();
    let bytes = serde_json::to_vec(&session.save()).unwrap();
    let replay = Session::load(
        serde_json::from_slice(&bytes).unwrap(),
        Execution::Individuals,
    )
    .unwrap();
    assert_eq!(session.sim.state_hash(), replay.sim.state_hash());
    let mut bad = session.save();
    bad.genesis.rules.field.strength = 0;
    assert!(Session::load(bad, Execution::Grouped).is_err());
    let mut bad = session.save();
    bad.entries[1].outcome = "Accepted".into();
    assert!(Session::load(bad, Execution::Grouped).is_err());
}

#[test]
fn branches_preserve_played_worlds_and_surface_changed_accepted_outcomes() {
    let base = Session::new(founding(25).generate().unwrap(), Execution::Grouped).unwrap();
    let mut a = base.fork_at(0, "branch:a".into()).unwrap();
    let mut b = base.fork_at(0, "branch:b".into()).unwrap();
    a.command(Command::Act {
        actor: 3,
        target: Some(4),
        process: "sim:give".into(),
        cause: None,
    })
    .unwrap();
    b.command(Command::Act {
        actor: 3,
        target: Some(5),
        process: "sim:give".into(),
        cause: None,
    })
    .unwrap();
    let original = a.sim.state_hash();
    let merged = a.merge(&b).unwrap();
    assert_eq!(a.sim.state_hash(), original);
    assert!(!merged.changes.is_empty());
    assert_eq!(
        merged
            .proposed
            .sim
            .state()
            .population
            .get(3)
            .unwrap()
            .accounts["world:soil"],
        2
    );
    assert_eq!(
        merged.proposed.sim.state_hash(),
        b.merge(&a).unwrap().proposed.sim.state_hash()
    );
    Session::load(merged.proposed.save(), Execution::Grouped).unwrap();
    b.advance(1).unwrap();
    assert!(a.merge(&b).is_err());
}

#[test]
fn dynamics_seed_is_absent_unless_set_and_defaults_to_the_world_seed() {
    let g = founding(3).generate().unwrap();
    assert_eq!(g.dynamics_seed(), g.seed);
    // Worlds without one serialize, and so hash, as before the field existed.
    assert!(!serde_json::to_string(&g).unwrap().contains("\"dynamics\""));
    let mut set = g.clone();
    set.dynamics = Some(g.seed ^ 1);
    assert_eq!(set.dynamics_seed(), g.seed ^ 1);
    assert_ne!(isocosm::digest(&set), isocosm::digest(&g));
}

#[test]
fn rules_without_a_competition_serialize_and_hash_as_before() {
    let g = founding(3).generate().unwrap();
    let json = serde_json::to_string(&g.rules).unwrap();
    assert!(!json.contains("\"competitions\"") && !json.contains("\"similitude\""));
    // Rules saved before the fields existed lack both, and still load.
    let back: isocosm::rules::Rules = serde_json::from_str(&json).unwrap();
    assert_eq!(back.revision(), g.rules.revision());
}

#[test]
fn operation_budget_failure_is_transactionally_inert() {
    let mut genesis = founding(91).generate().unwrap();
    genesis.rules.limits.events_per_advance = 1;
    let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
    let before = sim.state_hash();
    assert!(sim.advance(10).is_err());
    assert_eq!(before, sim.state_hash());
}

#[test]
fn conservation_does_not_suffice_to_merge_different_starvation_states() {
    let mut a = founding(18).generate().unwrap().population;
    let mut b = a.clone();
    a.lift(3).unwrap().accounts.insert("world:soil".into(), 0);
    a.lift(4).unwrap().accounts.insert("world:soil".into(), 8);
    b.lift(3).unwrap().accounts.insert("world:soil".into(), 4);
    b.lift(4).unwrap().accounts.insert("world:soil".into(), 4);
    assert_eq!(a.totals().unwrap(), b.totals().unwrap());
    a.restrict(&BTreeSet::new());
    b.restrict(&BTreeSet::new());
    assert_ne!(a, b);
}

#[test]
fn interacting_ecologies_keep_identity_and_mass_through_birth_and_death() {
    // Declared lineages remain valid when a small population draws none of them.
    Founding {
        seed: 12,
        ecology: true,
        population: 1,
        lineages: 6,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for seed in 0..3 {
        let f = Founding {
            seed,
            ecology: true,
            population: 18,
            cohort_size: 2,
            sites: 2,
            lineages: 3,
            ..Default::default()
        };
        let result = isocosm::aggregate::compare(&f, 64).unwrap();
        assert_eq!(result.individuals.accepted, result.grouped.accepted);
        let mut s = Session::new(f.generate().unwrap(), Execution::Grouped).unwrap();
        s.advance(64).unwrap();
        assert!(
            s.sim
                .state()
                .population
                .groups
                .values()
                .any(|g| !g.entity.alive)
        );
        assert!(
            s.sim
                .state()
                .events
                .values()
                .any(|e| e.process.starts_with("ecology:"))
        );
        Session::load(s.save(), Execution::Individuals).unwrap();
    }
}

#[test]
fn past_exposure_is_retained_after_movement_and_decay() {
    let mut g = founding(83).generate().unwrap();
    for site in g.sites.values_mut() {
        site.routes.clear();
    }
    g.sites.get_mut(&0).unwrap().routes.push(Route {
        to: 1,
        travel: 10,
        transmission: 0,
    });
    g.population.lift(3).unwrap().place = 0;
    let mut move_to = g.rules.processes["sim:remember"].clone();
    move_to.id = "sim:move".into();
    move_to.effects = vec![Effect::Move { destination: 1 }];
    g.rules.processes.insert(move_to.id.clone(), move_to);
    let mut sim = Simulation::new(g, Execution::Grouped).unwrap();
    let event = sim.execute(3, None, "sim:remember", None);
    sim.advance(1).unwrap();
    assert_eq!(
        sim.execute(3, None, "sim:move", None).outcome,
        Outcome::Accepted
    );
    sim.advance(150).unwrap();
    assert!(sim.knows(3, &event.id).unwrap());
    assert!(sim.learn(3, &event.id).unwrap());
    let n = sim.state().notes.len();
    assert!(sim.learn(3, &event.id).unwrap());
    assert_eq!(sim.state().notes.len(), n);
}

#[test]
fn failed_learning_keeps_observation_and_grouping_unchanged() {
    let mut g = founding(12).generate().unwrap();
    g.rules.limits.notes = 2;
    let mut sim = Simulation::new(g, Execution::Grouped).unwrap();
    let event = sim.execute(3, None, "sim:remember", None);
    let before = sim.state().clone();
    assert!(sim.learn(3, &event.id).is_err());
    assert_eq!(&before, sim.state());
}

#[test]
fn release_honours_buffer_and_history_judges_legend() {
    let mut sim = Simulation::new(founding(99).generate().unwrap(), Execution::Grouped).unwrap();
    sim.inspect(4).unwrap();
    sim.release(4);
    sim.collect();
    assert!(sim.kept().contains(&4));
    sim.advance(16).unwrap();
    sim.collect();
    assert!(!sim.kept().contains(&4));
    let first = sim.execute(3, None, "sim:reckon", None);
    assert!(!sim.state().events[&first.id].legend);
    sim.execute(4, Some(3), "sim:give", None);
    let record = sim.execute(3, None, "sim:reckon", None);
    assert!(sim.state().events[&record.id].legend);
    assert!(!sim.state().reach.arrivals.contains_key(&record.id));
    sim.release(3);
    sim.advance(200).unwrap();
    sim.collect();
    assert!(sim.kept().contains(&3));
    assert_eq!(
        sim.state().reach.strength(
            &sim.state().events[&record.id],
            2,
            sim.state().tick,
            &sim.genesis().rules.field
        ),
        250_000
    );
}
