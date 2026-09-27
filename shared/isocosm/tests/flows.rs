// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! **The flow record accounts for what the ledgers did** (rulings 270, 345
//! and 359), `mesocosm-core`'s `tests/flows.rs` ported: a matter move cannot
//! show in a ledger while absent from the record. Conservation would still
//! hold if matter moved in silence; this reads the two against each other,
//! for every account of every member and every site.
//!
//! The instrument is proved, not assumed: `reconcile` is shown doctored
//! books and doctored records, and must name the account. Of the source's
//! eight tests, the filially expressed birth is not ported: it reconciles
//! the cost of developing a revised line's tract, and Isocosm has neither
//! development nor its cost yet.

use isocosm::{
    Execution, Founding, Session,
    flows::{Flow, FlowResult, Holder, MadeBy},
    history::Command,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

#[path = "flows/draws.rs"]
mod draws;
#[path = "flows/handoff.rs"]
mod handoff;
#[path = "flows/receipts.rs"]
mod receipts;

/// Matter held, by holder and account; absent is nothing.
type Books = BTreeMap<(Holder, Key), i128>;

/// Every matter account of every member and every site.
fn books(session: &Session) -> Books {
    let (rules, state) = (&session.sim.genesis().rules, session.sim.state());
    let matter = |k: &Key| matches!(rules.accounts.get(k), Some(AccountKind::Matter { .. }));
    let mut books = Books::new();
    let mut hold = |holder: Holder, ledger: &Ledger| {
        for (k, v) in ledger.iter().filter(|(k, v)| matter(k) && **v > 0) {
            books.insert((holder, k.clone()), i128::from(*v));
        }
    };
    for (&first, cohort) in &state.population.groups {
        for id in first..first + cohort.count {
            hold(Holder::Entity(id), &cohort.entity.accounts);
        }
    }
    for (&id, site) in &state.sites {
        hold(Holder::Site(id), &site.accounts);
    }
    books
}

/// What the record says each account did: an entity holder and the members
/// after it each move the amount, a site the amount for each member.
fn claimed(flows: &[Flow]) -> Books {
    let mut claims = Books::new();
    for f in flows {
        let amount = i128::from(f.amount);
        for ((holder, key), sign) in [(&f.from, -1i128), (&f.to, 1)] {
            let mut claim = |holder: Holder, moved: i128| {
                *claims.entry((holder, key.clone())).or_default() += sign * moved;
            };
            match *holder {
                Holder::Entity(first) => {
                    (first..first + f.count).for_each(|id| claim(Holder::Entity(id), amount))
                },
                Holder::Site(_) => claim(*holder, amount * i128::from(f.count)),
                // The dev source holds nothing (ruling 344).
                Holder::Dev => {},
            }
        }
    }
    claims.retain(|_, v| *v != 0);
    claims
}

/// The check itself: `Ok` is silence, `Err` names the account that
/// disagreed. The runs that must pass and the controls that must fail use it
/// alike.
fn reconcile(before: &Books, after: &Books, flows: &[Flow], at: &str) -> Result<(), String> {
    let claims = claimed(flows);
    let accounts: BTreeSet<&(Holder, Key)> = before
        .keys()
        .chain(after.keys())
        .chain(claims.keys())
        .collect();
    for account in accounts {
        let get = |books: &Books| books.get(account).copied().unwrap_or(0);
        let (moved, claim) = (get(after) - get(before), get(&claims));
        if moved != claim {
            return Err(format!(
                "{account:?} moved {moved} {at}; the record accounts for {claim}"
            ));
        }
    }
    Ok(())
}

/// One step, reconciled. Returns what it returned and the flows it made.
fn stepped<T>(
    session: &mut Session,
    at: &str,
    step: impl FnOnce(&mut Session) -> FlowResult<T>,
) -> (T, Vec<Flow>) {
    let before = books(session);
    let FlowResult {
        tick,
        result: out,
        flows,
    } = step(session);
    assert_eq!(tick, session.sim.state().tick);
    assert!(
        flows.iter().all(|flow| flow.tick == tick),
        "mixed ticks {at}"
    );
    if let Err(why) = reconcile(&before, &books(session), &flows, at) {
        panic!("{why}");
    }
    (out, flows)
}

/// A generated ecology: producers, consumers eating them by a weighted draw,
/// decomposers, births, deaths and upkeep returned to the ground.
fn ecology(seed: u64) -> Genesis {
    Founding {
        seed,
        sites: 3,
        population: 60,
        lineages: 3,
        cohort_size: 4,
        ecology: true,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

fn new_session(g: Genesis, mode: Execution) -> Session {
    Session::new(g, mode).unwrap()
}

fn made_by(flows: &[Flow]) -> BTreeSet<String> {
    let name = |f: &Flow| match &f.made_by {
        MadeBy::Process(p) => p.clone(),
        MadeBy::Command(c) => format!("command {c}"),
    };
    flows.iter().map(name).collect()
}

#[test]
fn the_record_accounts_for_every_ledger_across_a_run() {
    // Long enough that every move the ecology makes fires: drawing from the
    // ground and returning to it, eating, births, deaths and upkeep. The two
    // modes run in step, and each tick's record claims the same moves in both.
    let mut seen = BTreeSet::new();
    for seed in [1u64, 7, 4_242] {
        let mut one = new_session(ecology(seed), Execution::Individuals);
        let mut all = new_session(ecology(seed), Execution::Grouped);
        let matter = one.sim.matter();
        for tick in 1..=120 {
            let at = format!("on tick {tick} of seed {seed}");
            let (_, a) = stepped(&mut one, &at, |s| s.advance_tick_with_flows().unwrap());
            let (_, b) = stepped(&mut all, &at, |s| s.advance_tick_with_flows().unwrap());
            assert_eq!(claimed(&a), claimed(&b), "{at}");
            assert_eq!(one.sim.state_hash(), all.sim.state_hash(), "{at}");
            assert_eq!(one.sim.matter(), matter);
            seen.extend(
                made_by(&a)
                    .into_iter()
                    .map(|p| p.split('-').next().unwrap().to_string()),
            );
        }
    }
    let kinds = [
        "ecology:produce",
        "ecology:upkeep",
        "ecology:feed",
        "ecology:birth",
    ];
    for kind in kinds {
        assert!(seen.contains(kind), "{kind} never moved matter: {seen:?}");
    }
}

/// The first living member of `lineage`.
fn member(session: &Session, lineage: &str) -> Id {
    let groups = &session.sim.state().population.groups;
    let found = groups
        .iter()
        .find(|(_, c)| c.entity.alive && c.entity.lineage == lineage);
    *found.expect("a living member of the lineage").0
}

/// A living member of each of two lineages, at one site.
fn neighbours(session: &Session, a: &'static str, b: &'static str) -> (Id, Id) {
    let groups = &session.sim.state().population.groups;
    let living = |l: &'static str| {
        groups
            .iter()
            .filter(move |(_, c)| c.entity.alive && c.entity.lineage == l)
    };
    for (&x, cx) in living(a) {
        let near =
            |(_, cy): &(&Id, &isocosm::population::Cohort)| cy.entity.place == cx.entity.place;
        if let Some((&y, _)) = living(b).find(near) {
            return (x, y);
        }
    }
    panic!("no site holds both lineages")
}

fn act(actor: Id, target: Option<Id>, process: &str) -> Command {
    Command::Act {
        actor,
        target,
        process: process.into(),
        cause: None,
    }
}

#[test]
fn the_record_accounts_for_acts_and_commands_too() {
    let mut session = new_session(ecology(11), Execution::Grouped);
    let (producer, consumer) = neighbours(&session, "lineage:0", "lineage:1");
    let place = Command::PlaceMatter {
        site: 0,
        account: "world:soil".into(),
        amount: 60,
    };
    let trace = [
        act(consumer, Some(producer), "ecology:feed-1"),
        act(producer, Some(consumer), "sim:give"),
        act(producer, None, "ecology:upkeep-0"),
        place,
        act(producer, None, "ecology:produce-0"),
    ];
    let mut moved = 0;
    for (step, command) in trace.into_iter().enumerate() {
        let at = format!("after command {step}");
        let (outcome, flows) = stepped(&mut session, &at, |s| {
            s.command_with_flows(command).unwrap()
        });
        assert!(
            !outcome.contains("Blocked") && !outcome.contains("Refused"),
            "{at}: {outcome}"
        );
        moved += flows.len();
    }
    assert!(moved >= 5, "every command moved matter");
}

#[test]
fn every_flow_is_stamped_with_the_tick_it_happened_on() {
    let mut session = new_session(ecology(7), Execution::Grouped);
    for _ in 0..30 {
        let (_, flows) = stepped(&mut session, "while stamping", |s| {
            s.advance_tick_with_flows().unwrap()
        });
        let tick = session.sim.state().tick;
        assert!(
            flows.iter().all(|f| f.tick == tick),
            "one tick's moves carry it"
        );
    }
}

#[test]
fn requesting_the_record_is_not_a_world_change() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut recorded = new_session(ecology(4_242), mode);
        let mut ordinary = recorded.clone();
        let mut moved = 0;
        for _ in 0..40 {
            let result = recorded.advance_tick_with_flows().unwrap();
            let ordinary_work = ordinary.advance(1).unwrap();
            moved += result.flows.len();
            assert_eq!(recorded.sim.state_hash(), ordinary.sim.state_hash());
            assert_eq!(result.result, ordinary_work);
        }
        assert!(moved > 0, "the comparison exercised real moves");
    }
}

#[test]
fn a_saved_world_does_not_carry_the_record() {
    let mut session = new_session(ecology(4_242), Execution::Grouped);
    let mut unrecorded = session.clone();
    let mut returned = vec![];
    for _ in 0..5 {
        returned.push(session.advance_tick_with_flows().unwrap());
        unrecorded.advance(1).unwrap();
    }
    assert!(returned.iter().any(|r| !r.flows.is_empty()));
    let holding = serde_json::to_string(&session.save()).unwrap();
    drop(returned);
    assert_eq!(serde_json::to_string(&session.save()).unwrap(), holding);
    assert_eq!(serde_json::to_string(&unrecorded.save()).unwrap(), holding);
}

/// **A birth reconciles to the unit.** Every unit of matter the child holds
/// came out of its parent, account by account, and is in the record as such.
#[test]
fn a_birth_reconciles_to_the_unit() {
    let mut g = ecology(11);
    let mut session = Session::new(g.clone(), Execution::Individuals).unwrap();
    let parent = member(&session, "lineage:1");
    g.population
        .lift(parent)
        .unwrap()
        .accounts
        .insert("matter:1-0".into(), 500);
    session = new_session(g, Execution::Individuals);
    let child = session.sim.state().population.next_id;
    let (outcome, flows) = stepped(&mut session, "at the birth", |s| {
        s.command_with_flows(act(parent, None, "ecology:birth-1"))
            .unwrap()
    });
    assert!(outcome.contains("Accepted"), "{outcome}");
    let born = session
        .sim
        .state()
        .population
        .get(child)
        .expect("a child")
        .clone();
    let given: Ledger = flows
        .iter()
        .filter(|f| f.to.0 == Holder::Entity(child))
        .map(|f| {
            assert_eq!(f.made_by, MadeBy::Process("ecology:birth-1".into()));
            assert_eq!(
                f.from,
                (Holder::Entity(parent), f.to.1.clone()),
                "out of the parent"
            );
            (f.to.1.clone(), f.amount)
        })
        .collect();
    assert!(!given.is_empty());
    assert_eq!(
        given, born.accounts,
        "every unit the child holds is in the record"
    );
}

#[test]
fn the_check_catches_a_move_the_record_did_not_make() {
    // The positive control: a mutation the record did not claim.
    let mut session = new_session(ecology(1), Execution::Grouped);
    session.advance(3).unwrap();
    let before = books(&session);
    let flows = session.advance_tick_with_flows().unwrap().flows;
    let honest = books(&session);
    reconcile(&before, &honest, &flows, "on an honest tick").expect("the tick reconciles");
    let mut doctored = honest.clone();
    let account = doctored
        .keys()
        .next()
        .expect("a world holding matter")
        .clone();
    *doctored.get_mut(&account).unwrap() += 1;
    let complaint = reconcile(&before, &doctored, &flows, "after a silent gain")
        .expect_err("an unrecorded unit must not pass");
    assert!(complaint.contains(&format!("{account:?}")), "{complaint}");
}

/// The dev source is a holder that holds nothing: its moves land at a site,
/// and what it has issued is the sum of the moves out of it (ruling 344).
#[test]
fn a_placement_is_in_the_record_and_the_issue_is_the_sum_of_its_moves() {
    let mut session = new_session(ecology(3), Execution::Individuals);
    let placements = [(400, 0), (75, 2)];
    let mut flows = vec![];
    for (amount, site) in placements {
        let place = Command::PlaceMatter {
            site,
            account: "world:soil".into(),
            amount,
        };
        flows.extend(
            stepped(&mut session, "placing", |s| {
                s.command_with_flows(place).unwrap()
            })
            .1,
        );
    }
    for _ in 0..2 {
        flows.extend(
            stepped(&mut session, "after", |s| {
                s.advance_tick_with_flows().unwrap()
            })
            .1,
        );
    }
    let from_dev = flows.iter().filter(|f| f.from.0 == Holder::Dev);
    let issued: u128 = from_dev.clone().map(|f| u128::from(f.amount)).sum();
    assert_eq!(issued, session.sim.issued());
    assert_eq!(issued, 475);
    for f in from_dev {
        assert_eq!(f.made_by, MadeBy::Command("PlaceMatter".into()));
        assert!(matches!(f.to.0, Holder::Site(_)));
    }
}

#[test]
fn an_accepted_act_is_in_the_record_and_a_refused_one_is_not() {
    let mut g = ecology(5);
    let overdraw = Effect::Transfer {
        from: Binding::Actor,
        to: Binding::Place,
        account: "world:soil".into(),
        amount: 1_000_000,
    };
    let give = g.rules.processes["sim:give"].clone();
    let mut greedy = give.clone();
    greedy.id = "test:overdraw".into();
    greedy.target = None;
    greedy.effects = vec![give.effects[0].clone(), overdraw];
    g.rules.processes.insert(greedy.id.clone(), greedy);
    let mut session = new_session(g, Execution::Grouped);
    let (producer, consumer) = neighbours(&session, "lineage:0", "lineage:1");
    let (outcome, flows) = stepped(&mut session, "blocked", |s| {
        s.command_with_flows(act(producer, Some(consumer), "test:overdraw"))
            .unwrap()
    });
    assert!(outcome.contains("Blocked"), "{outcome}");
    assert!(flows.is_empty(), "a blocked act moves nothing");
    let (outcome, flows) = stepped(&mut session, "accepted", |s| {
        s.command_with_flows(act(producer, Some(consumer), "sim:give"))
            .unwrap()
    });
    assert!(outcome.contains("Accepted"), "{outcome}");
    assert_eq!(flows.len(), 1);
    // An advance refused part way is put back, and takes its moves with it.
    let mut tight = ecology(5);
    tight.rules.limits.events_per_advance = 25;
    for process in tight.rules.processes.values_mut() {
        if process.period.is_some() {
            process.period = Some(1);
        }
    }
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = new_session(tight.clone(), mode);
        let hash = session.sim.state_hash();
        assert!(
            session.advance_tick_with_flows().is_err(),
            "the budget refuses the advance"
        );
        assert_eq!(session.sim.state_hash(), hash);
        let result = session
            .command_with_flows(Command::PlaceMatter {
                site: 0,
                account: "world:soil".into(),
                amount: 7,
            })
            .unwrap();
        assert_eq!(result.tick, 0);
        assert_eq!(
            result.flows.len(),
            1,
            "a refused tick leaks no moves into the next call"
        );
        assert_eq!(result.flows[0].amount, 7);
    }
}
