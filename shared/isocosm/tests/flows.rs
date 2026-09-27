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
    flows::{Flow, Holder, MadeBy},
    history::Command,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

#[path = "flows/draws.rs"]
mod draws;
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
    step: impl FnOnce(&mut Session) -> T,
) -> (T, Vec<Flow>) {
    let before = books(session);
    let out = step(session);
    let flows = session.sim.take_flows();
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

fn kept(g: Genesis, mode: Execution) -> Session {
    let mut session = Session::new(g, mode).unwrap();
    session.sim.keep_flows();
    session
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
        let mut one = kept(ecology(seed), Execution::Individuals);
        let mut all = kept(ecology(seed), Execution::Grouped);
        let matter = one.sim.matter();
        for tick in 1..=120 {
            let at = format!("on tick {tick} of seed {seed}");
            let (_, a) = stepped(&mut one, &at, |s| s.advance(1).unwrap());
            let (_, b) = stepped(&mut all, &at, |s| s.advance(1).unwrap());
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
    let mut session = kept(ecology(11), Execution::Grouped);
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
        let (outcome, flows) = stepped(&mut session, &at, |s| s.command(command).unwrap());
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
    let mut session = kept(ecology(7), Execution::Grouped);
    for _ in 0..30 {
        let (_, flows) = stepped(&mut session, "while stamping", |s| s.advance(1).unwrap());
        let tick = session.sim.state().tick;
        assert!(
            flows.iter().all(|f| f.tick == tick),
            "one tick's moves carry it"
        );
    }
}

#[test]
fn taking_the_record_is_not_a_world_change() {
    let mut drained = kept(ecology(4_242), Execution::Grouped);
    let mut never = Session::new(ecology(4_242), Execution::Grouped).unwrap();
    let mut took = 0;
    for _ in 0..40 {
        drained.advance(1).unwrap();
        never.advance(1).unwrap();
        took += drained.sim.take_flows().len();
        assert_eq!(drained.sim.state_hash(), never.sim.state_hash());
    }
    assert!(took > 0, "there was something to take");
    assert!(never.sim.take_flows().is_empty(), "nothing is kept unasked");
    let before = drained.sim.state_hash();
    drained.sim.take_flows();
    assert_eq!(drained.sim.state_hash(), before);
}

#[test]
fn a_saved_world_does_not_carry_the_record() {
    let mut session = kept(ecology(4_242), Execution::Grouped);
    session.advance(5).unwrap();
    let holding = serde_json::to_string(&session.save()).unwrap();
    assert!(
        !session.sim.take_flows().is_empty(),
        "the advance had moves to leave out"
    );
    let empty = serde_json::to_string(&session.save()).unwrap();
    assert_eq!(holding, empty);
    let mut unkept = Session::new(ecology(4_242), Execution::Grouped).unwrap();
    unkept.advance(5).unwrap();
    assert_eq!(serde_json::to_string(&unkept.save()).unwrap(), holding);
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
    session = kept(g, Execution::Individuals);
    let child = session.sim.state().population.next_id;
    let (outcome, flows) = stepped(&mut session, "at the birth", |s| {
        s.command(act(parent, None, "ecology:birth-1")).unwrap()
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
    let mut session = kept(ecology(1), Execution::Grouped);
    session.advance(3).unwrap();
    session.sim.take_flows();
    let before = books(&session);
    session.advance(1).unwrap();
    let flows = session.sim.take_flows();
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
    let mut session = kept(ecology(3), Execution::Individuals);
    let placements = [(400, 0), (75, 2)];
    let mut flows = vec![];
    for (amount, site) in placements {
        let place = Command::PlaceMatter {
            site,
            account: "world:soil".into(),
            amount,
        };
        flows.extend(stepped(&mut session, "placing", |s| s.command(place).unwrap()).1);
    }
    flows.extend(stepped(&mut session, "after", |s| s.advance(2).unwrap()).1);
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
    let mut session = kept(g, Execution::Grouped);
    let (producer, consumer) = neighbours(&session, "lineage:0", "lineage:1");
    let (outcome, flows) = stepped(&mut session, "blocked", |s| {
        s.command(act(producer, Some(consumer), "test:overdraw"))
            .unwrap()
    });
    assert!(outcome.contains("Blocked"), "{outcome}");
    assert!(flows.is_empty(), "a blocked act moves nothing");
    let (outcome, flows) = stepped(&mut session, "accepted", |s| {
        s.command(act(producer, Some(consumer), "sim:give"))
            .unwrap()
    });
    assert!(outcome.contains("Accepted"), "{outcome}");
    assert_eq!(flows.len(), 1);
    // An advance refused part way is put back, and takes its moves with it.
    let mut tight = ecology(5);
    tight.rules.limits.events_per_advance = 25;
    let mut session = kept(tight, Execution::Individuals);
    let hash = session.sim.state_hash();
    assert!(
        session.advance(40).is_err(),
        "the budget refuses the advance"
    );
    assert_eq!(session.sim.state_hash(), hash);
    assert!(
        session.sim.take_flows().is_empty(),
        "a refused advance leaves no moves"
    );
}
