// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Part shapes and the function catalogue (rulings 276, 338 to 341), and the
//! process's causal kind that gave up the name `Shape` for them (ruling 340).

use isocosm::{
    Execution, Founding, Session, Simulation,
    history::Saved,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

/// A session saved by the core before ruling 340, its processes' causal
/// kinds written under the old name.
const PRE_CAUSATION: &str = include_str!("data/pre-causation-world.json");

#[test]
fn a_world_saved_before_the_rename_still_loads() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let saved: Saved = serde_json::from_str(PRE_CAUSATION).unwrap();
        let hash = saved.state_hash.clone();
        // Loading checks the genesis digest, replays every command and
        // advance, and compares the state hash and every checkpoint.
        let session = Session::load(saved, mode).unwrap();
        assert_eq!(session.sim.state_hash(), hash, "{mode:?}");
        let rules = &session.sim.genesis().rules;
        let causation = |id: &str| rules.processes[id].causation;
        assert_eq!(causation("sim:remember"), Causation::Choice);
        assert_eq!(causation("ecology:death-0"), Causation::Transition);
        // Saved again, it writes the field under its old name.
        let json = serde_json::to_string(&session.save()).unwrap();
        assert!(json.contains(r#""shape":"Transition""#));
        assert!(!json.contains("causation"));
    }
}

/// Every non-empty set of the eight shapes, grown and acquired: the 510
/// functions ruling 338 asks the catalogue to admit.
fn every_function() -> BTreeMap<Key, Function> {
    let mut all = BTreeMap::new();
    for mask in 1u32..256 {
        let shapes: BTreeSet<Key> = (0..8)
            .filter(|i| mask >> i & 1 == 1)
            .map(|i| SHAPES[i].to_string())
            .collect();
        for (seeding, name) in [(Seeding::Grown, "grown"), (Seeding::Acquired, "acquired")] {
            let f = Function {
                shapes: shapes.clone(),
                seeding,
            };
            all.insert(format!("function:{mask}-{name}"), f);
        }
    }
    all
}

/// A generated world of one member, 1, adopting the eight shapes and the
/// five functions in use.
fn adopted() -> Genesis {
    let mut g = Founding {
        seed: 338,
        sites: 1,
        population: 1,
        lineages: 1,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.shapes = default_shapes();
    g.rules.functions = default_functions();
    g
}

fn part(shape: &str, functions: &[&str]) -> Part {
    Part {
        parent: None,
        traits: BTreeSet::new(),
        severed: false,
        shape: shape.into(),
        functions: functions.iter().map(|f| f.to_string()).collect(),
    }
}

fn process(id: &str, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires,
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

fn expresses(function: &str) -> Query {
    Query::Expresses {
        function: function.into(),
    }
}

#[test]
fn the_defaults_are_the_eight_shapes_and_the_five_functions_in_use() {
    let rules = adopted().rules;
    assert_eq!(rules.shapes.len(), 8);
    let names: BTreeSet<&str> = SHAPES
        .iter()
        .map(|s| s.strip_prefix("shape:").unwrap())
        .collect();
    let ruled = [
        "lump", "rod", "sheet", "point", "tube", "branch", "shell", "joint",
    ];
    assert_eq!(names, BTreeSet::from(ruled));
    let shapes = |f: &str| rules.functions[f].shapes.clone();
    let seeding = |f: &str| rules.functions[f].seeding;
    for (function, shape) in [
        ("function:contract", "shape:rod"),
        ("function:intake", "shape:lump"),
        ("function:sense", "shape:point"),
        ("function:fix", "shape:sheet"),
        ("function:secrete", "shape:sheet"),
    ] {
        assert_eq!(shapes(function), BTreeSet::from([shape.to_string()]));
    }
    assert_eq!(seeding("function:secrete"), Seeding::Acquired);
    assert_eq!(rules.functions.len(), 5);
    // A sheet grows fixing and admits secreting, which it only acquires.
    assert_eq!(
        rules.grown("shape:sheet"),
        BTreeSet::from(["function:fix".into()])
    );
    assert!(rules.admits("shape:sheet", "function:secrete"));
    assert!(!rules.admits("shape:rod", "function:secrete"));
    assert!(rules.grown("shape:tube").is_empty());
    // A generated world adopts neither; it serializes as it did before.
    let generated = Founding::default().generate().unwrap();
    let json = serde_json::to_string(&generated.rules).unwrap();
    assert!(!json.contains(r#""shapes""#) && !json.contains(r#""functions""#));
    let unshaped = serde_json::to_string(&generated.population.groups[&6].entity.parts).unwrap();
    assert!(!unshaped.contains("shape") && !unshaped.contains("functions"));
}

#[test]
fn every_one_of_the_510_functions_is_admitted() {
    let mut g = adopted();
    let all = every_function();
    assert_eq!(all.len(), 510);
    // Each is a distinct set of shapes and seeding.
    let distinct: BTreeSet<(Vec<&Key>, Seeding)> = all
        .values()
        .map(|f| (f.shapes.iter().collect(), f.seeding))
        .collect();
    assert_eq!(distinct.len(), 510);
    g.rules.functions = all.clone();
    // A process needs each; a part of each shape expresses all it admits.
    for f in all.keys() {
        let p = process(&format!("test:{f}"), vec![expresses(f)], vec![]);
        g.rules.processes.insert(p.id.clone(), p);
    }
    let body = g.population.lift(1).unwrap();
    body.parts = SHAPES
        .iter()
        .enumerate()
        .map(|(i, shape)| {
            let admitted = all.iter().filter(|(_, f)| f.shapes.contains(*shape));
            let admitted: Vec<&str> = admitted.map(|(k, _)| k.as_str()).collect();
            assert_eq!(admitted.len(), 256);
            (i as Id, part(shape, &admitted))
        })
        .collect();
    g.validate().unwrap();
    // Each binds the lowest-numbered part expressing it: part i is the i-th
    // shape, so the lowest shape in the function's set, and the act's
    // receipt reads that part's address.
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    for (mask, name) in (1u32..256).flat_map(|m| [(m, "grown"), (m, "acquired")]) {
        let f = format!("function:{mask}-{name}");
        let r = sim.execute(1, None, &format!("test:{f}"), None);
        assert_eq!(r.outcome, Outcome::Accepted, "{f}");
        let lowest = mask.trailing_zeros();
        let fact = format!("{:?} = part {lowest} at revision 1", expresses(&f));
        assert_eq!(r.facts_read, vec![fact], "{f}");
    }
}

#[test]
fn a_catalogue_and_its_parts_round_trip_through_bytes() {
    let mut g = adopted();
    g.rules.functions.extend(every_function());
    let body = g.population.lift(1).unwrap();
    body.parts.insert(
        1,
        part("shape:sheet", &["function:fix", "function:4-acquired"]),
    );
    body.parts.insert(
        2,
        part("shape:rod", &["function:contract", "function:3-grown"]),
    );
    g.validate().unwrap();
    let bytes = serde_json::to_vec(&g).unwrap();
    let back: Genesis = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(back, g);
    assert_eq!(serde_json::to_vec(&back).unwrap(), bytes);
    assert_eq!(back.rules.revision(), g.rules.revision());
}

#[test]
fn a_function_is_its_set_of_shapes_and_its_seeding() {
    let read = |json: &str| -> Function { serde_json::from_str(json).unwrap() };
    let revision = |f: Function| {
        let mut rules = adopted().rules;
        rules.functions.insert("function:x".into(), f);
        rules.revision()
    };
    let a = read(r#"{"shapes":["shape:sheet","shape:lump"],"seeding":"Grown"}"#);
    let b = read(r#"{"shapes":["shape:lump","shape:sheet","shape:lump"],"seeding":"Grown"}"#);
    let c = read(r#"{"shapes":["shape:lump","shape:sheet"],"seeding":"Acquired"}"#);
    assert_eq!(a, b);
    assert_eq!(revision(a.clone()), revision(b));
    assert_ne!(revision(a), revision(c));
    // A part's functions are a set too.
    let json = r#"{"parent":null,"traits":[],"severed":false,"shape":"shape:sheet",
        "functions":["function:secrete","function:fix","function:secrete"]}"#;
    let p: Part = serde_json::from_str(json).unwrap();
    assert_eq!(
        p,
        part("shape:sheet", &["function:fix", "function:secrete"])
    );
}

#[test]
fn bad_shapes_functions_and_bindings_are_refused() {
    let usable = |p: &mut Process| p.requires.push(expresses("function:fix"));
    type Change = Box<dyn Fn(&mut Genesis)>;
    let cases: Vec<(&str, Change)> = vec![
        (
            "is admitted by no shape",
            Box::new(|g: &mut Genesis| {
                let f = Function {
                    shapes: BTreeSet::new(),
                    seeding: Seeding::Grown,
                };
                g.rules.functions.insert("function:none".into(), f);
            }),
        ),
        (
            "names an unknown shape",
            Box::new(|g: &mut Genesis| {
                let shapes = BTreeSet::from(["shape:wing".into()]);
                let f = Function {
                    shapes,
                    seeding: Seeding::Grown,
                };
                g.rules.functions.insert("function:fly".into(), f);
            }),
        ),
        (
            "invalid namespaced key",
            Box::new(|g: &mut Genesis| {
                g.rules.shapes.insert("lump".into());
            }),
        ),
        (
            "invalid namespaced key",
            Box::new(|g: &mut Genesis| {
                let f = g.rules.functions["function:fix"].clone();
                g.rules.functions.insert("fix".into(), f);
            }),
        ),
        (
            "unknown part shape",
            Box::new(|g: &mut Genesis| {
                g.population
                    .lift(1)
                    .unwrap()
                    .parts
                    .insert(1, part("shape:wing", &[]));
            }),
        ),
        (
            "unknown function",
            Box::new(|g: &mut Genesis| {
                let p = part("shape:rod", &["function:fly"]);
                g.population.lift(1).unwrap().parts.insert(1, p);
            }),
        ),
        (
            "cannot express",
            Box::new(|g: &mut Genesis| {
                let p = part("shape:rod", &["function:fix"]);
                g.population.lift(1).unwrap().parts.insert(1, p);
            }),
        ),
        (
            "cannot express",
            Box::new(|g: &mut Genesis| {
                let p = part("", &["function:contract"]);
                g.population.lift(1).unwrap().parts.insert(1, p);
            }),
        ),
        (
            "unknown function",
            Box::new(|g: &mut Genesis| {
                let p = process("test:fly", vec![expresses("function:fly")], vec![]);
                g.rules.processes.insert(p.id.clone(), p);
            }),
        ),
        (
            "binds more than one part",
            Box::new(|g: &mut Genesis| {
                let two = vec![expresses("function:fix"), expresses("function:intake")];
                let p = process("test:two", two, vec![]);
                g.rules.processes.insert(p.id.clone(), p);
            }),
        ),
        (
            "uses a part it does not bind",
            Box::new(|g: &mut Genesis| {
                let read = Query::Trait {
                    who: Binding::Part,
                    key: "ability:cycle-0".into(),
                };
                let p = process("test:unbound", vec![read], vec![]);
                g.rules.processes.insert(p.id.clone(), p);
            }),
        ),
        (
            "keeps no ledger",
            Box::new(move |g: &mut Genesis| {
                let read = Query::Account {
                    who: Binding::Part,
                    key: "sim:energy".into(),
                    at_least: 1,
                };
                let mut p = process("test:ledger", vec![read], vec![]);
                usable(&mut p);
                g.rules.processes.insert(p.id.clone(), p);
            }),
        ),
        (
            "keeps no ledger",
            Box::new(move |g: &mut Genesis| {
                let into = Effect::Transfer {
                    from: Binding::Actor,
                    to: Binding::Part,
                    account: "world:soil".into(),
                    amount: 1,
                };
                let mut p = process("test:into", vec![], vec![into]);
                usable(&mut p);
                g.rules.processes.insert(p.id.clone(), p);
            }),
        ),
        (
            "a need reads only its member",
            Box::new(|g: &mut Genesis| {
                let read = Query::Trait {
                    who: Binding::Part,
                    key: "ability:cycle-0".into(),
                };
                with_mind(g, read);
            }),
        ),
    ];
    for (why, change) in cases {
        let mut g = adopted();
        change(&mut g);
        let refusal = g.validate().unwrap_err();
        assert!(refusal.contains(why), "{why}: {refusal}");
    }
    // A need may ask whether its member expresses a function, which reads
    // only the member.
    let mut g = adopted();
    with_mind(&mut g, expresses("function:sense"));
    g.validate().unwrap();
}

/// A mind with one need, reading `query`.
fn with_mind(g: &mut Genesis, query: Query) {
    g.rules
        .accounts
        .insert("test:strain".into(), AccountKind::Strain);
    g.rules.mind = Some(Mind {
        strain: "test:strain".into(),
        needs: vec![Need {
            traits: BTreeSet::new(),
            query,
            weight: 1,
        }],
        bearing: 10,
        bearing_traits: BTreeMap::new(),
        rise: 0,
        rise_traits: BTreeMap::new(),
        stake: 0,
    });
}
