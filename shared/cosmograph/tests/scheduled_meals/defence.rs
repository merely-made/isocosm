// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{
    geometry::{Body, Sketch},
    rules::{Binding, Defence, Effect, Expr, Reading, default_functions, default_shapes},
    schema::Part,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn defence_reads_the_actual_offering_after_an_earlier_authored_transfer() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = world();
        g.rules.functions = default_functions();
        g.rules.shapes = default_shapes();
        g.rules.defence = Some(Defence {
            tissue: BTreeSet::from(["function:store".into()]),
            key: "test:defence".into(),
            threshold: Expr::Read(Reading::Cells {
                who: Binding::Part,
                function: "function:store".into(),
            }),
            layers: vec![],
        });
        let e = g.population.lift(prey).unwrap();
        e.accounts.remove(PREY);
        e.embody(Body::sketch([(
            0,
            Sketch {
                half_extent: [4, 4, 4],
                part: Part {
                    functions: BTreeSet::from(["function:store".into()]),
                    cells: BTreeMap::from([("function:store".into(), 11)]),
                    matter: BTreeMap::from([(PREY.into(), 20)]),
                    ..Part::default()
                },
                ..Sketch::default()
            },
        )]));
        assert!(anatomy::capacity(e.extent(cosmograph::schema::PartId(0))) >= 11);
        g.rules.processes.get_mut(FEED).unwrap().effects = vec![
            Effect::Transfer {
                from: Binding::Target,
                to: Binding::Actor,
                account: PREY.into(),
                amount: 10.into(),
            },
            Effect::Eat {
                from: Binding::Target,
                amount: 20.into(),
                into: BODY.into(),
                of: vec![PREY.into()],
                whole: false,
            },
        ];
        let mut plain = Session::new(g.clone(), mode).unwrap();
        let total = plain.sim.matter();
        let receipt = direct(&mut plain, eater, prey);
        assert!(!receipt.accepted(), "{receipt:?}");
        assert!(
            format!("{:?}", receipt.outcome).contains("final granted bite 10"),
            "{receipt:?}"
        );
        let mut scheduled = Session::new(g.clone(), mode).unwrap();
        scheduled.sim.watch(FEED);
        let work = scheduled.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (0, 1), "{work:?}");
        assert!(
            scheduled.sim.take_watched().is_empty(),
            "a refused act has no accepted watch"
        );
        for s in [&plain, &scheduled] {
            for id in [eater, prey] {
                assert_eq!(s.sim.state().population.get(id), g.population.get(id));
            }
            assert_eq!(s.sim.matter(), total);
            assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
        }

        // Lower only the law's threshold. The same ten units left by the
        // earlier transfer are now sufficient, proving the scheduled
        // refusal above was the defence check rather than another gate.
        g.rules.defence.as_mut().unwrap().threshold = Expr::Min(vec![
            Expr::Read(Reading::Cells {
                who: Binding::Part,
                function: "function:store".into(),
            }),
            Expr::Const(10),
        ]);
        let mut plain = Session::new(g.clone(), mode).unwrap();
        let receipt = direct(&mut plain, eater, prey);
        assert!(
            receipt.accepted(),
            "the ten-unit defence permits this offering: {receipt:?}"
        );
        let mut scheduled = Session::new(g, mode).unwrap();
        scheduled.sim.watch(FEED);
        let work = scheduled.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
        assert_eq!(scheduled.sim.take_watched().len(), 1);
        for s in [&plain, &scheduled] {
            assert_eq!(held(s, prey, PREY), 0);
            assert_eq!(
                held(s, eater, PREY),
                10,
                "the earlier transfer stays distinct"
            );
            assert_eq!(held(s, eater, BODY), 22, "the actual meal credits only ten");
            assert_eq!(s.sim.matter(), total);
            assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
        }
    }
}
