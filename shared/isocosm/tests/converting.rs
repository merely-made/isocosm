// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Conversions of a computed amount (checkpoint 6): a meal digested, soil
//! synthesized and matter mineralized by `Convert`, a share of each account
//! taken in proportion, and rent spent into the world's matter at once.

use isocosm::{
    Execution, Founding, Session, Simulation,
    flows::Holder,
    population::Population,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const SOIL: &str = "world:soil";
const RESERVE: &str = "test:reserve-0";
const TISSUE: &str = "test:tissue-0";
const PREY: &str = "test:tissue-1";

fn ledger(entries: &[(&str, u64)]) -> Ledger {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// One site holding `site`, its world body (0), and a member of lineage 0
/// (1) holding `own`; lineage 0 keeps a reserve and tissue, lineage 1
/// tissue.
fn world(own: &[(&str, u64)], site: &[(&str, u64)], processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 6,
        sites: 1,
        population: 2,
        lineages: 2,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let ground = g.population.groups[&0].entity.clone();
    let mut body = g.population.get(1).unwrap().clone();
    body.lineage = "lineage:0".into();
    body.accounts = ledger(own);
    let mut population = Population::default();
    population.insert(ground, 1).unwrap();
    population.insert(body, 1).unwrap();
    g.population = population;
    for (key, lineage, reserve) in [
        (RESERVE, "lineage:0", true),
        (TISSUE, "lineage:0", false),
        (PREY, "lineage:1", false),
    ] {
        let kind = AccountKind::Matter {
            lineage: lineage.into(),
            reserve,
        };
        g.rules.accounts.insert(key.into(), kind);
    }
    g.sites.get_mut(&0).unwrap().accounts = ledger(site);
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

fn convert(who: Binding, from: &[&str], to: &str, amount: u64, conversion: Conversion) -> Effect {
    Effect::Convert {
        who,
        from: from.iter().map(|k| k.to_string()).collect(),
        to: to.into(),
        amount: amount.into(),
        conversion,
    }
}

fn run(g: Genesis, id: &str) -> (Simulation, Outcome) {
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let before = sim.matter();
    let outcome = sim.execute(1, None, id, None).outcome;
    assert_eq!(sim.matter(), before, "{id} keeps the world's matter");
    (sim, outcome)
}

fn held(sim: &Simulation, id: Id) -> Ledger {
    let mut l = sim.state().population.get(id).unwrap().accounts.clone();
    l.retain(|_, v| *v > 0);
    l
}

#[test]
fn a_conversion_takes_up_to_its_amount_in_proportion() {
    // A meal of 6 and 2 of two lineages' tissue, digested 4 at a time:
    // shares of 3 and 1. Synthesis of soil in hand into the reserve, and
    // more asked than held takes what there is.
    let digest = convert(
        Binding::Actor,
        &[PREY, TISSUE],
        RESERVE,
        4,
        Conversion::Digestion,
    );
    let synthesize = convert(Binding::Actor, &[SOIL], RESERVE, 9, Conversion::Synthesis);
    let g = world(
        &[(PREY, 6), (TISSUE, 2), (SOIL, 5)],
        &[],
        vec![
            process("test:digest", vec![digest]),
            process("test:synthesize", vec![synthesize]),
        ],
    );
    let (sim, outcome) = run(g.clone(), "test:digest");
    assert_eq!(outcome, Outcome::Accepted);
    let want = ledger(&[(PREY, 3), (TISSUE, 1), (RESERVE, 4), (SOIL, 5)]);
    assert_eq!(held(&sim, 1), want);
    let (sim, outcome) = run(g, "test:synthesize");
    assert_eq!(outcome, Outcome::Accepted);
    let want = ledger(&[(PREY, 6), (TISSUE, 2), (RESERVE, 5)]);
    assert_eq!(held(&sim, 1), want);
}

#[test]
fn a_site_mineralizes_a_dose_of_its_living_matter() {
    let dose = convert(
        Binding::Place,
        &[TISSUE, PREY],
        SOIL,
        4,
        Conversion::Mineralization,
    );
    let g = world(&[], &[(TISSUE, 3), (PREY, 5), (SOIL, 1)], vec![]);
    let mut g = g;
    let mut p = process("test:mineralize", vec![dose]);
    p.causation = Causation::Agentless;
    g.rules.processes.insert(p.id.clone(), p);
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let before = sim.matter();
    assert_eq!(
        sim.execute(0, None, "test:mineralize", None).outcome,
        Outcome::Accepted
    );
    assert_eq!(sim.matter(), before);
    // Shares of 1.5 and 2.5: the unit left over goes to the larger
    // remainder, tied here, so to the first key.
    let site = &sim.state().sites[&0].accounts;
    assert_eq!(
        (site[TISSUE], site[PREY], site[SOIL]),
        (1, 3, 5),
        "{site:?}"
    );
}

#[test]
fn rent_spent_into_soil_arrives_as_the_worlds_matter() {
    let rent = Effect::Spend {
        from: vec![RESERVE.into(), TISSUE.into()],
        to: Binding::Place,
        amount: 5.into(),
        into: Some(SOIL.into()),
    };
    let mut p = process("test:rent", vec![rent]);
    p.period = Some(1);
    let g = world(&[(RESERVE, 3), (TISSUE, 10)], &[(SOIL, 1)], vec![p]);
    let mut session = Session::new(g, Execution::Individuals).unwrap();
    let flows = session.advance_tick_with_flows().unwrap().flows;
    let legs: Vec<_> = flows
        .iter()
        .map(|f| (f.from.clone(), f.to.clone(), f.amount))
        .collect();
    let (body, ground) = (Holder::Entity(1), Holder::Site(0));
    assert_eq!(
        legs,
        [
            ((body, RESERVE.into()), (ground, SOIL.into()), 3),
            ((body, TISSUE.into()), (ground, SOIL.into()), 2),
        ]
    );
    let sim = &session.sim;
    assert_eq!(sim.state().sites[&0].accounts[SOIL], 6);
    assert_eq!(held(sim, 1), ledger(&[(TISSUE, 8)]));
}

#[test]
fn rules_and_acts_refuse_what_a_conversion_cannot_do() {
    let refused = |e: Effect| {
        world(&[], &[], vec![process("test:x", vec![e])])
            .validate()
            .is_err()
    };
    let a = Binding::Actor;
    assert!(
        refused(convert(a, &[PREY], RESERVE, 1, Conversion::Synthesis)),
        "living into living"
    );
    assert!(
        refused(convert(a, &[SOIL], RESERVE, 1, Conversion::Digestion)),
        "the world's, digested"
    );
    assert!(
        refused(convert(a, &[TISSUE], SOIL, 1, Conversion::Digestion)),
        "into the world's"
    );
    assert!(
        refused(convert(a, &[TISSUE], TISSUE, 1, Conversion::Mineralization)),
        "into itself"
    );
    assert!(refused(convert(
        Binding::Part,
        &[TISSUE],
        SOIL,
        1,
        Conversion::Mineralization
    )));
    assert!(
        refused(convert(
            Binding::Place,
            &[SOIL],
            TISSUE,
            1,
            Conversion::Synthesis
        )),
        "a site synthesizes"
    );
    let spend = |into: &str| Effect::Spend {
        from: vec![TISSUE.into()],
        to: Binding::Place,
        amount: 1.into(),
        into: Some(into.into()),
    };
    assert!(refused(spend(PREY)), "spent into another's living matter");
    assert!(!refused(spend(SOIL)), "the control, into the world's");
    // A synthesis into another lineage's matter is refused when it acts.
    let theirs = convert(a, &[SOIL], PREY, 1, Conversion::Synthesis);
    let g = world(
        &[(SOIL, 2)],
        &[],
        vec![process("test:theirs", vec![theirs])],
    );
    let (sim, outcome) = run(g, "test:theirs");
    assert!(matches!(outcome, Outcome::Blocked(_)), "{outcome:?}");
    assert_eq!(held(&sim, 1), ledger(&[(SOIL, 2)]));
}

#[test]
fn spends_without_a_target_serialize_as_before() {
    let plain = Effect::Spend {
        from: vec![RESERVE.into()],
        to: Binding::Place,
        amount: 1.into(),
        into: None,
    };
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("into"), "{json}");
    let back: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(back, plain);
    let _ = BTreeMap::<Key, u64>::new();
}
