// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{
    Command, Execution, Founding, Session,
    ecology::pocket::{CONTACT, LEAVE, SETTLE, opening},
    schema::Relation,
};

#[test]
fn a_seat_and_resident_each_belong_to_one_live_contact() {
    let mut founding: Founding = toml::from_str(include_str!("../examples/pocket.toml")).unwrap();
    founding.population = 6;
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut session = Session::new(founding.generate().unwrap(), mode).unwrap();
        let ids = |line: &str| {
            session
                .sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, g)| g.entity.lineage == line)
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        };
        let h = ids("lineage:2");
        let r = ids("lineage:0");
        assert_eq!((h.len(), r.len()), (2, 2));
        let host = session.sim.state().population.get(h[0]).unwrap();
        let [_, wall, _] = opening(host, 1).expect("two living exchange walls");
        let mut damaged = host.clone();
        damaged.parts.remove(&wall);
        assert!(opening(&damaged, 1).is_none());
        let act = |s: &mut Session, host, resident, key: &str, expected| {
            let _ = s.command(Command::Act {
                actor: host,
                target: Some(resident),
                process: key.into(),
                cause: None,
            });
            assert_eq!(
                s.sim
                    .state()
                    .relations
                    .contains(&Relation::new(host, CONTACT, resident)),
                expected
            );
        };
        assert!(
            session
                .sim
                .state()
                .relations
                .contains(&Relation::new(h[0], CONTACT, r[0]))
        );
        act(&mut session, h[0], r[1], SETTLE, false);
        act(&mut session, h[1], r[0], SETTLE, false);
        act(&mut session, h[0], r[0], LEAVE, false);
        act(&mut session, h[1], r[0], SETTLE, true);
        session.command(Command::Kill { entity: h[1] }).unwrap();
        act(&mut session, h[0], r[0], SETTLE, true);
    }
}

#[test]
fn one_seat_does_not_trade_on_behalf_of_its_founder_cohort() {
    let mut founding: Founding = toml::from_str(include_str!("../examples/pocket.toml")).unwrap();
    founding.population = 6;
    founding.cohort_size = 2;
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut session = Session::new(founding.generate().unwrap(), mode).unwrap();
        let contacts: Vec<_> = session
            .sim
            .state()
            .relations
            .iter()
            .filter(|r| r.kind == CONTACT)
            .cloned()
            .collect();
        assert_eq!(contacts.len(), 1);
        let pair = &contacts[0];
        let mut observed = (0, 0);
        for _ in 0..3 {
            for f in session.advance_tick_with_flows().unwrap().flows {
                if !f.by_process(cosmograph::ecology::pocket::TRADE) {
                    continue;
                }
                assert_eq!(f.count, 1);
                if let (Some(from), Some(to)) = (f.from.0.body(), f.to.0.body()) {
                    if f.to.1 == "world:soil" {
                        assert_eq!((from, to), (pair.subject, pair.object));
                        observed.0 += f.amount;
                    } else {
                        assert_eq!((from, to), (pair.object, pair.subject));
                        observed.1 += f.amount;
                    }
                }
            }
        }
        assert_eq!(observed, (4, 3));
    }
}

#[test]
fn a_replacement_resident_gets_its_own_deliveries() {
    let mut founding: Founding = toml::from_str(include_str!("../examples/pocket.toml")).unwrap();
    founding.population = 6;
    founding.cohort_size = 2;
    let mut s = Session::new(founding.generate().unwrap(), Execution::Grouped).unwrap();
    let pair = s
        .sim
        .state()
        .relations
        .iter()
        .find(|r| r.kind == CONTACT)
        .unwrap()
        .clone();
    let replacement = pair.object + 1;
    for (process, guest) in [(LEAVE, pair.object), (SETTLE, replacement)] {
        s.command(Command::Act {
            actor: pair.subject,
            target: Some(guest),
            process: process.into(),
            cause: None,
        })
        .unwrap();
    }
    assert!(
        s.sim
            .state()
            .relations
            .contains(&Relation::new(pair.subject, CONTACT, replacement))
    );
    let mut soil = 0;
    for _ in 0..3 {
        for f in s.advance_tick_with_flows().unwrap().flows {
            if f.by_process(cosmograph::ecology::pocket::TRADE)
                && f.from.0.body() == Some(pair.subject)
                && f.to.1 == "world:soil"
                && f.to.0.body().is_some()
            {
                assert_eq!(f.to.0.body(), Some(replacement));
                soil += f.count * f.amount;
            }
        }
    }
    assert_eq!(soil, 4);
}
