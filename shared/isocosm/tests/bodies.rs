// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's X2 and minimal body (rulings 446 and 453): native readings
//! of a body's living parts, the part fields they read, and the reserve
//! marked among a lineage's matter accounts.

use isocosm::{
    Execution, Founding, Simulation,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

fn shaped(shape: &str, functions: &[&str], half_extent: [i32; 3]) -> Part {
    Part {
        shape: format!("part-shape:{shape}"),
        functions: functions.iter().map(|f| f.to_string()).collect(),
        half_extent,
        ..Default::default()
    }
}

/// One critter, id 1: a live contracting rod, the same severed, a sheet that
/// fixes and secretes, and a sensing point.
fn world() -> Genesis {
    let mut g = Founding {
        seed: 453,
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
    let rod = Part {
        capacity: 9,
        cells: BTreeMap::from([("function:contract".into(), 6)]),
        cell_mass: 2,
        ..shaped("rod", &["function:contract"], [3, -1, 1])
    };
    let severed = Part {
        severed: true,
        ..shaped("rod", &["function:contract"], [2, 4, 1])
    };
    let sheet = Part {
        capacity: 4,
        cells: BTreeMap::from([("function:fix".into(), 1), ("function:secrete".into(), 3)]),
        cell_mass: 5,
        ..shaped("sheet", &["function:fix", "function:secrete"], [1, 1, 1])
    };
    let point = shaped("point", &["function:sense"], [0, 0, 2]);
    g.population.lift(1).unwrap().parts =
        BTreeMap::from([(0, rod), (1, severed), (2, sheet), (3, point)]);
    g
}

fn practise(id: &str, reading: Reading) -> Process {
    Process {
        id: format!("test:{id}"),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects: vec![Effect::Practice {
            key: format!("skill:{id}"),
            amount: Amount::Computed(Expr::Read(reading)),
        }],
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

fn of(function: &str) -> Key {
    format!("function:{function}")
}

#[test]
fn a_body_reads_the_sum_over_its_living_parts() {
    let actor = Binding::Actor;
    let readings = [
        (
            "span",
            Reading::Span {
                who: actor,
                function: of("contract"),
            },
            3,
        ),
        (
            "sight",
            Reading::Span {
                who: actor,
                function: of("sense"),
            },
            2,
        ),
        ("voxels", Reading::Voxels { who: actor }, 7 * 3 * 3 + 27 + 5),
        (
            "cells",
            Reading::Cells {
                who: actor,
                function: of("contract"),
            },
            6,
        ),
        (
            "glands",
            Reading::Cells {
                who: actor,
                function: of("secrete"),
            },
            3,
        ),
        (
            "held",
            Reading::CellMass {
                who: actor,
                function: of("secrete"),
            },
            15,
        ),
    ];
    let mut g = world();
    for (id, reading, _) in &readings {
        let p = practise(id, reading.clone());
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    for (id, _, expected) in readings {
        let r = sim.execute(1, None, &format!("test:{id}"), None);
        assert_eq!(r.outcome, Outcome::Accepted, "{id}");
        let skills = &sim.state().population.get(1).unwrap().skills;
        // The severed rod would add 4 to the span and 105 voxels.
        assert_eq!(skills[&format!("skill:{id}")], expected, "{id}");
    }
}

#[test]
fn three_quarter_power_is_mesocosms() {
    let power = |m: i64| {
        let e = Expr::Sqrt(Box::new(Expr::Mul(vec![
            Expr::Const(m),
            Expr::Sqrt(Box::new(Expr::Const(m))),
        ])));
        e.eval(&mut |_| Ok(0), &mut |_| Ok(0)).unwrap()
    };
    for m in [1u64, 7, 1_000, 10_000, 123_456, 9_999_999] {
        let mesocosm = (m * m.isqrt()).isqrt();
        assert_eq!(power(m as i64), mesocosm as i64, "{m}");
    }
    assert_eq!(power(10_000), 1_000);
    assert_eq!(power(-5), 0, "nothing below nothing");
}

#[test]
fn division_floors_and_comparison_is_one_or_zero() {
    let eval = |e: Expr| e.eval(&mut |_| Ok(0), &mut |_| Ok(0)).unwrap();
    let div = |a, b| Expr::Div(Box::new(Expr::Const(a)), Box::new(Expr::Const(b)));
    assert_eq!(eval(div(-7, 2)), -4);
    assert_eq!(eval(div(7, -2)), -4);
    assert_eq!(eval(div(-7, -2)), 3);
    assert_eq!(eval(div(6, -2)), -3);
    assert_eq!(eval(div(i64::MIN, -1)), i64::MAX);
    assert_eq!(eval(div(7, 0)), 0);
    let at_least = |a, b| Expr::AtLeast(Box::new(Expr::Const(a)), Box::new(Expr::Const(b)));
    assert_eq!(eval(at_least(5, 5)), 1);
    assert_eq!(eval(at_least(4, 5)), 0);
}

#[test]
fn rules_refuse_a_body_read_where_there_is_none() {
    let refused = |reading: Reading| {
        let mut g = world();
        let p = practise("read", reading);
        g.rules.processes.insert(p.id.clone(), p);
        g.validate().is_err()
    };
    assert!(
        refused(Reading::Voxels {
            who: Binding::Place
        }),
        "a site has no body"
    );
    assert!(refused(Reading::Span {
        who: Binding::Actor,
        function: of("absent")
    }));
    assert!(refused(Reading::Cells {
        who: Binding::Target,
        function: of("fix")
    }));
    assert!(refused(Reading::Voxels { who: Binding::Part }));
    assert!(
        !refused(Reading::Voxels {
            who: Binding::Actor
        }),
        "the control passes"
    );
}

#[test]
fn a_body_reading_of_the_actor_is_bulk_safe_and_of_another_is_not() {
    let reading = |who| Reading::Span {
        who,
        function: of("contract"),
    };
    assert!(practise("own", reading(Binding::Actor)).bulk_safe());
    assert!(!practise("theirs", reading(Binding::Target)).bulk_safe());
}

#[test]
fn bodies_and_tissue_serialize_as_before() {
    let g = Founding::default().generate().unwrap();
    let json = serde_json::to_string(&g).unwrap();
    for key in [
        "half_extent",
        "capacity",
        "\"cells\"",
        "cell_mass",
        "\"reserve\"",
    ] {
        assert!(!json.contains(key), "{key}");
    }
    let reserve = AccountKind::Matter {
        lineage: "lineage:0".into(),
        reserve: true,
    };
    let back: AccountKind =
        serde_json::from_str(&serde_json::to_string(&reserve).unwrap()).unwrap();
    assert_eq!(back, reserve);
    let part = world().population.get(1).unwrap().parts[&2].clone();
    let back: Part = serde_json::from_str(&serde_json::to_string(&part).unwrap()).unwrap();
    assert_eq!(back, part);
}

#[test]
fn a_part_allots_only_its_own_cells_to_what_it_expresses() {
    assert!(world().validate().is_ok(), "the control passes");
    let refused = |change: &dyn Fn(&mut Part)| {
        let mut g = world();
        change(g.population.lift(1).unwrap().parts.get_mut(&2).unwrap());
        g.validate().is_err()
    };
    assert!(
        refused(&|p| p.capacity = 3),
        "four cells in a capacity of three"
    );
    assert!(refused(&|p| {
        p.cells.insert(of("contract"), 0);
    }));
}
