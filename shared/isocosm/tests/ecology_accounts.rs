// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{
    Execution, Founding, Session, anatomy,
    bodied::Bodies,
    directing::Played,
    flows::{Flow, Holder},
    history::Command,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

fn world() -> (Genesis, Id, Id) {
    let mut g = Founding {
        seed: 3,
        sites: 1,
        population: 30,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(Bodies::default()),
        played: Some(Played {
            lineage: 1,
            region_sites: 1,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    for p in g.rules.processes.values_mut() {
        p.period = None;
    }
    let member = |line: &str| {
        *g.population
            .groups
            .iter()
            .find(|(_, c)| c.entity.lineage == line)
            .unwrap()
            .0
    };
    let (eater, prey) = (member("lineage:1"), member("lineage:0"));
    (g, eater, prey)
}

fn held(s: &Session, id: Id, key: &str) -> u64 {
    anatomy::held(
        s.sim.state().population.get(id).unwrap(),
        &s.sim.genesis().rules,
        key,
    )
}

type Books = BTreeMap<(Holder, Key), i128>;

fn books(s: &Session) -> Books {
    let mut out = Books::new();
    let mut hold = |holder, ledger: &Ledger| {
        for (key, n) in ledger {
            if matches!(
                s.sim.genesis().rules.accounts.get(key),
                Some(AccountKind::Matter { .. })
            ) {
                out.insert((holder, key.clone()), i128::from(*n));
            }
        }
    };
    for (&id, g) in &s.sim.state().population.groups {
        for member in id..id + g.count {
            hold(Holder::Entity(member), &g.entity.accounts);
            for (&part, p) in &g.entity.parts {
                hold(Holder::Part(member, part), &p.matter);
            }
        }
    }
    for (&id, site) in &s.sim.state().sites {
        hold(Holder::Site(id), &site.accounts);
    }
    out
}

fn reconcile(before: Books, s: &Session, flows: &[Flow]) {
    let after = books(s);
    let mut claimed = Books::new();
    for flow in flows {
        for ((holder, key), sign) in [(&flow.from, -1), (&flow.to, 1)] {
            let mut add =
                |holder, n| *claimed.entry((holder, key.clone())).or_default() += sign * n;
            match *holder {
                Holder::Entity(id) => {
                    for member in id..id + flow.count {
                        add(Holder::Entity(member), i128::from(flow.amount));
                    }
                },
                Holder::Part(id, part) => {
                    for member in id..id + flow.count {
                        add(Holder::Part(member, part), i128::from(flow.amount));
                    }
                },
                Holder::Site(_) => add(*holder, i128::from(flow.amount) * i128::from(flow.count)),
                Holder::Dev => {},
            }
        }
    }
    let keys: BTreeSet<_> = before
        .keys()
        .chain(after.keys())
        .chain(claimed.keys())
        .collect();
    for key in keys {
        let get = |b: &Books| b.get(key).copied().unwrap_or(0);
        assert_eq!(get(&after) - get(&before), get(&claimed), "{key:?}");
    }
    let replay = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), s.sim.state_hash());
}

#[test]
fn unspecified_meals_take_bodied_tissue_and_land_in_parts() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey) = world();
        let mut s = Session::new(g, mode).unwrap();
        let matter = s.sim.matter();
        let before = books(&s);
        let (eaten, has) = (held(&s, prey, "matter:0-0"), held(&s, eater, "matter:1-0"));
        let r = s
            .command_with_flows(Command::Act {
                actor: eater,
                target: Some(prey),
                process: "ecology:feed-1".into(),
                cause: None,
            })
            .unwrap();
        assert!(r.result.contains("Accepted"), "{}", r.result);
        assert_eq!(held(&s, prey, "matter:0-0"), eaten - 1);
        assert_eq!(held(&s, eater, "matter:1-0"), has + 1);
        assert!(
            r.flows
                .iter()
                .any(|f| matches!(f.from.0, Holder::Part(id, _) if id == prey)
                    && matches!(f.to.0, Holder::Part(id, _) if id == eater))
        );
        assert_eq!(
            s.sim
                .state()
                .population
                .get(eater)
                .unwrap()
                .accounts
                .get("matter:1-0"),
            None
        );
        assert_eq!(s.sim.matter(), matter);
        reconcile(before, &s, &r.flows);
    }
}

#[test]
fn scheduled_birth_reads_and_pays_anatomical_accounts_without_cloning_matter() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, parent, _) = world();
        let rules = g.rules.clone();
        anatomy::give(g.population.lift(parent).unwrap(), &rules, "matter:1-0", 20)
            .unwrap()
            .unwrap();
        g.rules.processes.get_mut("ecology:birth-1").unwrap().period = Some(1);
        let child = g.population.next_id;
        let mut s = Session::new(g, mode).unwrap();
        let matter = s.sim.matter();
        let before = books(&s);
        let r = s.advance_tick_with_flows().unwrap();
        let pop = &s.sim.state().population;
        let born = pop.get(child).expect("birth account gate sees its parts");
        assert_eq!(held(&s, child, "matter:1-0"), 8);
        assert_eq!(held(&s, parent, "matter:1-0"), 22);
        assert!(born.accounts.is_empty());
        assert!(
            s.sim
                .state()
                .relations
                .contains(&Relation::new(parent, "sim:child", child))
        );
        assert!(
            s.sim
                .state()
                .relations
                .contains(&Relation::new(child, "sim:parent", parent))
        );
        assert!(
            r.flows
                .iter()
                .any(|f| matches!(f.from.0, Holder::Part(id, _) if id == parent)
                    && matches!(f.to.0, Holder::Part(id, _) if id == child))
        );
        assert_eq!(s.sim.matter(), matter);
        reconcile(before, &s, &r.flows);
    }
}

#[test]
fn scheduled_need_threshold_reads_parts_before_taking_a_meal() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, _, _) = world();
        let p = g.rules.processes.get_mut("ecology:feed-1").unwrap();
        p.period = Some(1);
        p.need_account = Some("matter:1-0".into());
        p.need_below = 10;
        let mut s = Session::new(g, mode).unwrap();
        let before = books(&s);
        let members = s.sim.state().population.count();
        let r = s.advance_tick_with_flows().unwrap();
        assert_eq!(r.result.evaluations, 0);
        assert_eq!(books(&s), before);
        assert_eq!(s.sim.state().population.count(), members);
        assert!(r.flows.is_empty());
    }
}
