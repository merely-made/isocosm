// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's X5, ordered take and X6: an effect applied only where its
//! guard comes to something, upkeep paid from the reserve before the tissue
//! (ruling 446), and the bound part's cells moved between its functions.

use isocosm::{
    Execution, Founding, Session, Simulation,
    flows::Holder,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const RESERVE: &str = "test:reserve";
const TISSUE: &str = "test:tissue";
const SOIL: &str = "world:soil";

/// One critter, id 1, at site 0: a reserve of 3 and tissue of 10, and a
/// sheet that fixes and secretes, its four cells all allotted.
fn world() -> Genesis {
    let mut g = Founding {
        seed: 6,
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
    let critter = g.population.lift(1).unwrap();
    let lineage = critter.lineage.clone();
    critter.accounts = BTreeMap::from([(RESERVE.into(), 3), (TISSUE.into(), 10)]);
    let sheet = isocosm::geometry::Sketch {
        shape: "part-shape:sheet".into(),
        part: Part {
            functions: ["function:fix", "function:secrete"]
                .map(String::from)
                .into(),
            ..Default::default()
        },
        ..Default::default()
    };
    critter.embody(isocosm::geometry::Body::sketch([(0, sheet)]));
    for (key, reserve) in [(RESERVE, true), (TISSUE, false)] {
        let kind = AccountKind::Matter {
            lineage: lineage.clone(),
            reserve,
            provision: false,
        };
        g.rules.accounts.insert(key.into(), kind);
    }
    for site in g.sites.values_mut() {
        site.accounts.insert(SOIL.into(), 10);
    }
    g
}

fn act(id: &str, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
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

fn with(g: &mut Genesis, p: Process) {
    g.rules.processes.insert(p.id.clone(), p);
}

fn held(sim: &Simulation, key: &str) -> u64 {
    let body = sim.state().population.get(1).unwrap();
    body.accounts.get(key).copied().unwrap_or(0)
}

fn site(sim: &Simulation, key: &str) -> u64 {
    sim.state().sites[&0]
        .accounts
        .get(key)
        .copied()
        .unwrap_or(0)
}

fn take_soil() -> Effect {
    Effect::Transfer {
        from: Binding::Place,
        to: Binding::Actor,
        account: SOIL.into(),
        amount: 2.into(),
    }
}

fn site_holds_at_least(n: i64) -> Expr {
    let soil = Reading::Account {
        who: Binding::Place,
        key: SOIL.into(),
    };
    Expr::AtLeast(Box::new(Expr::Read(soil)), Box::new(Expr::Const(n)))
}

#[test]
fn a_guarded_effect_applies_only_where_its_guard_comes_to_something() {
    let mut g = world();
    let guarded = |n| Effect::When {
        guard: site_holds_at_least(n),
        then: vec![take_soil()],
        otherwise: vec![],
    };
    with(&mut g, act("test:rich", vec![], vec![guarded(5)]));
    with(&mut g, act("test:poor", vec![], vec![guarded(50)]));
    with(&mut g, act("test:plain", vec![], vec![take_soil()]));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    assert_eq!(
        sim.execute(1, None, "test:poor", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(site(&sim, SOIL), 10, "the guard came to nothing");
    assert_eq!(
        sim.execute(1, None, "test:rich", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(site(&sim, SOIL), 8);
    assert_eq!(
        sim.execute(1, None, "test:plain", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(site(&sim, SOIL), 6, "the control, unguarded");
    let p = act(
        "test:any",
        vec![],
        vec![Effect::When {
            guard: Expr::Draw { below: 2, slot: 0 },
            then: vec![take_soil()],
            otherwise: vec![],
        }],
    );
    assert!(p.effects[0].draws() && !p.bulk_safe());
}

#[test]
fn a_guard_is_read_once_for_both_its_branches() {
    // Soil taken while the site holds eight or more, a skill practised
    // otherwise: taking it leaves less than eight, yet the other branch
    // does not then run, the guard having been read before either.
    let practise = Effect::Practice {
        key: "skill:test:else".into(),
        amount: 1.into(),
    };
    let either = Effect::When {
        guard: site_holds_at_least(8),
        then: vec![take_soil()],
        otherwise: vec![practise],
    };
    let mut g = world();
    with(&mut g, act("test:either", vec![], vec![either]));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let skill = |sim: &Simulation| {
        let body = sim.state().population.get(1).unwrap();
        body.skills.get("skill:test:else").copied().unwrap_or(0)
    };
    let run = |sim: &mut Simulation| sim.execute(1, None, "test:either", None).outcome;
    assert_eq!(run(&mut sim), Outcome::Accepted);
    assert_eq!(
        (site(&sim, SOIL), skill(&sim)),
        (8, 0),
        "one branch, not both"
    );
    assert_eq!(run(&mut sim), Outcome::Accepted);
    assert_eq!((site(&sim, SOIL), skill(&sim)), (6, 0));
    // The control: below eight, the other branch runs instead.
    assert_eq!(run(&mut sim), Outcome::Accepted);
    assert_eq!((site(&sim, SOIL), skill(&sim)), (6, 1));
}

#[test]
fn a_kept_value_is_read_as_it_was_kept() {
    // The site's soil kept, then two taken, then the kept value practised:
    // ten, as it was kept, not the eight left.
    let soil = Expr::Read(Reading::Account {
        who: Binding::Place,
        key: SOIL.into(),
    });
    let keep = Effect::Keep {
        name: "test:soil".into(),
        value: soil.clone(),
    };
    let kept = Expr::Read(Reading::Kept {
        name: "test:soil".into(),
    });
    let practise = |x: Expr| Effect::Practice {
        key: "skill:test:kept".into(),
        amount: Amount::Computed(x),
    };
    let mut g = world();
    let effects = vec![keep.clone(), take_soil(), practise(kept.clone())];
    with(&mut g, act("test:keep", vec![], effects));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    assert_eq!(
        sim.execute(1, None, "test:keep", None).outcome,
        Outcome::Accepted
    );
    let body = sim.state().population.get(1).unwrap();
    assert_eq!((site(&sim, SOIL), body.skills["skill:test:kept"]), (8, 10));
    // Read before it is kept, or kept within a guard: refused.
    let refused = |effects: Vec<Effect>| {
        let mut g = world();
        with(&mut g, act("test:x", vec![], effects));
        g.validate().is_err()
    };
    assert!(refused(vec![practise(kept.clone()), keep.clone()]));
    let within = Effect::When {
        guard: Expr::Const(1),
        then: vec![keep.clone()],
        otherwise: vec![],
    };
    assert!(refused(vec![within, practise(kept.clone())]));
    assert!(!refused(vec![keep, practise(kept)]), "the control passes");
}

#[test]
fn upkeep_drains_the_reserve_before_the_tissue() {
    let spend = |amount: u64| Effect::Spend {
        from: vec![RESERVE.into(), TISSUE.into()],
        to: Binding::Place,
        amount: amount.into(),
        into: None,
    };
    let mut g = world();
    with(&mut g, act("test:upkeep", vec![], vec![spend(5)]));
    with(&mut g, act("test:starve", vec![], vec![spend(20)]));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let r = sim.execute(1, None, "test:upkeep", None);
    assert_eq!(r.outcome, Outcome::Accepted);
    assert_eq!((held(&sim, RESERVE), held(&sim, TISSUE)), (0, 8));
    assert_eq!((site(&sim, RESERVE), site(&sim, TISSUE)), (3, 2));
    assert_eq!(r.matter_before, r.matter_after);
    // Owing more than it holds, a body gives all it has and is not refused.
    let r = sim.execute(1, None, "test:starve", None);
    assert_eq!(r.outcome, Outcome::Accepted);
    assert_eq!((held(&sim, RESERVE), held(&sim, TISSUE)), (0, 0));
    assert_eq!((site(&sim, RESERVE), site(&sim, TISSUE)), (3, 10));
}

#[test]
fn an_ordered_take_records_each_share_as_its_own_move() {
    let mut g = world();
    let mut p = act(
        "test:upkeep",
        vec![],
        vec![Effect::Spend {
            from: vec![RESERVE.into(), TISSUE.into()],
            to: Binding::Place,
            amount: 5.into(),
            into: None,
        }],
    );
    p.period = Some(1);
    p.requires.push(Query::Account {
        who: Binding::Actor,
        key: TISSUE.into(),
        at_least: 1,
    });
    with(&mut g, p);
    let mut session = Session::new(g, Execution::Individuals).unwrap();
    let moved = session.advance_tick_with_flows().unwrap().flows;
    let legs: Vec<_> = moved
        .iter()
        .map(|f| (f.from.clone(), f.to.clone(), f.amount))
        .collect();
    let (body, ground) = (Holder::Entity(1), Holder::Site(0));
    assert_eq!(
        legs,
        [
            ((body, RESERVE.into()), (ground, RESERVE.into()), 3),
            ((body, TISSUE.into()), (ground, TISSUE.into()), 2),
        ]
    );
}

#[test]
fn allocation_moves_the_bound_parts_cells_within_its_capacity() {
    let fix = || {
        vec![Query::Expresses {
            function: "function:fix".into(),
        }]
    };
    let allot = |from: Option<&str>, cells: u64| Effect::Allocate {
        from: from.map(String::from),
        to: "function:fix".into(),
        cells: cells.into(),
    };
    // A bodied sheet of four cells (ruling 460), its tissue in it and no
    // reserve, since it stores nothing (rulings 463 and 504).
    let mut g = world();
    let critter = g.population.lift(1).unwrap();
    critter.body.as_mut().unwrap().parts[0].half_extent = [3, 3, 1];
    let sheet = critter.parts.get_mut(&PartId(0)).unwrap();
    sheet.cells = BTreeMap::from([("function:fix".into(), 1), ("function:secrete".into(), 3)]);
    sheet.matter = BTreeMap::from([(TISSUE.into(), 10)]);
    critter.accounts.clear();
    with(
        &mut g,
        act(
            "test:shift",
            fix(),
            vec![allot(Some("function:secrete"), 2)],
        ),
    );
    with(
        &mut g,
        act("test:last", fix(), vec![allot(Some("function:secrete"), 1)]),
    );
    with(&mut g, act("test:free", fix(), vec![allot(None, 1)]));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let cells = |sim: &Simulation| {
        sim.state().population.get(1).unwrap().parts[&PartId(0)]
            .cells
            .clone()
    };
    let revision = sim.state().population.get(1).unwrap().body_revision;
    assert_eq!(
        sim.execute(1, None, "test:shift", None).outcome,
        Outcome::Accepted
    );
    let expected = BTreeMap::from([("function:fix".into(), 3), ("function:secrete".into(), 1)]);
    assert_eq!(cells(&sim), expected);
    assert_eq!(
        sim.state().population.get(1).unwrap().body_revision,
        revision + 1
    );
    // Every cell is allotted: none is free, and the act is refused whole.
    assert!(sim.execute(1, None, "test:free", None).outcome != Outcome::Accepted);
    assert_eq!(cells(&sim), expected);
    // More cells than a function holds cannot move.
    assert!(sim.execute(1, None, "test:shift", None).outcome != Outcome::Accepted);
    // Moving its last cell empties the function's entry.
    assert_eq!(
        sim.execute(1, None, "test:last", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(cells(&sim), BTreeMap::from([("function:fix".into(), 4)]));
}

#[test]
fn rules_refuse_what_these_effects_cannot_do() {
    let refused = |p: Process| {
        let mut g = world();
        with(&mut g, p);
        g.validate().is_err()
    };
    let soil = || take_soil();
    let nested = Effect::When {
        guard: Expr::Const(1),
        then: vec![],
        otherwise: vec![Effect::When {
            guard: Expr::Const(1),
            then: vec![soil()],
            otherwise: vec![],
        }],
    };
    assert!(
        refused(act("test:a", vec![], vec![nested])),
        "a guard guarding a guard"
    );
    let spend = |from: Vec<Key>, to| Effect::Spend {
        from,
        to,
        amount: 1.into(),
        into: None,
    };
    assert!(refused(act(
        "test:b",
        vec![],
        vec![spend(vec![], Binding::Place)]
    )));
    assert!(refused(act(
        "test:c",
        vec![],
        vec![spend(vec![RESERVE.into()], Binding::Actor)]
    )));
    let allot = |to: &str| Effect::Allocate {
        from: None,
        to: to.into(),
        cells: 1.into(),
    };
    assert!(
        refused(act("test:d", vec![], vec![allot("function:fix")])),
        "binds no part"
    );
    let fix = vec![Query::Expresses {
        function: "function:fix".into(),
    }];
    assert!(refused(act(
        "test:e",
        fix.clone(),
        vec![allot("function:absent")]
    )));
    assert!(
        !refused(act("test:f", fix, vec![allot("function:fix")])),
        "the control"
    );
}
