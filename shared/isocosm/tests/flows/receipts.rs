// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! `mesocosm-core`'s receipt tests that reconcile a book against a stream,
//! ported to the flow record: its soil, parts and reserves are members' and
//! sites' matter accounts, and its typed conversions are declared ones.
//! Those about what a conversion may do are in `tests/conversions.rs`. The
//! test that a reserve refuses living matter at an intermediate step is not
//! ported: Isocosm keeps no reserve account apart from a body's matter.

use super::*;
use isocosm::population::Population;

const SOIL: &str = "world:soil";
const OWN: [&str; 3] = ["matter:0-0", "matter:1-0", "matter:2-0"];

/// One site, its world body (0), and members 1, 2 and 3 of lineages 0, 1
/// and 2, a producer, a consumer and a decomposer, holding `ledgers`.
/// `processes` are the only processes, and none comes due.
fn trio(ledgers: [&[(&str, u64)]; 3], processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 345,
        sites: 1,
        population: 3,
        lineages: 3,
        cohort_size: 1,
        ecology: true,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let ground = g.population.groups[&0].entity.clone();
    let template = g.population.get(1).unwrap().clone();
    let mut population = Population::default();
    population.insert(ground, 1).unwrap();
    for (i, entries) in ledgers.iter().enumerate() {
        let lineage = format!("lineage:{i}");
        let mut e = template.clone();
        e.kingdom = g.lineages[&lineage].kingdom.clone();
        e.traits = g.lineages[&lineage].traits.clone();
        e.lineage = lineage;
        e.accounts = entries.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        population.insert(e, 1).unwrap();
    }
    g.population = population;
    g.rules.processes.retain(|_, p| p.period.is_none());
    for p in processes {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g
}

fn step(id: &str, effects: Vec<Effect>, eats: bool) -> Process {
    let mut p = trio([&[], &[], &[]], vec![]).rules.processes["sim:give"].clone();
    p.id = id.into();
    p.note = false;
    p.effects = effects;
    p.target = eats.then(|| Target {
        same_place: true,
        alive: None,
        lineage: None,
        among: BTreeSet::new(),
        weighted: false,
    });
    p
}

fn convert(take: &str, give: &str, kind: Conversion) -> Effect {
    Effect::Transform {
        who: Binding::Actor,
        take: Ledger::from([(take.into(), 10)]),
        give: Ledger::from([(give.into(), 10)]),
        conversion: Some(kind),
    }
}

fn taking(account: &str) -> Effect {
    Effect::Transfer {
        from: Binding::Target,
        to: Binding::Actor,
        account: account.into(),
        amount: 10,
    }
}

/// The world's matter through a producer, a consumer and a decomposer and
/// back, each act reconciled: world into producer tissue, eaten and
/// digested twice, then returned.
fn chain() -> Genesis {
    use Conversion::*;
    let processes = vec![
        step("test:fix", vec![convert(SOIL, OWN[0], Synthesis)], false),
        step(
            "test:graze",
            vec![taking(OWN[0]), convert(OWN[0], OWN[1], Digestion)],
            true,
        ),
        step(
            "test:rot",
            vec![taking(OWN[1]), convert(OWN[1], OWN[2], Digestion)],
            true,
        ),
        step(
            "test:return",
            vec![convert(OWN[2], SOIL, Mineralization)],
            false,
        ),
    ];
    trio([&[(SOIL, 10)], &[], &[]], processes)
}

const ACTS: [(Id, Option<Id>, &str); 4] = [
    (1, None, "test:fix"),
    (2, Some(1), "test:graze"),
    (3, Some(2), "test:rot"),
    (3, None, "test:return"),
];

#[test]
fn a_chain_from_the_world_through_three_lineages_and_back_reconciles() {
    let mut session = new_session(chain(), Execution::Grouped);
    let world = |s: &Session| -> u128 {
        let e = |id| {
            s.sim
                .state()
                .population
                .get(id)
                .unwrap()
                .accounts
                .get(SOIL)
                .copied()
        };
        (1..=3).filter_map(e).map(u128::from).sum()
    };
    let (start, mut moves) = (world(&session), vec![]);
    for (actor, target, process) in ACTS {
        let (outcome, flows) = stepped(&mut session, process, |s| {
            s.command_with_flows(act(actor, target, process)).unwrap()
        });
        assert!(outcome.contains("Accepted"), "{process}: {outcome}");
        moves.extend(flows);
    }
    assert_eq!(world(&session), start, "the world's matter came back");
    let by: Vec<String> = moves
        .iter()
        .map(|f| match &f.made_by {
            MadeBy::Process(p) => p.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    let expected = ["fix", "graze", "graze", "rot", "rot", "return"];
    assert_eq!(by, expected.map(|p| format!("test:{p}")));
}

/// `mesocosm-core`'s reserve and graft moves: a transfer between bodies
/// keeps its account on both sides.
#[test]
fn transfers_between_bodies_keep_their_accounts() {
    let mut session = new_session(chain(), Execution::Individuals);
    for (actor, target, process) in &ACTS[..2] {
        let (_, flows) = stepped(&mut session, process, |s| {
            s.command_with_flows(act(*actor, *target, process)).unwrap()
        });
        for f in flows.iter().filter(|f| f.from.0 != f.to.0) {
            assert_eq!(f.from.1, f.to.1, "{f:?}");
            assert_eq!((f.from.0, f.to.0), (Holder::Entity(1), Holder::Entity(2)));
        }
    }
}

/// `mesocosm-core`'s typed control: the same total in a different account
/// must not pass, though a check of totals alone would.
#[test]
fn a_relabel_is_caught_though_the_total_matter_holds() {
    let mut session = new_session(chain(), Execution::Grouped);
    let before = books(&session);
    let flows = session
        .command_with_flows(act(1, None, "test:fix"))
        .unwrap()
        .flows;
    let honest = books(&session);
    reconcile(&before, &honest, &flows, "after fixing").unwrap();
    let mut relabelled = honest.clone();
    let tissue = (Holder::Entity(1), OWN[0].to_string());
    let moved = relabelled.remove(&tissue).unwrap();
    relabelled.insert((Holder::Entity(1), OWN[1].to_string()), moved);
    let total = |b: &Books| b.values().sum::<i128>();
    assert_eq!(total(&relabelled), total(&honest));
    assert!(reconcile(&before, &relabelled, &flows, "relabelled").is_err());
}

/// `mesocosm-core`'s omitted destination and missing receipt: a record
/// naming the wrong holder, or none at all, must not pass.
#[test]
fn a_record_missing_a_move_or_naming_the_wrong_holder_is_caught() {
    let mut session = new_session(chain(), Execution::Grouped);
    session.command(act(1, None, "test:fix")).unwrap();
    let before = books(&session);
    let flows = session
        .command_with_flows(act(2, Some(1), "test:graze"))
        .unwrap()
        .flows;
    let after = books(&session);
    reconcile(&before, &after, &flows, "after grazing").unwrap();
    let mut misplaced = flows.clone();
    misplaced[0].to.0 = Holder::Entity(3);
    assert!(reconcile(&before, &after, &misplaced, "misplaced").is_err());
    assert!(reconcile(&before, &after, &[], "unrecorded").is_err());
}
