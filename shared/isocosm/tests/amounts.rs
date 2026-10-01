// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's X3: computed amounts as bounded expression trees (ruling
//! 268), resolved by each act before it stages anything.

use isocosm::{
    Execution, Founding, Session, Simulation, rules::*, schema::Id, simulation::Outcome,
};
use std::collections::BTreeSet;

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

fn soil(who: Binding) -> Reading {
    Reading::Account {
        who,
        key: "world:soil".into(),
    }
}

fn due(id: &str, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: Some(1),
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

fn eval(e: &Expr, held: i64) -> i64 {
    e.eval(&mut |_| Ok(held), &mut |below| Ok(below - 1))
        .unwrap()
}

#[test]
fn a_fixed_amount_serializes_as_a_bare_number() {
    let e = Effect::Ease {
        who: Binding::Actor,
        key: "mind:strain".into(),
        amount: 3.into(),
    };
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"amount\":3"), "{json}");
    let back: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(back, e);
    let computed = Amount::Computed(Expr::linear(1, 2, soil(Binding::Actor), 0, 9));
    let json = serde_json::to_string(&computed).unwrap();
    assert_eq!(serde_json::from_str::<Amount>(&json).unwrap(), computed);
}

#[test]
fn an_expression_keeps_its_bounds() {
    let ok = Expr::linear_drawn(1, 2, soil(Binding::Actor), MAX_DRAW, 0, 9);
    ok.validate().unwrap();
    let wide = Expr::Add((0..MAX_NODES as i64).map(Expr::Const).collect());
    assert!(wide.validate().is_err(), "more than {MAX_NODES} nodes");
    for below in [0, MAX_DRAW + 1] {
        assert!(Expr::Draw { below }.validate().is_err(), "{below}");
    }
    let clamp = Expr::Clamp {
        value: Box::new(Expr::Const(1)),
        lo: 2,
        hi: 1,
    };
    assert!(clamp.validate().is_err());
    assert!(Expr::Min(vec![]).validate().is_err());
}

#[test]
fn an_expression_saturates_and_floors() {
    assert_eq!(eval(&Expr::linear(1, 2, soil(Binding::Actor), 0, 9), 3), 7);
    assert_eq!(eval(&Expr::linear(1, 2, soil(Binding::Actor), 0, 9), 30), 9);
    let half = |a: i64, b: i64| Expr::Div(Box::new(Expr::Const(a)), Box::new(Expr::Const(b)));
    assert_eq!(eval(&half(7, 0), 0), 0, "a zero divisor gives zero");
    assert_eq!(eval(&half(-7, 2), 0), -4, "floor, not truncation");
    let big = Expr::Mul(vec![Expr::Const(i64::MAX), Expr::Const(2)]);
    assert_eq!(eval(&big, 0), i64::MAX);
    let draws = Expr::linear_drawn(0, 0, soil(Binding::Actor), 4, 0, 9);
    assert_eq!(eval(&draws, 0), 3, "the draw's highest value");
}

#[test]
fn an_act_moves_what_its_amount_comes_to_and_never_less_than_nothing() {
    let mut genesis = founding(1).generate().unwrap();
    for site in genesis.sites.values_mut() {
        site.accounts.insert("world:soil".into(), 10);
    }
    let half_the_site = Expr::Div(
        Box::new(Expr::Read(soil(Binding::Place))),
        Box::new(Expr::Const(2)),
    );
    let take = |amount: Expr| Effect::Transfer {
        from: Binding::Place,
        to: Binding::Actor,
        account: "world:soil".into(),
        amount: Amount::Computed(amount),
    };
    genesis.rules.processes.insert(
        "test:half".into(),
        due("test:half", vec![take(half_the_site)]),
    );
    genesis.rules.processes.insert(
        "test:less".into(),
        due("test:less", vec![take(Expr::Const(-5))]),
    );
    let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
    let (actor, place) = (3, sim.state().population.get(3).unwrap().place);
    let held = sim.state().population.get(actor).unwrap().accounts["world:soil"];
    let r = sim.execute(actor, None, "test:half", None);
    assert_eq!(r.outcome, Outcome::Accepted);
    let body = sim.state().population.get(actor).unwrap();
    assert_eq!(body.accounts["world:soil"], held + 5);
    assert_eq!(sim.state().sites[&place].accounts["world:soil"], 5);
    // A negative amount resolves to nothing: accepted, nothing moves.
    let r = sim.execute(actor, None, "test:less", None);
    assert_eq!(r.outcome, Outcome::Accepted);
    assert_eq!(sim.state().sites[&place].accounts["world:soil"], 5);
}

#[test]
fn rules_refuse_an_amount_that_reads_what_its_act_does_not_bind() {
    let base = founding(2).generate().unwrap();
    let refused = |reading: Reading| {
        let mut g = base.clone();
        let amount = Amount::Computed(Expr::Read(reading));
        let effect = Effect::Practice {
            key: "skill:test".into(),
            amount,
        };
        g.rules
            .processes
            .insert("test:read".into(), due("test:read", vec![effect]));
        g.validate().is_err()
    };
    assert!(refused(soil(Binding::Part)), "a part keeps no ledger");
    assert!(refused(soil(Binding::Target)), "no target is bound");
    assert!(refused(Reading::Account {
        who: Binding::Actor,
        key: "test:absent".into(),
    }));
    assert!(!refused(soil(Binding::Actor)), "the control is accepted");
}

#[test]
fn draws_key_by_the_act_and_stay_in_range() {
    let draws = |seed: u64| -> Vec<u64> {
        let mut genesis = founding(seed).generate().unwrap();
        for site in genesis.sites.values_mut() {
            site.accounts.insert("world:soil".into(), 1_000);
        }
        let effect = Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: "world:soil".into(),
            amount: Amount::Computed(Expr::Draw { below: 4 }),
        };
        genesis
            .rules
            .processes
            .insert("test:draw".into(), due("test:draw", vec![effect]));
        let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
        let held = |sim: &Simulation, actor: Id| {
            let body = sim.state().population.get(actor).unwrap();
            body.accounts.get("world:soil").copied().unwrap_or(0)
        };
        // The members, after the world's own body at each of three sites.
        (3..27)
            .map(|actor| {
                let before = held(&sim, actor);
                let r = sim.execute(actor, None, "test:draw", None);
                assert_eq!(r.outcome, Outcome::Accepted);
                held(&sim, actor) - before
            })
            .collect()
    };
    let first = draws(3);
    assert_eq!(first, draws(3), "the same acts draw alike");
    assert!(first.iter().all(|d| *d < 4));
    let distinct: BTreeSet<_> = first.iter().collect();
    assert!(distinct.len() > 1, "the draws vary: {first:?}");
}

#[test]
fn bulk_safety_follows_the_amounts() {
    let ease = |amount: Amount| {
        due(
            "test:ease",
            vec![Effect::Ease {
                who: Binding::Actor,
                key: "world:soil".into(),
                amount,
            }],
        )
    };
    assert!(ease(1.into()).bulk_safe());
    assert!(ease(Amount::Computed(Expr::Read(soil(Binding::Actor)))).bulk_safe());
    assert!(!ease(Amount::Computed(Expr::Read(soil(Binding::Place)))).bulk_safe());
    assert!(!ease(Amount::Computed(Expr::Draw { below: 2 })).bulk_safe());
}

#[test]
fn both_runners_agree_on_a_computed_amount() {
    let normalized = |s: &Session| {
        let mut state = s.sim.state().clone();
        state.population = state.population.normalized();
        state
    };
    let run = |seed: u64, mode: Execution| {
        let mut genesis = founding(seed).generate().unwrap();
        let effect = Effect::Practice {
            key: "skill:test".into(),
            amount: Amount::Computed(Expr::linear(1, 1, soil(Binding::Actor), 0, 50)),
        };
        let p = due("test:practise", vec![effect]);
        assert!(p.bulk_safe());
        genesis.rules.processes.insert(p.id.clone(), p);
        let mut session = Session::new(genesis, mode).unwrap();
        session.advance(12).unwrap();
        normalized(&session)
    };
    for seed in 0..4 {
        let grouped = run(seed, Execution::Grouped);
        assert!(grouped == run(seed, Execution::Individuals), "seed {seed}");
        assert!(
            grouped != run(seed + 100, Execution::Individuals),
            "the control differs"
        );
    }
}
