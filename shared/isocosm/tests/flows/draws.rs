// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seeded draws over Part B's steps 2 and 3: ecology worlds whose own
//! transforms declare their conversions, with a consumer grazing a producer
//! and digesting what it took, and cohorts respiring into the ground at once;
//! advances, dev placements and host acts in a drawn order, run individually
//! and grouped. Every step reconciles every ledger with the record, the
//! world holds what it was founded with plus what the dev source issued,
//! and both modes claim the same moves and reach the same world.

use super::*;

const STEPS: u64 = 60;

fn process(id: &str, requires: Vec<Query>, effects: Vec<Effect>, period: Tick) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Transition,
        requires,
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: Some(period),
        priority: 3,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

fn own(i: u32, at_least: u64) -> Vec<Query> {
    vec![
        Query::Alive(Binding::Actor),
        Query::Trait {
            who: Binding::Actor,
            key: format!("ability:cycle-{i}"),
        },
        Query::Account {
            who: Binding::Actor,
            key: format!("matter:{i}-0"),
            at_least,
        },
    ]
}

/// The generated ecology, its producing a synthesis and its upkeep a
/// mineralization; lineage 1 grazes lineage 0 and digests it; every lineage
/// respires, cohorts together.
pub(super) fn declared(seed: u64) -> Genesis {
    let mut g = ecology(seed);
    for p in g.rules.processes.values_mut() {
        let kind = match p.id.split('-').next() {
            Some("ecology:produce") => Some(Conversion::Synthesis),
            Some("ecology:upkeep") => Some(Conversion::Mineralization),
            _ => None,
        };
        for e in &mut p.effects {
            if let Effect::Transform { conversion, .. } = e {
                *conversion = kind;
            }
        }
    }
    let mut graze = process(
        "test:graze",
        own(1, 0),
        vec![
            Effect::Transfer {
                from: Binding::Target,
                to: Binding::Actor,
                account: "matter:0-0".into(),
                amount: 1,
            },
            Effect::Transform {
                who: Binding::Actor,
                take: Ledger::from([("matter:0-0".into(), 1)]),
                give: Ledger::from([("matter:1-0".into(), 1)]),
                conversion: Some(Conversion::Digestion),
            },
        ],
        4,
    );
    graze.causation = Causation::Choice;
    graze.requires.push(Query::Account {
        who: Binding::Target,
        key: "matter:0-0".into(),
        at_least: 1,
    });
    graze.target = Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: Some("lineage:0".into()),
        among: BTreeSet::new(),
        weighted: false,
    });
    g.rules.processes.insert(graze.id.clone(), graze);
    for i in 0..3 {
        let respire = Effect::Transform {
            who: Binding::Actor,
            take: Ledger::from([(format!("matter:{i}-0"), 1)]),
            give: Ledger::from([("world:soil".into(), 1)]),
            conversion: Some(Conversion::Mineralization),
        };
        let p = process(&format!("test:respire-{i}"), own(i, 3), vec![respire], 5);
        g.rules.processes.insert(p.id.clone(), p);
    }
    g
}

/// One drawn step: an advance, a placement or a host act.
fn command(seed: u64, step: u64, session: &Session) -> Option<Command> {
    let pick = |domain: &str, n: u64| isocosm::draw(seed, domain, &[step]) % n.max(1);
    let roll = pick("roll", 100);
    if roll < 60 {
        return None;
    }
    if roll < 72 {
        let accounts = ["world:soil", "matter:0-0", "matter:1-0", "matter:2-0"];
        return Some(Command::PlaceMatter {
            site: pick("site", 3),
            account: accounts[pick("account", 4) as usize].into(),
            amount: 1 + pick("amount", 50),
        });
    }
    let next = session.sim.state().population.next_id;
    let processes: Vec<&Key> = session.sim.genesis().rules.processes.keys().collect();
    let actor = pick("actor", next);
    let target = (pick("targeted", 2) == 0).then(|| pick("target", next));
    let process = processes[pick("process", processes.len() as u64) as usize];
    Some(act(actor, target, process))
}

/// A draw run in one mode: each step's state hash and the moves its record
/// claims, and every flow it recorded.
fn run(seed: u64, mode: Execution) -> (Vec<(Key, Books)>, Vec<Flow>) {
    let mut session = kept(declared(seed), mode);
    let founded = session.sim.matter();
    let (mut steps, mut all) = (vec![], vec![]);
    for step in 0..STEPS {
        let at = format!("at step {step} of seed {seed} ({mode:?})");
        let drawn = command(seed, step, &session);
        let ticks = 1 + isocosm::draw(seed, "ticks", &[step]) % 3;
        let (_, flows) = stepped(&mut session, &at, |s| match drawn {
            None => s.advance(ticks).map(|_| ()),
            Some(c) => s.command(c).map(|_| ()),
        });
        let issued = session.sim.issued();
        assert_eq!(session.sim.matter(), founded + issued, "{at}");
        steps.push((session.sim.state_hash(), claimed(&flows)));
        all.extend(flows);
    }
    (steps, all)
}

#[test]
fn draws_reconcile_every_ledger_in_both_modes() {
    let mut made = BTreeSet::new();
    let mut cohorts = 0;
    for seed in 345_000..345_008 {
        let (one, flows) = run(seed, Execution::Individuals);
        let (all, grouped) = run(seed, Execution::Grouped);
        for (step, (a, b)) in one.iter().zip(&all).enumerate() {
            assert_eq!(a, b, "seed {seed}, step {step}");
        }
        made.extend(
            made_by(&flows)
                .into_iter()
                .map(|p| p.split('-').next().unwrap().to_string()),
        );
        cohorts += grouped.iter().filter(|f| f.count > 1).count();
    }
    let kinds = [
        "ecology:produce",
        "ecology:upkeep",
        "ecology:feed",
        "ecology:birth",
        "test:graze",
        "test:respire",
        "command PlaceMatter",
    ];
    for kind in kinds {
        assert!(made.contains(kind), "{kind} never moved matter: {made:?}");
    }
    assert!(cohorts > 0, "no cohort moved as one");
}

#[test]
fn a_draw_missing_one_move_does_not_reconcile() {
    // The draws' positive control: the same step with one move dropped.
    let mut session = kept(declared(345_000), Execution::Grouped);
    let (before, flows) = loop {
        let before = books(&session);
        session.advance(1).unwrap();
        let flows = session.sim.take_flows();
        if flows.len() > 1 {
            break (before, flows);
        }
    };
    let after = books(&session);
    reconcile(&before, &after, &flows, "whole").unwrap();
    for skip in 0..flows.len() {
        let mut missing = flows.clone();
        missing.remove(skip);
        assert!(reconcile(&before, &after, &missing, "missing").is_err());
    }
}
