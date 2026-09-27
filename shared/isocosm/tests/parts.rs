// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The part a process binds (ruling 338): its actor's lowest-numbered live
//! part expressing the function the process requires, read and marked
//! through `Binding::Part`, its address in the act's receipt. Seeded draws
//! over the whole catalogue must agree run individually and grouped.

use isocosm::{
    Execution, Founding, Session, Simulation,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const USED: &str = "test:used";
const WORN: &str = "test:worn";
const ENERGY: &str = "sim:energy";

fn part(shape: &str, functions: impl IntoIterator<Item = impl Into<Key>>, severed: bool) -> Part {
    Part {
        parent: None,
        traits: BTreeSet::new(),
        severed,
        shape: shape.into(),
        functions: functions.into_iter().map(Into::into).collect(),
    }
}

fn process(id: &str, causation: Causation, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation,
        requires,
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

fn expresses(function: &str) -> Query {
    Query::Expresses {
        function: function.into(),
    }
}

/// The bound part carries `key`.
fn marked(key: &str) -> Query {
    Query::Trait {
        who: Binding::Part,
        key: key.into(),
    }
}

fn mark(key: &str, present: bool) -> Effect {
    Effect::Trait {
        who: Binding::Part,
        key: key.into(),
        present,
    }
}

/// Energy spent, or with a negative `amount` regained; it is not matter.
fn energy(amount: i64) -> Effect {
    let entry = BTreeMap::from([(ENERGY.to_string(), amount.unsigned_abs())]);
    let (take, give) = if amount > 0 {
        (entry, BTreeMap::new())
    } else {
        (BTreeMap::new(), entry)
    };
    Effect::Transform {
        who: Binding::Actor,
        take,
        give,
        conversion: None,
    }
}

#[test]
fn an_act_binds_its_actors_lowest_numbered_live_part_expressing_the_function() {
    let mut g = Founding {
        seed: 338,
        sites: 1,
        population: 1,
        lineages: 1,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.shapes = default_shapes();
    g.rules.functions = default_functions();
    g.rules.traits.insert(USED.into());
    g.population.lift(1).unwrap().parts = BTreeMap::from([
        (0, part("part-shape:sheet", ["function:fix"], true)),
        (1, part("part-shape:rod", ["function:contract"], false)),
        (2, part("part-shape:sheet", ["function:fix"], false)),
        (
            3,
            part(
                "part-shape:sheet",
                ["function:fix", "function:secrete"],
                false,
            ),
        ),
    ]);
    let choice = Causation::Choice;
    for p in [
        process(
            "test:fix",
            choice,
            vec![expresses("function:fix")],
            vec![mark(USED, true)],
        ),
        process(
            "test:rest",
            choice,
            vec![expresses("function:fix"), marked(USED)],
            vec![mark(USED, false)],
        ),
        process(
            "test:secrete",
            choice,
            vec![expresses("function:secrete")],
            vec![mark(USED, true)],
        ),
        process(
            "test:sense",
            choice,
            vec![expresses("function:sense")],
            vec![],
        ),
    ] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let body = |sim: &Simulation| sim.state().population.get(1).unwrap().clone();
    let used = |sim: &Simulation| -> Vec<Id> {
        let parts = body(sim).parts;
        let used = parts.into_iter().filter(|(_, p)| p.traits.contains(USED));
        used.map(|(id, _)| id).collect()
    };
    let address = |f: &str, part: Id, revision: u64| {
        format!("{:?} = part {part} at revision {revision}", expresses(f))
    };
    // Part 2 is bound: part 0 is severed and part 1 does not fix.
    let rest = sim.execute(1, None, "test:rest", None);
    let unmarked = format!("missing requirement: {:?} = false", marked(USED));
    assert_eq!(rest.outcome, Outcome::Blocked(unmarked));
    let fix = sim.execute(1, None, "test:fix", None);
    assert_eq!(fix.outcome, Outcome::Accepted);
    assert_eq!(fix.facts_read, [address("function:fix", 2, 1)]);
    assert_eq!(used(&sim), [2]);
    assert_eq!(body(&sim).body_revision, 2);
    // Reading and clearing the mark binds the same part, now revised.
    let rest = sim.execute(1, None, "test:rest", None);
    assert_eq!(rest.outcome, Outcome::Accepted);
    assert_eq!(rest.facts_read[0], address("function:fix", 2, 2));
    assert!(used(&sim).is_empty());
    // Only part 3 has acquired secreting.
    let secrete = sim.execute(1, None, "test:secrete", None);
    assert_eq!(secrete.facts_read, [address("function:secrete", 3, 3)]);
    assert_eq!(used(&sim), [3]);
    // Nothing senses, so the act is blocked and changes nothing.
    let hash = sim.state_hash();
    let sense = sim.execute(1, None, "test:sense", None);
    let why = format!(
        "missing requirement: {:?} = no live part",
        expresses("function:sense")
    );
    assert_eq!(sense.outcome, Outcome::Blocked(why));
    assert_eq!(sim.state_hash(), hash);
}

/// Every non-empty set of the eight shapes, grown and acquired, in order.
fn every_function() -> Vec<(Key, Function)> {
    let mut all = vec![];
    for mask in 1u32..256 {
        let shapes: BTreeSet<Key> = (0..8)
            .filter(|i| mask >> i & 1 == 1)
            .map(|i| SHAPES[i].to_string())
            .collect();
        for (seeding, name) in [(Seeding::Grown, "grown"), (Seeding::Acquired, "acquired")] {
            let f = Function {
                shapes: shapes.clone(),
                seeding,
            };
            all.push((format!("function:{mask}-{name}"), f));
        }
    }
    all
}

const SLICES: usize = 16;
const TICKS: Tick = 10;

fn live(e: &Entity, function: &str) -> bool {
    e.parts
        .values()
        .any(|p| !p.severed && p.functions.contains(function))
}

/// A seeded draw: a generated world whose catalogue holds every function
/// whose index is `slice` modulo `SLICES`, and sixteen more drawn. Each is
/// used by one process, spending energy and marking the part, and rested by
/// another, which reads the mark and clears it. Members take drawn parts,
/// each growing what its shape grows, acquiring drawn functions, and some
/// severed; every function is expressed by some live part. One function's
/// part is worn on each member's own draw, which splits cohorts.
fn drawn(seed: u64, slice: usize) -> (Genesis, BTreeSet<usize>) {
    let random = |domain: &str, i: u64| isocosm::draw(seed, domain, &[i]);
    let mut g = Founding {
        seed,
        sites: 2,
        population: 18,
        lineages: 2,
        cohort_size: 3,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let all = every_function();
    let mut chosen: BTreeSet<usize> = (slice..all.len()).step_by(SLICES).collect();
    chosen.extend((0..16).map(|k| random("extra", k) as usize % all.len()));
    g.rules.shapes = default_shapes();
    g.rules.functions = chosen.iter().map(|&i| all[i].clone()).collect();
    g.rules.traits.extend([USED.into(), WORN.into()]);
    let transition = Causation::Transition;
    for &i in &chosen {
        let (f, n) = (&all[i].0, i as u64);
        let needs = vec![
            expresses(f),
            Query::Account {
                who: Binding::Actor,
                key: ENERGY.into(),
                at_least: 1,
            },
        ];
        let effects = vec![energy(1), mark(USED, true)];
        let mut work = process(&format!("test:use-{i}"), transition, needs, effects);
        work.period = Some(1 + random("period", n) % 3);
        work.priority = (random("priority", n) % 3) as i32;
        let needs = vec![expresses(f), marked(USED)];
        let effects = vec![mark(USED, false), energy(-1)];
        let mut rest = process(&format!("test:rest-{i}"), transition, needs, effects);
        rest.period = Some(2 + random("rest", n) % 3);
        rest.priority = (random("rest-priority", n) % 3) as i32;
        for p in [work, rest] {
            g.rules.processes.insert(p.id.clone(), p);
        }
    }
    let first = &all[*chosen.first().unwrap()].0;
    let mut stumble = process(
        "test:stumble",
        Causation::Choice,
        vec![expresses(first)],
        vec![],
    );
    stumble.risk = Some(Risk {
        per_million: 250_000,
        effects: vec![mark(WORN, true)],
    });
    stumble.period = Some(2);
    let needs = vec![expresses(first), marked(WORN)];
    let mut mend = process("test:mend", transition, needs, vec![mark(WORN, false)]);
    mend.period = Some(5);
    for p in [stumble, mend] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    let groups = &g.population.groups;
    let firsts: Vec<Id> = groups
        .iter()
        .filter(|(_, c)| c.entity.kingdom != "kingdom:world")
        .map(|(first, _)| *first)
        .collect();
    for &member in &firsts {
        let mut parts = BTreeMap::new();
        for k in 0..1 + random("parts", member) % 4 {
            let at = member * 8 + k;
            let shape = SHAPES[(random("shape", at) % 8) as usize];
            let mut functions = g.rules.grown(shape);
            let acquired = g.rules.functions.iter().filter(|(f, def)| {
                def.seeding == Seeding::Acquired
                    && def.shapes.contains(shape)
                    && random(f, at) % 3 == 0
            });
            functions.extend(acquired.map(|(f, _)| f.clone()));
            let mut p = part(shape, functions, random("severed", at) % 6 == 0);
            p.parent = (k > 0).then(|| random("parent", at) % k);
            parts.insert(k, p);
        }
        let e = &mut g.population.groups.get_mut(&member).unwrap().entity;
        e.parts = parts;
        e.accounts
            .insert(ENERGY.into(), 100 + random("energy", member) % 100);
    }
    for &i in &chosen {
        let (f, def) = &all[i];
        if g.population.groups.values().any(|c| live(&c.entity, f)) {
            continue;
        }
        let home = firsts[(random("home", i as u64) % firsts.len() as u64) as usize];
        let shapes: Vec<&Key> = def.shapes.iter().collect();
        let shape = shapes[(random("home-shape", i as u64) % shapes.len() as u64) as usize];
        let mut functions = g.rules.grown(shape);
        functions.insert(f.clone());
        let e = &mut g.population.groups.get_mut(&home).unwrap().entity;
        let id = e.parts.keys().last().map_or(0, |k| k + 1);
        e.parts.insert(id, part(shape, functions, false));
    }
    (g, chosen)
}

/// What a run leaves: its state, its matter, the members its evaluations
/// stood for, accepted and blocked, and each watched process's acts, by the
/// members they stood for.
type Run = (Key, u128, [u64; 3], BTreeMap<Key, u64>);

fn run(g: &Genesis, mode: Execution, reload: bool) -> (Run, u64) {
    let mut session = Session::new(g.clone(), mode).unwrap();
    for p in g.rules.processes.keys().filter(|p| p.starts_with("test:")) {
        session.sim.watch(p);
    }
    let work = session.advance(TICKS).unwrap();
    let mut acts: BTreeMap<Key, u64> = BTreeMap::new();
    for w in session.sim.take_watched() {
        *acts.entry(w.process).or_default() += w.count;
    }
    // The world, parts and marks included, saves and loads in the other
    // mode.
    let other = match mode {
        Execution::Individuals => Execution::Grouped,
        Execution::Grouped => Execution::Individuals,
    };
    if reload {
        let loaded = Session::load(session.save(), other).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
    }
    let counts = [work.represented, work.accepted, work.blocked];
    let sim = &session.sim;
    (
        (sim.state_hash(), sim.matter(), counts, acts),
        work.evaluations,
    )
}

#[test]
fn draws_over_the_whole_catalogue_agree_individually_and_grouped() {
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let (mut rested, mut mended) = (0, 0);
    let (mut individual, mut grouped) = (0, 0);
    for d in 0..SLICES as u64 {
        let (g, chosen) = drawn(338_000 + d, d as usize);
        let (one, evaluations) = run(&g, Execution::Individuals, d == 0);
        individual += evaluations;
        let (all, evaluations) = run(&g, Execution::Grouped, d == 0);
        grouped += evaluations;
        assert_eq!(one, all, "draw {d}");
        let acts = &one.3;
        let acted = |p: String| acts.get(&p).is_some_and(|n| *n > 0);
        used.extend(chosen.iter().filter(|i| acted(format!("test:use-{i}"))));
        rested += chosen
            .iter()
            .filter(|i| acted(format!("test:rest-{i}")))
            .count();
        mended += usize::from(acted("test:mend".into()));
    }
    let missing: Vec<usize> = (0..510).filter(|i| !used.contains(i)).collect();
    assert!(missing.is_empty(), "never bound: {missing:?}");
    // The marks were read back through the binding, worn parts mended, and
    // the grouped runs acted for cohorts at once.
    assert!(rested > 0 && mended > 0, "{rested} rested, {mended} mended");
    assert!(
        grouped < individual,
        "{grouped} grouped, {individual} individual"
    );
}
