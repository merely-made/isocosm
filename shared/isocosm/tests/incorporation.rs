// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8f: a part taken whole (rulings 468, 510, 516 and
//! 544). A bite that would take all of a frond takes it whole, landing at
//! the first free box the grazer's plan finds: native as it was, an
//! adapter's expressing nothing, a refused one eaten as a meal. It keeps
//! the producer's matter, which fills its room for the grazer's own; the
//! producer, left with nothing, is dead; and the grazer's lineage learns
//! the frond's kind. Every act reconciles in the flow record.

use isocosm::{
    Command, Execution, Session, anatomy,
    development::{Soma, develop},
    flows::Holder,
    probe::BodyFounding,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

fn tagma(segment: &str, bears: Option<&str>) -> Tagma {
    Tagma {
        segments: 1,
        segment: segment.into(),
        bears: bears.map(Into::into),
        per_segment: u8::from(bears.is_some()),
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    }
}

fn development(t: Tagma, domain: u16) -> Development {
    let recipe = Recipe {
        tagmata: vec![t],
        variance: 0,
        absence: [0, 1],
        riff: [0, 1],
        vary: [0, 1],
    };
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain,
        clutch: 1,
        anamorphic: false,
    }
}

fn first(g: &Genesis, lineage: &str) -> Id {
    let groups = g.population.groups.iter();
    let mut of = groups.filter(|(_, c)| c.entity.lineage == lineage);
    *of.next().unwrap().0
}

/// A body developed from `d`, its tissue full.
fn dress(g: &mut Genesis, id: Id, d: &Development, tissue: &str) {
    let rules = g.rules.clone();
    let soma = Soma {
        segments: vec![1],
        absent: vec![],
        seed: 0,
    };
    let e = g.population.lift(id).unwrap();
    e.embody(develop(&rules, d, &soma).unwrap());
    e.soma = vec![1];
    e.accounts.clear();
    let room = anatomy::room(e, &rules, tissue);
    anatomy::give(e, &rules, tissue, room).unwrap().unwrap();
}

/// The probe's world with a frond producer of `domain` and a grazer of
/// domain 0, each developed and full; returns it, the grazer and the
/// producer.
fn world(domain: u16, bite: u64, whole: bool) -> (Genesis, Id, Id) {
    let mut g = BodyFounding::default().generate().unwrap().genesis;
    g.rules.kinds = BTreeMap::from([
        (
            "kind:frond".into(),
            template([3, 3, 1], &[("fix", 3), ("reproduce", 1)]),
        ),
        (
            "kind:lump".into(),
            template([2, 2, 2], &[("intake", 5), ("store", 1), ("reproduce", 1)]),
        ),
        ("kind:limb".into(), template([3, 1, 1], &[("contract", 2)])),
    ]);
    let frond = development(tagma("kind:frond", None), domain);
    let grazer = development(tagma("kind:lump", Some("kind:limb")), 0);
    g.lineages.get_mut("lineage:0").unwrap().development = Some(frond.clone());
    g.lineages.get_mut("lineage:1").unwrap().development = Some(grazer.clone());
    let (eater, prey) = (first(&g, "lineage:1"), first(&g, "lineage:0"));
    dress(&mut g, eater, &grazer, "tissue:1");
    dress(&mut g, prey, &frond, "tissue:0");
    let mut graze = Process {
        id: "test:graze".into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects: vec![Effect::Eat {
            from: Binding::Target,
            amount: Amount::Fixed(bite),
            into: "tissue:0".into(),
            of: vec!["tissue:0".into()],
            whole,
        }],
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    };
    graze.target = Some(Target {
        same_place: false,
        alive: Some(true),
        lineage: Some("lineage:0".into()),
        among: BTreeSet::new(),
        weighted: false,
    });
    g.rules.processes.insert(graze.id.clone(), graze);
    (g, eater, prey)
}

type Books = BTreeMap<(Holder, Key), i128>;

fn books(s: &Session) -> Books {
    let (rules, state) = (&s.sim.genesis().rules, s.sim.state());
    let matter = |k: &Key| matches!(rules.accounts.get(k), Some(AccountKind::Matter { .. }));
    let mut books = Books::new();
    let mut hold = |h: Holder, l: &Ledger| {
        for (k, v) in l.iter().filter(|(k, v)| matter(k) && **v > 0) {
            *books.entry((h, k.clone())).or_default() += i128::from(*v);
        }
    };
    for (&first, c) in &state.population.groups {
        for id in first..first + c.count {
            hold(Holder::Entity(id), &c.entity.accounts);
            for (&part, p) in &c.entity.parts {
                hold(Holder::Part(id, part), &p.matter);
            }
        }
    }
    for (&id, site) in &state.sites {
        hold(Holder::Site(id), &site.accounts);
    }
    books
}

/// The grazer grazes the producer, the record checked against every
/// ledger's change; returns the session and the act's outcome.
fn graze(domain: u16, bite: u64, whole: bool) -> (Session, Id, Id, String) {
    let (g, eater, prey) = world(domain, bite, whole);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let before = books(&s);
    let command = Command::Act {
        actor: eater,
        target: Some(prey),
        process: "test:graze".into(),
        cause: None,
    };
    let record = s.command_with_flows(command).unwrap();
    let mut claimed = Books::new();
    for f in &record.flows {
        for (holder, key, sign) in [(f.from.0, &f.from.1, -1), (f.to.0, &f.to.1, 1)] {
            let amount = sign * i128::from(f.amount) * i128::from(f.count);
            *claimed.entry((holder, key.clone())).or_default() += amount;
        }
    }
    let after = books(&s);
    let keys: BTreeSet<_> = before
        .keys()
        .chain(after.keys())
        .chain(claimed.keys())
        .collect();
    for k in keys {
        let delta = after.get(k).copied().unwrap_or(0) - before.get(k).copied().unwrap_or(0);
        assert_eq!(delta, claimed.get(k).copied().unwrap_or(0), "{k:?}");
    }
    let v: serde_json::Value = serde_json::from_str(&record.result).unwrap();
    (s, eater, prey, v["outcome"].to_string())
}

fn body(s: &Session, id: Id) -> Entity {
    s.sim.state().population.get(id).unwrap().clone()
}

fn lexicon(s: &Session) -> BTreeSet<Key> {
    let l = &s.sim.state().lineages["lineage:1"];
    l.development.as_ref().unwrap().lexicon.clone()
}

#[test]
fn a_native_frond_lands_whole_keeping_its_matter_and_its_lesson() {
    let (s, eater, prey, outcome) = graze(0, 1000, true);
    assert_eq!(outcome, "\"Accepted\"");
    let grazer = body(&s, eater);
    let frond = grazer
        .parts
        .values()
        .find(|p| p.functions.contains("function:fix"));
    let frond = frond.expect("the frond landed");
    assert_eq!(frond.cells["function:fix"], 3, "native, as it was");
    // A frond of 147 voxels weighs 117 mg, the producer's still.
    assert_eq!(frond.matter.get("tissue:0").copied(), Some(117));
    assert_eq!(
        grazer.accounts.get("tissue:0").copied().unwrap_or(0),
        0,
        "no meal"
    );
    assert!(!body(&s, prey).alive, "the producer had nothing else");
    assert!(lexicon(&s).contains("kind:frond"));
    // The donor's tissue fills the frond's room for the grazer's own.
    let rules = &s.sim.genesis().rules;
    let mut only = grazer.clone();
    only.parts
        .retain(|_, p| p.functions.contains("function:fix"));
    assert_eq!(anatomy::room(&only, rules, "tissue:1"), 0);
}

#[test]
fn an_adapter_lands_expressing_nothing_and_a_refused_frond_is_eaten() {
    // Domain 2 is favoured into 0: an adapter.
    let (s, eater, _, _) = graze(2, 1000, true);
    let grazer = body(&s, eater);
    let landed = grazer
        .parts
        .values()
        .find(|p| p.matter.contains_key("tissue:0"));
    let landed = landed.expect("the adapter's frond landed");
    assert!(landed.cells.is_empty() && landed.functions.is_empty());
    // Domain 1 into 0 is refused: the frond is eaten, the grazer holding
    // its tissue in hand, and nothing is learned.
    let (s, eater, prey, _) = graze(1, 1000, true);
    let grazer = body(&s, eater);
    assert_eq!(grazer.accounts.get("tissue:0").copied(), Some(117));
    assert!(
        grazer
            .parts
            .values()
            .all(|p| !p.matter.contains_key("tissue:0"))
    );
    assert_eq!(body(&s, prey).parts.len(), 1, "the frond stays, eaten");
    assert!(!lexicon(&s).contains("kind:frond"));
}

#[test]
fn a_bite_that_takes_less_than_the_part_or_wants_no_part_eats_it() {
    for (bite, whole) in [(50, true), (1000, false)] {
        let (s, eater, prey, _) = graze(0, bite, whole);
        let grazer = body(&s, eater);
        assert!(
            grazer
                .parts
                .values()
                .all(|p| !p.matter.contains_key("tissue:0")),
            "{bite} {whole}"
        );
        assert_eq!(
            grazer.accounts.get("tissue:0").copied(),
            Some(bite.min(117))
        );
        assert!(body(&s, prey).alive);
        assert!(!lexicon(&s).contains("kind:frond"));
    }
}

/// One pass of two grazers on one producer whose root holds 3 mg and its
/// frond 10, each bite drawn by holdings and taken whole where it would
/// take all of the frond (rulings 454 and 516). A part an earlier act of
/// the pass bit or took may not be taken again as the pass found it, and
/// a part it took stays taken, so the world's matter holds whichever order
/// the bites come in; across seeds the frond is taken whole and a later
/// bite refused.
#[test]
fn a_pass_takes_a_part_whole_only_as_it_finds_it() {
    let (mut taken, mut refused) = (0, 0);
    for seed in 0..24 {
        for asks in [[10, 3], [3, 10]] {
            let (g, grazers, prey) = pass_world(seed, asks);
            let mut sim = isocosm::Simulation::new(g, Execution::Individuals).unwrap();
            sim.watch("test:graze");
            let before = sim.matter();
            sim.advance(1).unwrap();
            assert_eq!(sim.matter(), before, "seed {seed} {asks:?}");
            let state = sim.state();
            let holds = |id: Id| {
                let e = state.population.get(id).unwrap();
                let foreign = |p: &&Part| p.matter.get("tissue:0").is_some_and(|v| *v > 0);
                e.parts.values().filter(foreign).count()
            };
            let whole: usize = grazers.iter().map(|g| holds(*g)).sum();
            assert!(whole <= 1, "seed {seed}: the frond taken twice");
            assert!(
                state.population.get(prey).unwrap().parts.len() + whole == 2,
                "seed {seed}: a part taken is gone, and only it"
            );
            taken += whole;
            refused += 2 - sim.take_watched().len();
        }
    }
    assert!(taken > 0 && refused > 0, "{taken} taken, {refused} refused");
}

/// The probe's world cut to one site's ground, a producer of two fronds
/// holding 3 and 10 mg and two grazers asking `asks`, grazing alone.
fn pass_world(seed: u64, asks: [u64; 2]) -> (Genesis, [Id; 2], Id) {
    let (mut g, eater, prey) = world(0, 0, true);
    g.seed = seed;
    let frond = g.lineages["lineage:0"].development.clone().unwrap();
    let rules = g.rules.clone();
    let mut producer = g.population.get(prey).unwrap().clone();
    let soma = Soma {
        segments: vec![2],
        absent: vec![],
        seed: 0,
    };
    producer.embody(develop(&rules, &frond, &soma).unwrap());
    producer.soma = vec![2];
    for (part, held) in [(0, 3), (1, 10)] {
        producer.parts.get_mut(&PartId(part)).unwrap().matter =
            BTreeMap::from([("tissue:0".into(), held)]);
    }
    let grazer = g.population.get(eater).unwrap().clone();
    let site = producer.place;
    let mut population = isocosm::population::Population::default();
    for c in g.population.groups.values() {
        if c.entity.lineage == "world:ground" {
            population.insert(c.entity.clone(), 1).unwrap();
        }
    }
    let prey = population.insert(producer, 1).unwrap();
    let mut ids = [0; 2];
    for (k, ask) in asks.into_iter().enumerate() {
        let mut e = grazer.clone();
        e.place = site;
        e.accounts.insert("test:appetite".into(), ask);
        ids[k] = population.insert(e, 1).unwrap();
    }
    g.population = population;
    g.rules
        .accounts
        .insert("test:appetite".into(), AccountKind::Energy);
    let mut graze = g.rules.processes["test:graze"].clone();
    if let Some(Effect::Eat { amount, .. }) = graze.effects.first_mut() {
        *amount = Amount::Computed(Expr::Read(Reading::Account {
            who: Binding::Actor,
            key: "test:appetite".into(),
        }));
    }
    graze.period = Some(1);
    graze.target.as_mut().unwrap().same_place = true;
    g.rules.processes = BTreeMap::from([(graze.id.clone(), graze)]);
    (g, ids, prey)
}

/// The same pass where the smaller grazer gives the producer 5 mg instead
/// of biting it, which lands by room on its fronds: a gift to a frond an
/// earlier act took whole may not land, and the world's matter holds.
#[test]
fn a_pass_gives_nothing_to_a_part_it_took() {
    let mut refused = 0;
    for seed in 0..24 {
        for asks in [[10, 0], [0, 10]] {
            let (mut g, grazers, _) = pass_world(seed, asks);
            let tend = g.rules.processes.get_mut("test:graze").unwrap();
            let eat = tend.effects.remove(0);
            let appetite = Expr::Read(Reading::Account {
                who: Binding::Actor,
                key: "test:appetite".into(),
            });
            tend.effects = vec![Effect::When {
                guard: Expr::AtLeast(Box::new(appetite), Box::new(Expr::Const(1))),
                then: vec![eat],
                otherwise: vec![Effect::Transfer {
                    from: Binding::Actor,
                    to: Binding::Target,
                    account: "tissue:0".into(),
                    amount: Amount::Fixed(5),
                }],
            }];
            for (k, ask) in asks.into_iter().enumerate() {
                if ask == 0 {
                    let e = g.population.lift(grazers[k]).unwrap();
                    e.accounts.insert("tissue:0".into(), 5);
                }
            }
            let mut sim = isocosm::Simulation::new(g, Execution::Individuals).unwrap();
            sim.watch("test:graze");
            let before = sim.matter();
            sim.advance(1).unwrap();
            assert_eq!(sim.matter(), before, "seed {seed} {asks:?}");
            refused += 2 - sim.take_watched().len();
        }
    }
    assert!(refused > 0, "no gift met a frond taken");
}
