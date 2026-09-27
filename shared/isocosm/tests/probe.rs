use isocosm::probe::{
    Crowd, PredatorFounding, ProbeFounding, Variant,
    check::{self, Settings},
    readings::{self, Probe},
    run_exact,
};

fn world(seed: u64) -> isocosm::probe::ProbeWorld {
    ProbeFounding {
        seed,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

fn values(world: &isocosm::probe::ProbeWorld, arm: &str, dynamics: u64) -> Vec<u64> {
    let derived = readings::derive(world);
    if arm == "exact" {
        let run = run_exact(world, dynamics, true).unwrap();
        let members = readings::exact_members(&run);
        let inspected = readings::inspect(&members, dynamics);
        let s = run.sim.state();
        let meals = &run.meals;
        readings::evaluate(
            &derived, world, &members, &s.sites, s.tick, inspected, meals,
        )
        .unwrap()
    } else {
        let variant = match arm {
            "crowd" => Variant::Histogram,
            "approximate" => Variant::Approximate,
            _ => Variant::Averaged,
        };
        let crowd = Crowd::new(world, dynamics, variant).unwrap().run().unwrap();
        let members = readings::crowd_members(&crowd);
        let inspected = readings::inspect(&members, dynamics);
        readings::evaluate(
            &derived,
            world,
            &members,
            &crowd.sites,
            crowd.tick,
            inspected,
            &crowd.meals,
        )
        .unwrap()
    }
}

#[test]
fn drawn_probe_worlds_run_both_ways_and_conserve_matter() {
    for seed in 0..4 {
        let w = ProbeFounding {
            seed,
            ticks: 8,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let before = isocosm::Simulation::new(w.genesis.clone(), isocosm::Execution::Individuals)
            .unwrap()
            .matter();
        let run = run_exact(&w, seed, true).unwrap();
        assert_eq!(run.sim.matter(), before);
        assert_eq!(run.sim.state().tick, w.ticks);
        // The crowd checks its own total every tick and fails if it moves.
        for variant in [Variant::Histogram, Variant::Averaged, Variant::Approximate] {
            let crowd = Crowd::new(&w, seed, variant).unwrap().run().unwrap();
            assert_eq!(crowd.tick, w.ticks);
        }
    }
}

#[test]
fn collecting_and_rerunning_change_nothing_while_dynamics_do() {
    let w = world(5);
    let a = run_exact(&w, 9, true).unwrap().sim.state_hash();
    assert_eq!(a, run_exact(&w, 9, true).unwrap().sim.state_hash());
    assert_eq!(a, run_exact(&w, 9, false).unwrap().sim.state_hash());
    let differs = (10..14).any(|d| run_exact(&w, d, true).unwrap().sim.state_hash() != a);
    assert!(
        differs,
        "the dynamics seed should change how the world unfolds"
    );
}

#[test]
fn without_shortage_the_crowd_is_the_exact_runner() {
    // Food that always covers every hungry member leaves nothing random, so
    // every reading but the inspection must agree exactly.
    let mut founding = ProbeFounding {
        seed: 17,
        ticks: 2,
        ..Default::default()
    };
    founding.regrowth = [5000, 5000];
    for seed in 17..21 {
        founding.seed = seed;
        let w = founding.generate().unwrap();
        let derived = readings::derive(&w);
        let exact = values(&w, "exact", 1);
        for arm in ["crowd", "approximate"] {
            let crowd = values(&w, arm, 2);
            for (r, (x, y)) in derived.iter().zip(exact.iter().zip(&crowd)) {
                if !matches!(r.probe, Probe::Inspect { .. }) {
                    assert_eq!(x, y, "{} in world {seed}, {arm}", r.key);
                }
            }
        }
    }
}

#[test]
fn readings_come_from_the_definitions() {
    let w = world(2);
    let derived = readings::derive(&w);
    let keys: Vec<&str> = derived.iter().map(|r| r.key.as_str()).collect();
    for kind in w.kinds() {
        assert!(keys.contains(&format!("alive:{}", kind.identity).as_str()));
        assert!(keys.contains(&format!("world:food#hungry:{}", kind.identity).as_str()));
        assert!(keys.contains(&format!("past-bearing:{}", kind.identity).as_str()));
    }
    let starvation: Vec<&str> = derived
        .iter()
        .filter(|r| r.starvation)
        .map(|r| r.source.as_str())
        .collect();
    assert_eq!(starvation, ["probe:starve-0", "probe:starve-1"]);
    // A new threshold in the rules is a new reading, with no other change.
    let mut more = w.clone();
    let p = more.genesis.rules.processes.get_mut("probe:eat-0").unwrap();
    p.requires.push(isocosm::rules::Query::Below {
        who: isocosm::rules::Binding::Actor,
        key: "matter:0-0".into(),
        amount: 99,
    });
    assert_eq!(readings::derive(&more).len(), derived.len() + 1);
}

#[test]
fn the_competition_and_its_bounds_are_rules_the_world_admits() {
    let w = world(4);
    let json = serde_json::to_string(&w.genesis.rules).unwrap();
    let back: isocosm::rules::Rules = serde_json::from_str(&json).unwrap();
    assert_eq!(back, w.genesis.rules);
    let refused = |edit: &dyn Fn(&mut isocosm::rules::Rules)| {
        let mut g = w.genesis.clone();
        edit(&mut g.rules);
        g.validate().unwrap_err()
    };
    let food = "world:food";
    assert!(
        refused(&|r| r.competitions.get_mut(food).unwrap().kinds[0].eat = "probe:none".into())
            .contains("unknown process")
    );
    // A competition is keyed by the matter it contests (ruling 236).
    assert!(
        refused(&|r| {
            let c = r.competitions.remove(food).unwrap();
            r.competitions.insert("mind:strain".into(), c);
        })
        .contains("not a matter account")
    );
    assert!(
        refused(&|r| r.similitude.as_mut().unwrap().default_bound = 1001)
            .contains("exceeds certainty")
    );
    // A lineage fights with one reserve, spent one way, everywhere.
    assert!(
        refused(&|r| {
            let mut c = r.competitions[food].clone();
            c.kinds[0].spend = "probe:upkeep-0".into();
            r.competitions.insert("world:soil".into(), c);
        })
        .contains("fights differently across competitions")
    );
}

#[test]
fn averaging_flattens_reserves_within_each_lineage_and_site() {
    let w = world(8);
    let crowd = Crowd::new(&w, 3, Variant::Averaged).unwrap().run().unwrap();
    for kind in w.kinds() {
        for site in crowd.sites.keys() {
            let bodies: Vec<u64> = crowd
                .bins
                .keys()
                .filter(|e| e.alive && e.place == *site && e.traits.contains(&kind.identity))
                .map(|e| e.accounts.get(&kind.body).copied().unwrap_or(0))
                .collect();
            let (lo, hi) = (bodies.iter().min(), bodies.iter().max());
            if let (Some(lo), Some(hi)) = (lo, hi) {
                assert!(hi - lo <= 1, "{kind:?} at {site}: {bodies:?}");
            }
        }
    }
}

#[test]
fn the_statistics_detect_what_they_should() {
    let same: Vec<u64> = (0..200).map(|i| i % 7).collect();
    assert_eq!(check::distance(&same, &same), 0.0);
    assert_eq!(check::distance(&[1, 1, 1], &[5, 5, 5]), 1.0);
    let adjusted = check::holm(&[0.01, 0.04, 0.03]);
    for (got, want) in adjusted.iter().zip([0.03, 0.06, 0.06]) {
        assert!((got - want).abs() < 1e-12, "{adjusted:?}");
    }
    assert!(check::equivalence(0.0, 0.2, 1000) < check::equivalence(0.0, 0.2, 200));
    assert_eq!(check::equivalence(0.25, 0.2, 1000), 1.0);
    let settings = Settings {
        alpha: 0.05,
        permutations: 199,
        seed: 1,
    };
    let rows = |shift: u64| -> Vec<Vec<u64>> { (0..600).map(|k| vec![k % 9 + shift]).collect() };
    let readings = [("r".to_string(), 200)];
    let alike = check::compare("alike", &readings, &rows(0), &rows(0), settings);
    assert!(alike.pass && !alike.different && alike.equivalent);
    let apart = check::compare("apart", &readings, &rows(0), &rows(4), settings);
    assert!(apart.different && !apart.equivalent && !apart.pass);
}

#[test]
fn a_crowd_under_certification_may_refuse_one_draw_in_a_hundred() {
    let settings = Settings {
        alpha: 0.05,
        permutations: 199,
        seed: 1,
    };
    let readings = [("r".to_string(), 200)];
    let exact: Vec<Option<Vec<u64>>> = (0..1000).map(|k| Some(vec![k % 9])).collect();
    // The crowd reads as the exact runner does, but for the draws it refused.
    let run = |refused: usize, bounded: bool| {
        let crowd: Vec<_> = (exact.iter().enumerate())
            .map(|(k, row)| row.clone().filter(|_| k >= refused))
            .collect();
        check::compare_runs("r", &readings, &exact, &crowd, settings, bounded)
    };
    // Ten of a thousand is within the bound, and the draws refused are left
    // out of the readings.
    let within = run(10, true);
    assert!(within.pass && within.equivalent && !within.different);
    assert_eq!(within.draws, 990);
    let r = within.refusals.unwrap();
    assert_eq!((r.refused, r.of, r.within), (10, 1000, true));
    // Eleven is past it, and fails a comparison that otherwise passes.
    let past = run(11, true);
    assert!(past.equivalent && !past.different && !past.pass);
    assert!(!past.refusals.unwrap().within);
    // A control's refusals are recorded with no bound.
    let control = run(11, false);
    assert!(control.pass);
    let r = control.refusals.unwrap();
    assert_eq!((r.refused, r.bound_per_mille, r.within), (11, None, true));
}

fn hunted(seed: u64, water: bool) -> isocosm::probe::ProbeWorld {
    ProbeFounding {
        seed,
        ticks: 9,
        water,
        predators: Some(PredatorFounding::default()),
        ..Default::default()
    }
    .generate()
    .unwrap()
}

#[test]
fn hunters_live_by_eating_the_others_both_ways_and_keep_matter() {
    for seed in 0..2 {
        let w = hunted(seed, seed % 2 == 1);
        let before = isocosm::Simulation::new(w.genesis.clone(), isocosm::Execution::Individuals)
            .unwrap()
            .matter();
        let run = run_exact(&w, seed, true).unwrap();
        assert_eq!(run.sim.matter(), before);
        // The hunters ate, and the core's scheduler reported each meal.
        let hunt = "probe:hunt-2";
        assert!(run.meals[hunt].count > 0, "world {seed}");
        let variants = [
            Variant::Histogram,
            Variant::Averaged,
            Variant::Approximate,
            Variant::Unweighted,
        ];
        for variant in variants {
            let crowd = Crowd::new(&w, seed, variant).unwrap().run().unwrap();
            assert!(crowd.meals[hunt].count > 0, "world {seed}, {variant:?}");
            if variant == Variant::Histogram {
                let again = Crowd::new(&w, seed, variant).unwrap().run().unwrap();
                assert_eq!(again.bins, crowd.bins, "world {seed}");
            }
        }
    }
}

#[test]
fn hunters_read_as_any_lineage_does() {
    let w = hunted(2, false);
    let derived = readings::derive(&w);
    let keys: Vec<&str> = derived.iter().map(|r| r.key.as_str()).collect();
    // Hunting while below the appetite is a threshold of its own; what the
    // prey holds is read of the prey, which a count of members cannot show.
    assert!(keys.contains(&"probe:hunt-2#2"));
    assert!(!keys.contains(&"probe:hunt-2#3"));
    assert!(keys.contains(&"inspect:account:matter:2-0"));
    // Where the draw took its meals is read directly, and so is the prey's
    // fat, which only a hunt takes.
    assert!(keys.contains(&"probe:hunt-2#meal-holdings"));
    assert!(keys.contains(&"inspect:account:matter:0-2"));
    let starvation: Vec<&str> = derived
        .iter()
        .filter(|r| r.starvation)
        .map(|r| r.source.as_str())
        .collect();
    assert_eq!(
        starvation,
        ["probe:starve-0", "probe:starve-1", "probe:starve-2"]
    );
    // Every reading of the world without hunters is still read.
    let plain = readings::derive(&world(2));
    for r in &plain {
        assert!(keys.contains(&r.key.as_str()), "{}", r.key);
    }
}
