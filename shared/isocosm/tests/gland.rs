// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's lowered script: Mesocosm's `gland.lua` as a process
//! definition, with what lowering it needed: draws keyed by slot, readings of
//! the bound part, and an allocation that places an acquired function.

use isocosm::{
    Execution, Founding, Simulation,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const FIX: &str = "function:fix";
const SECRETE: &str = "function:secrete";
const CANDIDATE: &str = "ability:candidate-secrete";

/// One critter, id 1: a frond fixing with all eight of its cells, five
/// matter a cell, and a lump; `soil` at its site.
fn world(soil: u64) -> Genesis {
    let mut g = Founding {
        seed: 9,
        sites: 1,
        population: 1,
        lineages: 1,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.processes.retain(|_, p| p.period.is_none());
    g.rules.shapes = default_shapes();
    g.rules.functions = default_functions();
    g.rules.traits.insert(CANDIDATE.into());
    let critter = g.population.lift(1).unwrap();
    critter.traits.insert(CANDIDATE.into());
    critter.parts = BTreeMap::from([
        (
            0,
            Part {
                shape: "part-shape:sheet".into(),
                functions: BTreeSet::from([FIX.into()]),
                capacity: 8,
                cells: BTreeMap::from([(FIX.into(), 8)]),
                cell_mass: 5,
                ..Default::default()
            },
        ),
        (
            1,
            Part {
                shape: "part-shape:lump".into(),
                functions: BTreeSet::from(["function:intake".into()]),
                capacity: 2,
                ..Default::default()
            },
        ),
    ]);
    for site in g.sites.values_mut() {
        site.accounts.insert("world:soil".into(), soil);
    }
    g
}

fn process(id: &str, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires: [vec![Query::Alive(Binding::Actor)], requires].concat(),
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

fn binds(function: &str) -> Vec<Query> {
    vec![Query::Expresses {
        function: function.into(),
    }]
}

fn read(r: Reading) -> Expr {
    Expr::Read(r)
}

/// `gland.lua`, lowered: four cells or five by a draw, one where the ground
/// cannot charge that many at the part's cell price, never more than the
/// frond holds; the line must have the gland among its candidates.
fn gland() -> Process {
    let appetite = || Expr::Add(vec![Expr::Const(4), Expr::Draw { below: 2, slot: 0 }]);
    let ground = read(Reading::Account {
        who: Binding::Place,
        key: "world:soil".into(),
    });
    let price = Expr::Mul(vec![
        read(Reading::CellWeight { who: Binding::Part }),
        appetite(),
    ]);
    let rich = Expr::AtLeast(Box::new(ground), Box::new(price));
    let wanted = Expr::Add(vec![
        Expr::Const(1),
        Expr::Mul(vec![rich, Expr::Add(vec![appetite(), Expr::Const(-1)])]),
    ]);
    let held = read(Reading::Cells {
        who: Binding::Part,
        function: FIX.into(),
    });
    let mut requires = binds(FIX);
    requires.push(Query::Trait {
        who: Binding::Actor,
        key: CANDIDATE.into(),
    });
    process(
        "test:gland",
        requires,
        vec![Effect::Allocate {
            from: Some(FIX.into()),
            to: SECRETE.into(),
            cells: Amount::Computed(Expr::Min(vec![wanted, held])),
        }],
    )
}

fn cells(sim: &Simulation, part: Id) -> BTreeMap<Key, u32> {
    sim.state().population.get(1).unwrap().parts[&part]
        .cells
        .clone()
}

fn express_gland(soil: u64) -> (BTreeMap<Key, u32>, BTreeSet<Key>) {
    let mut g = world(soil);
    let p = gland();
    g.rules.processes.insert(p.id.clone(), p);
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    assert_eq!(
        sim.execute(1, None, "test:gland", None).outcome,
        Outcome::Accepted
    );
    let part = &sim.state().population.get(1).unwrap().parts[&0];
    (part.cells.clone(), part.functions.clone())
}

#[test]
fn the_gland_takes_its_appetite_on_rich_ground_and_a_token_on_lean() {
    let (rich, functions) = express_gland(1_000);
    let gland = rich[SECRETE];
    assert!(gland == 4 || gland == 5, "{rich:?}");
    assert_eq!(rich[FIX], 8 - gland);
    assert!(functions.contains(SECRETE), "the frond came to secrete");
    let (lean, _) = express_gland(10);
    assert_eq!(lean[SECRETE], 1, "{lean:?}");
    assert_eq!(lean[FIX], 7);
}

#[test]
fn a_slot_read_twice_in_one_act_is_one_draw() {
    let twice = |a: u8, b: u8| {
        Expr::Add(vec![
            Expr::Draw { below: 64, slot: a },
            Expr::Mul(vec![Expr::Const(-1), Expr::Draw { below: 64, slot: b }]),
        ])
    };
    let practise = |id: &str, e: Expr| {
        let effect = Effect::Practice {
            key: format!("skill:{id}"),
            amount: Amount::Computed(Expr::Add(vec![Expr::Const(64), e])),
        };
        process(id, vec![], vec![effect])
    };
    let mut g = world(0);
    for (id, a, b) in [("test:same", 0, 0), ("test:apart", 0, 1)] {
        let p = practise(id, twice(a, b));
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let mut apart = BTreeSet::new();
    for _ in 0..8 {
        assert_eq!(
            sim.execute(1, None, "test:same", None).outcome,
            Outcome::Accepted
        );
        assert_eq!(
            sim.execute(1, None, "test:apart", None).outcome,
            Outcome::Accepted
        );
        let skills = &sim.state().population.get(1).unwrap().skills;
        apart.insert(skills["skill:test:apart"]);
    }
    let skills = &sim.state().population.get(1).unwrap().skills;
    assert_eq!(
        skills["skill:test:same"],
        8 * 64,
        "the slot repeats its draw"
    );
    assert!(apart.len() > 1, "two slots differ: {apart:?}");
}

#[test]
fn a_bound_part_reads_alone_and_only_where_bound() {
    let practise = |id: &str, requires: Vec<Query>, r: Reading| {
        let effect = Effect::Practice {
            key: format!("skill:{id}"),
            amount: Amount::Computed(read(r)),
        };
        process(id, requires, vec![effect])
    };
    let weight = Reading::CellWeight { who: Binding::Part };
    let held = Reading::Cells {
        who: Binding::Part,
        function: FIX.into(),
    };
    let mut g = world(0);
    for p in [
        practise("test:weight", binds(FIX), weight.clone()),
        practise("test:held", binds(FIX), held.clone()),
    ] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
    for id in ["test:weight", "test:held"] {
        assert_eq!(sim.execute(1, None, id, None).outcome, Outcome::Accepted);
    }
    let skills = &sim.state().population.get(1).unwrap().skills;
    assert_eq!(
        (skills["skill:test:weight"], skills["skill:test:held"]),
        (5, 8)
    );
    let refused = |p: Process| {
        let mut g = world(0);
        g.rules.processes.insert(p.id.clone(), p);
        g.validate().is_err()
    };
    assert!(
        refused(practise("test:a", vec![], weight)),
        "no part is bound"
    );
    let body = Reading::CellWeight {
        who: Binding::Actor,
    };
    assert!(
        refused(practise("test:b", binds(FIX), body)),
        "a body has no one weight"
    );
    let ledger = Reading::Account {
        who: Binding::Part,
        key: "world:soil".into(),
    };
    assert!(
        refused(practise("test:c", binds(FIX), ledger)),
        "a part keeps no ledger"
    );
    assert!(
        !refused(practise("test:d", binds(FIX), held)),
        "the control"
    );
}

#[test]
fn a_development_places_only_an_acquired_function_its_shape_admits() {
    let allot = |id: &str, bound: &str, to: &str| {
        process(
            id,
            binds(bound),
            vec![Effect::Allocate {
                from: Some(bound.into()),
                to: to.into(),
                cells: 1.into(),
            }],
        )
    };
    let mut g = world(0);
    // A function a sheet admits that grows: only its seeding can refuse it.
    let grown = Function {
        shapes: BTreeSet::from(["part-shape:sheet".into()]),
        seeding: Seeding::Grown,
    };
    g.rules
        .functions
        .insert("function:test-grown".into(), grown);
    for p in [
        allot("test:secrete", FIX, SECRETE),
        allot("test:grown", FIX, "function:test-grown"),
        allot("test:lump", "function:intake", SECRETE),
    ] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    assert_eq!(
        sim.execute(1, None, "test:secrete", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(cells(&sim, 0)[SECRETE], 1);
    let refused = |sim: &mut Simulation, id: &str| {
        sim.execute(1, None, id, None).outcome != Outcome::Accepted
    };
    assert!(
        refused(&mut sim, "test:grown"),
        "a grown function is not placed"
    );
    assert!(refused(&mut sim, "test:lump"), "a lump cannot secrete");
}
