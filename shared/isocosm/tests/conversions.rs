// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an act may do to matter (Part B, step 2): declared conversions
//! (rulings 342 and 357), the dev source (rulings 344 and 358), and the parts
//! of `mesocosm-core`'s stock and receipt tests about them. The receipt tests
//! about reconciling a record are in `tests/flows.rs`.

use isocosm::{
    Execution, Founding, Session, Simulation,
    history::Command,
    population::Population,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::BTreeSet;

const SOIL: &str = "world:soil";
/// The matter of lineages 0, 1 and 2, the generated ecology's producer,
/// consumer and decomposer.
const OWN: [&str; 3] = ["matter:0-0", "matter:1-0", "matter:2-0"];

fn ledger(entries: &[(&str, u64)]) -> Ledger {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// One site, its world body (0), and a member of each of lineages 0, 1 and
/// 2 (members 1, 2 and 3) holding `ledgers`. `processes` are the world's
/// only processes, and none comes due.
fn world(ledgers: [&[(&str, u64)]; 3], processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 342,
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
        e.accounts = ledger(entries);
        population.insert(e, 1).unwrap();
    }
    g.population = population;
    g.rules.processes.retain(|_, p| p.period.is_none());
    for p in processes {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g
}

fn process(id: &str, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
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

fn convert(
    who: Binding,
    take: &[(&str, u64)],
    give: &[(&str, u64)],
    conversion: Option<Conversion>,
) -> Effect {
    Effect::Transform {
        who,
        take: ledger(take),
        give: ledger(give),
        conversion,
    }
}

fn held(sim: &Simulation, id: Id) -> Ledger {
    sim.state().population.get(id).unwrap().accounts.clone()
}

/// `mesocosm-core`'s stock refusing a partial removal: here a transform
/// taking more of one account than a body holds is refused whole, as its
/// stage is, and the body keeps every account as it was.
#[test]
fn a_take_beyond_one_account_is_refused_whole() {
    let accounts = ["test:a", "test:b", "test:c", "test:d", "test:e"];
    let take = convert(
        Binding::Actor,
        &[("test:a", 4), ("test:b", 4)],
        &[("test:e", 8)],
        None,
    );
    let body: &[(&str, u64)] = &[("test:a", 4), ("test:b", 3), ("test:c", 2), ("test:d", 1)];
    let mut g = world([body, &[], &[]], vec![process("test:take", vec![take])]);
    for key in accounts {
        let kind = AccountKind::Matter {
            lineage: "lineage:0".into(),
        };
        g.rules.accounts.insert(key.into(), kind);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let hash = sim.state_hash();
    let r = sim.execute(1, None, "test:take", None);
    assert_eq!(r.outcome, Outcome::Blocked("insufficient test:b".into()));
    assert_eq!(held(&sim, 1), ledger(body));
    assert_eq!(sim.state_hash(), hash);
}

#[test]
fn mesocosms_three_conversions_are_admitted() {
    let effects = vec![
        convert(
            Binding::Actor,
            &[(SOIL, 2)],
            &[(OWN[0], 2)],
            Some(Conversion::Synthesis),
        ),
        convert(
            Binding::Actor,
            &[(OWN[0], 2)],
            &[(OWN[1], 2)],
            Some(Conversion::Digestion),
        ),
        convert(
            Binding::Actor,
            &[(OWN[2], 2)],
            &[(SOIL, 2)],
            Some(Conversion::Mineralization),
        ),
        // What is mineralized may carry the world's matter along.
        convert(
            Binding::Place,
            &[(OWN[2], 1), (SOIL, 1)],
            &[(SOIL, 2)],
            Some(Conversion::Mineralization),
        ),
    ];
    let processes = effects.into_iter().enumerate();
    let processes = processes
        .map(|(i, e)| process(&format!("test:{i}"), vec![e]))
        .collect();
    world([&[], &[], &[]], processes).validate().unwrap();
}

#[test]
fn a_declared_conversion_of_the_wrong_matter_is_refused() {
    use Conversion::*;
    let wrong = "moves the wrong kinds of matter";
    let cases = [
        // The world's matter relabelled as a body's is no digestion.
        (
            wrong,
            convert(
                Binding::Actor,
                &[(SOIL, 10)],
                &[(OWN[1], 10)],
                Some(Digestion),
            ),
        ),
        // Nor is losing matter a synthesis.
        (
            "unbalanced",
            convert(
                Binding::Actor,
                &[(SOIL, 10)],
                &[(OWN[0], 9)],
                Some(Synthesis),
            ),
        ),
        // A synthesis takes only the world's matter.
        (
            wrong,
            convert(
                Binding::Actor,
                &[(SOIL, 2), (OWN[1], 1)],
                &[(OWN[0], 3)],
                Some(Synthesis),
            ),
        ),
        // And yields one lineage's.
        (
            wrong,
            convert(
                Binding::Actor,
                &[(SOIL, 2)],
                &[(OWN[0], 1), (OWN[1], 1)],
                Some(Synthesis),
            ),
        ),
        (
            wrong,
            convert(
                Binding::Actor,
                &[(OWN[2], 1)],
                &[(OWN[0], 1)],
                Some(Mineralization),
            ),
        ),
        (
            wrong,
            convert(
                Binding::Place,
                &[(SOIL, 1)],
                &[(SOIL, 1)],
                Some(Mineralization),
            ),
        ),
        (
            "not a matter account",
            convert(
                Binding::Actor,
                &[("sim:energy", 1), (SOIL, 2)],
                &[(OWN[0], 2)],
                Some(Synthesis),
            ),
        ),
        (
            "of nothing",
            convert(Binding::Place, &[], &[], Some(Mineralization)),
        ),
        (
            "not a body",
            convert(
                Binding::Place,
                &[(SOIL, 1)],
                &[(OWN[0], 1)],
                Some(Synthesis),
            ),
        ),
    ];
    for (why, effect) in cases {
        let g = world([&[], &[], &[]], vec![process("test:wrong", vec![effect])]);
        let refusal = g.validate().unwrap_err();
        assert!(refusal.contains(why), "{why}: {refusal}");
    }
    // Undeclared, the same relabel passes as it did before (ruling 342).
    let undeclared = convert(Binding::Actor, &[(SOIL, 10)], &[(OWN[1], 10)], None);
    world(
        [&[], &[], &[]],
        vec![process("test:as-before", vec![undeclared])],
    )
    .validate()
    .unwrap();
}

#[test]
fn synthesis_and_digestion_yield_only_the_bodys_own_matter() {
    use Conversion::*;
    let processes = vec![
        process(
            "test:digest",
            vec![convert(
                Binding::Actor,
                &[(OWN[0], 2)],
                &[(OWN[1], 2)],
                Some(Digestion),
            )],
        ),
        process(
            "test:fix",
            vec![convert(
                Binding::Actor,
                &[(SOIL, 2)],
                &[(OWN[0], 2)],
                Some(Synthesis),
            )],
        ),
        // Whatever synthesizes is a producer (ruling 357): a consumer that
        // keeps what it ate working, fixing into its own matter.
        process(
            "test:kleptoplasty",
            vec![convert(
                Binding::Actor,
                &[(SOIL, 2)],
                &[(OWN[1], 2)],
                Some(Synthesis),
            )],
        ),
    ];
    let holding: &[(&str, u64)] = &[(OWN[0], 5), (SOIL, 5)];
    let mut sim = Simulation::new(
        world([holding, holding, holding], processes),
        Execution::Individuals,
    )
    .unwrap();
    let not_own = |kind: &str, key: &str, lineage: usize| {
        Outcome::Blocked(format!(
            "{kind} gives {key}, which is not lineage:{lineage}'s own matter"
        ))
    };
    assert_eq!(
        sim.execute(2, None, "test:digest", None).outcome,
        Outcome::Accepted
    );
    let hash = sim.state_hash();
    assert_eq!(
        sim.execute(3, None, "test:digest", None).outcome,
        not_own("Digestion", OWN[1], 2)
    );
    assert_eq!(sim.state_hash(), hash);
    assert_eq!(
        sim.execute(1, None, "test:fix", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(
        sim.execute(2, None, "test:fix", None).outcome,
        not_own("Synthesis", OWN[0], 1)
    );
    assert_eq!(
        sim.execute(2, None, "test:kleptoplasty", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(
        held(&sim, 2),
        ledger(&[(OWN[0], 3), (OWN[1], 4), (SOIL, 3)])
    );
}

/// `mesocosm-core`'s receipt refusing a stock overflow: a transfer that
/// would overflow its destination is refused and changes nothing.
#[test]
fn an_overflowing_transfer_is_refused_and_changes_nothing() {
    let mut give = process(
        "test:give",
        vec![Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Target,
            account: SOIL.into(),
            amount: 1,
        }],
    );
    give.target = Some(Target {
        same_place: true,
        alive: None,
        lineage: None,
        among: BTreeSet::new(),
        weighted: false,
    });
    let g = world([&[(SOIL, 1)], &[(SOIL, u64::MAX)], &[]], vec![give]);
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let hash = sim.state_hash();
    let r = sim.execute(1, Some(2), "test:give", None);
    assert_eq!(
        r.outcome,
        Outcome::Blocked(format!("account overflow: {SOIL}"))
    );
    assert_eq!(sim.state_hash(), hash);
}

fn place(amount: u64) -> Command {
    Command::PlaceMatter {
        site: 0,
        account: SOIL.into(),
        amount,
    }
}

fn act(actor: Id, process: &str) -> Command {
    Command::Act {
        actor,
        target: None,
        process: process.into(),
        cause: None,
    }
}

fn plain() -> Genesis {
    let fix = convert(
        Binding::Actor,
        &[(SOIL, 1)],
        &[(OWN[0], 1)],
        Some(Conversion::Synthesis),
    );
    world(
        [&[(SOIL, 9)], &[], &[]],
        vec![process("test:fix", vec![fix])],
    )
}

#[test]
fn placed_matter_comes_from_outside_and_every_receipt_says_so() {
    let mut session = Session::new(plain(), Execution::Grouped).unwrap();
    let founded = session.sim.matter();
    let before = session.command(act(1, "test:fix")).unwrap();
    assert!(!before.contains("issued"), "{before}");
    assert!(!session.assisted());
    assert_eq!(session.command(place(40)).unwrap(), "placed");
    assert!(session.assisted());
    assert_eq!(session.sim.issued(), 40);
    assert_eq!(session.sim.matter(), founded + 40);
    let site = &session.sim.state().sites[&0].accounts;
    assert_eq!(site[SOIL], plain().sites[&0].accounts[SOIL] + 40);
    let after = session.command(act(1, "test:fix")).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&after).unwrap();
    assert_eq!(receipt.issued, 40);
    assert_eq!(receipt.matter_before - receipt.issued, founded);
    assert_eq!(receipt.matter_after, receipt.matter_before);
}

#[test]
fn a_placement_is_of_declared_matter_at_a_site_that_exists() {
    let mut session = Session::new(plain(), Execution::Individuals).unwrap();
    let hash = session.sim.state_hash();
    let refused = |session: &mut Session, command| session.command(command).unwrap_err();
    let energy = Command::PlaceMatter {
        site: 0,
        account: "sim:energy".into(),
        amount: 1,
    };
    assert!(refused(&mut session, energy).contains("not a matter account"));
    let elsewhere = Command::PlaceMatter {
        site: 9,
        account: SOIL.into(),
        amount: 1,
    };
    assert!(refused(&mut session, elsewhere).contains("unknown site"));
    assert!(refused(&mut session, place(0)).contains("nothing to place"));
    assert!(refused(&mut session, place(u64::MAX)).contains("overflow"));
    // Any declared matter account will do (ruling 358), not only the world's.
    let living = Command::PlaceMatter {
        site: 0,
        account: OWN[2].into(),
        amount: 3,
    };
    assert_eq!(session.command(living).unwrap(), "placed");
    assert_eq!(
        session.entries.len(),
        1,
        "refused placements are not logged"
    );
    assert_eq!(session.sim.issued(), 3);
    assert_ne!(session.sim.state_hash(), hash);
}

#[test]
fn a_world_with_matter_placed_hashes_as_one_founded_holding_it() {
    let mut placed = Simulation::new(plain(), Execution::Grouped).unwrap();
    placed.place(0, SOIL, 40).unwrap();
    let mut holding = plain();
    *holding
        .sites
        .get_mut(&0)
        .unwrap()
        .accounts
        .get_mut(SOIL)
        .unwrap() += 40;
    let founded = Simulation::new(holding, Execution::Grouped).unwrap();
    assert_eq!(placed.state_hash(), founded.state_hash());
    assert_eq!(placed.matter(), founded.matter());
    assert_eq!((placed.issued(), founded.issued()), (40, 0));
}

#[test]
fn placements_replay_from_the_history() {
    let mut session = Session::new(plain(), Execution::Grouped).unwrap();
    session.advance(3).unwrap();
    session.command(place(25)).unwrap();
    session.advance(4).unwrap();
    session.command(act(1, "test:fix")).unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load(session.save(), mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
        assert_eq!(loaded.sim.issued(), 25);
        assert!(loaded.assisted());
    }
    let before = session.fork_at(2, "branch:unassisted".into()).unwrap();
    assert!(!before.assisted());
    assert_eq!(before.sim.issued(), 0);
}
