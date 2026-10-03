// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's X2 and minimal body (rulings 446 and 453): native readings
//! of a body's living parts, the part fields they read, and the reserve
//! marked among a lineage's matter accounts; since checkpoint 7, cells and
//! their mass read from extents (ruling 460) and measurements taken by a
//! function's share of the cells (ruling 493).

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

/// One critter, id 1: a live contracting rod, a severed sheet that
/// contracted, a sheet that fixes and secretes, and a sensing point.
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
    // Four cells along the rod; nine in the sheet, each weighing 21 mg.
    let rod = Part {
        cells: BTreeMap::from([("function:contract".into(), 4)]),
        ..shaped("rod", &["function:contract"], [6, -1, 1])
    };
    let severed = Part {
        severed: true,
        ..shaped("sheet", &["function:contract"], [2, 4, 1])
    };
    let sheet = Part {
        cells: BTreeMap::from([("function:fix".into(), 1), ("function:secrete".into(), 3)]),
        ..shaped("sheet", &["function:fix", "function:secrete"], [4, 4, 1])
    };
    let point = shaped("point", &["function:sense"], [1, 1, 1]);
    let critter = g.population.lift(1).unwrap();
    critter.parts = BTreeMap::from([(0, rod), (1, severed), (2, sheet), (3, point)]);
    // Its own matter lives in its parts now that they have bodies (ruling
    // 504): all of it in the rod.
    let lineage = critter.lineage.clone();
    let own: Vec<Key> = critter
        .accounts
        .keys()
        .filter(|k| {
            matches!(g.rules.accounts.get(*k),
                Some(AccountKind::Matter { lineage: l, .. }) if *l == lineage)
        })
        .cloned()
        .collect();
    let critter = g.population.lift(1).unwrap();
    for k in own {
        let v = critter.accounts.remove(&k).unwrap_or(0);
        critter.parts.get_mut(&0).unwrap().matter.insert(k, v);
    }
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
            6,
        ),
        (
            "sight",
            Reading::Span {
                who: actor,
                function: of("sense"),
            },
            1,
        ),
        (
            "voxels",
            Reading::Voxels { who: actor },
            13 * 3 * 3 + 9 * 9 * 3 + 27,
        ),
        (
            "cells",
            Reading::Cells {
                who: actor,
                function: of("contract"),
            },
            4,
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
            3 * 21,
        ),
        // Ruling 493: the sheet's largest face, 81, of which fix holds one
        // cell in nine; its 243 voxels, of which secrete holds three; the
        // rod's length, all of it contracting.
        (
            "area",
            Reading::Measured {
                who: actor,
                function: of("fix"),
                measure: Measure::Area,
            },
            9,
        ),
        (
            "volume",
            Reading::Measured {
                who: actor,
                function: of("secrete"),
                measure: Measure::Volume,
            },
            81,
        ),
        (
            "length",
            Reading::Measured {
                who: actor,
                function: of("contract"),
                measure: Measure::Length,
            },
            13,
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
        // The severed sheet would add 4 to the span and 135 voxels.
        assert_eq!(skills[&format!("skill:{id}")], expected, "{id}");
    }
}

/// A body's ceiling as Mesocosm prices it: each living part's voxels at
/// 100 mg a 125-voxel segment, floored and at least 1 mg (ruling 455).
fn ceiling() -> Expr {
    let voxels = Expr::Read(Reading::Voxels { who: Binding::Part });
    let priced = Expr::Div(
        Box::new(Expr::Mul(vec![voxels, Expr::Const(100)])),
        Box::new(Expr::Const(125)),
    );
    Expr::Parts {
        who: Binding::Actor,
        each: Box::new(Expr::Max(vec![Expr::Const(1), priced])),
    }
}

#[test]
fn a_ceiling_floors_part_by_part_as_mesocosm_does() {
    // A lump, two limbs and an eye of the primitive palette, a speck of one
    // voxel and a severed lump: 100, 64, 64, 21 and 1 mg.
    let parts = [
        (shaped("lump", &["function:intake"], [2, 2, 2]), false),
        (shaped("rod", &["function:contract"], [4, 1, 1]), false),
        (shaped("rod", &["function:contract"], [4, -1, 1]), false),
        (shaped("point", &["function:sense"], [1, 1, 1]), false),
        (shaped("point", &["function:sense"], [0, 0, 0]), false),
        (shaped("lump", &["function:intake"], [2, 2, 2]), true),
    ];
    let mesocosm: u64 = parts
        .iter()
        .filter(|(_, severed)| !severed)
        .map(|(p, _)| {
            let voxels: u64 = p
                .half_extent
                .iter()
                .map(|h| 2 * u64::from(h.unsigned_abs()) + 1)
                .product();
            (voxels * 100 / 125).max(1)
        })
        .sum();
    assert_eq!(mesocosm, 250);
    let mut g = world();
    g.population.lift(1).unwrap().parts = parts
        .into_iter()
        .enumerate()
        .map(|(i, (p, severed))| (i as Id, Part { severed, ..p }))
        .collect();
    let whole = Expr::Div(
        Box::new(Expr::Mul(vec![
            Expr::Read(Reading::Voxels {
                who: Binding::Actor,
            }),
            Expr::Const(100),
        ])),
        Box::new(Expr::Const(125)),
    );
    for (id, e) in [("parts", ceiling()), ("whole", whole)] {
        let mut p = practise(
            id,
            Reading::Voxels {
                who: Binding::Actor,
            },
        );
        p.effects = vec![Effect::Practice {
            key: format!("skill:{id}"),
            amount: Amount::Computed(e),
        }];
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    for id in ["parts", "whole"] {
        let r = sim.execute(1, None, &format!("test:{id}"), None);
        assert_eq!(r.outcome, Outcome::Accepted, "{id}");
    }
    let skills = &sim.state().population.get(1).unwrap().skills;
    // The control: read over the whole body the floors fall once, 252.
    assert_eq!(
        (skills["skill:parts"], skills["skill:whole"]),
        (mesocosm, 252)
    );
}

#[test]
fn rules_refuse_a_sum_over_parts_where_there_are_none() {
    let refused = |e: Expr| {
        let mut g = world();
        let mut p = practise(
            "sum",
            Reading::Voxels {
                who: Binding::Actor,
            },
        );
        p.effects = vec![Effect::Practice {
            key: "skill:sum".into(),
            amount: Amount::Computed(e),
        }];
        g.rules.processes.insert(p.id.clone(), p);
        g.validate().is_err()
    };
    let over = |who, each| Expr::Parts {
        who,
        each: Box::new(each),
    };
    let part = || Expr::Read(Reading::Voxels { who: Binding::Part });
    assert!(refused(over(Binding::Place, part())), "a site has no parts");
    assert!(refused(over(Binding::Target, part())), "no target is bound");
    assert!(
        refused(over(Binding::Actor, over(Binding::Actor, part()))),
        "a sum within a sum"
    );
    assert!(
        refused(Expr::Add(vec![part(), over(Binding::Actor, part())])),
        "a part read outside the sum binds none"
    );
    assert!(!refused(over(Binding::Actor, part())), "the control passes");
}

#[test]
fn three_quarter_power_is_mesocosms() {
    let power = |m: i64| {
        let e = Expr::Sqrt(Box::new(Expr::Mul(vec![
            Expr::Const(m),
            Expr::Sqrt(Box::new(Expr::Const(m))),
        ])));
        e.eval(&mut |_| Ok(0), &mut |_, _| Ok(0)).unwrap()
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
    let eval = |e: Expr| e.eval(&mut |_| Ok(0), &mut |_, _| Ok(0)).unwrap();
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
    for key in ["half_extent", "\"cells\"", "\"matter\"", "\"reserve\""] {
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
        g.validate().err().unwrap_or_default()
    };
    // Extents of one cell by three by one hold three cells (ruling 460);
    // the name is cleared so only the capacity can refuse it.
    let shrunk = refused(&|p| {
        p.half_extent = [1, 4, 1];
        p.shape.clear();
    });
    assert!(shrunk.contains("capacity of 3"), "{shrunk}");
    assert!(
        !refused(&|p| {
            p.cells.insert(of("contract"), 0);
        })
        .is_empty()
    );
}
