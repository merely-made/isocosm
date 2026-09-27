// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 5's draws (Part B, steps 1 to 3), each run individually and
//! grouped, one JSON line a draw and a summary last. Seeds follow from one
//! master seed, taken from the clock unless given; the clock is the host's,
//! never the sim's.
//!
//!   c5-draws parts <draws> [master]   the whole function catalogue (X1)
//!   c5-draws flows <draws> [master]   declared conversions, placements and
//!                                     the flow record, reconciled each step
//!
//! `parts` builds the draw world of `tests/parts.rs`, `flows` that of
//! `tests/flows/draws.rs`, copied here unchanged in what they draw.

use isocosm::{
    Execution, Founding, Session,
    flows::{Flow, Holder, MadeBy},
    history::Command,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let draws: u64 = args[2].parse().unwrap();
    let master: u64 = match args.get(3) {
        Some(s) => s.parse().unwrap(),
        None => {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
            t.as_nanos() as u64
        },
    };
    eprintln!("master seed {master}");
    match args[1].as_str() {
        "parts" => parts::run_all(master, draws),
        "flows" => flows::run_all(master, draws),
        other => panic!("unknown family {other}"),
    }
}

mod parts {
    use super::*;

    const SLICES: usize = 16;
    const TICKS: Tick = 10;
    const USED: &str = "test:used";
    const WORN: &str = "test:worn";
    const ENERGY: &str = "sim:energy";

    fn part(shape: &str, functions: impl IntoIterator<Item = Key>, severed: bool) -> Part {
        Part { parent: None, traits: BTreeSet::new(), severed, shape: shape.into(), functions: functions.into_iter().collect() }
    }
    fn process(id: &str, causation: Causation, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
        Process { id: id.into(), causation, requires, commitments: vec![], effects, risk: None, target: None, period: None, priority: 0, need_account: None, need_below: 0, glyphs: BTreeSet::new(), invariants: BTreeSet::new(), note: false }
    }
    fn expresses(f: &str) -> Query {
        Query::Expresses { function: f.into() }
    }
    fn marked(key: &str) -> Query {
        Query::Trait { who: Binding::Part, key: key.into() }
    }
    fn mark(key: &str, present: bool) -> Effect {
        Effect::Trait { who: Binding::Part, key: key.into(), present }
    }
    fn energy(amount: i64) -> Effect {
        let entry = BTreeMap::from([(ENERGY.to_string(), amount.unsigned_abs())]);
        let (take, give) = if amount > 0 { (entry, BTreeMap::new()) } else { (BTreeMap::new(), entry) };
        Effect::Transform { who: Binding::Actor, take, give, conversion: None }
    }
    fn every_function() -> Vec<(Key, Function)> {
        let mut all = vec![];
        for mask in 1u32..256 {
            let shapes: BTreeSet<Key> = (0..8).filter(|i| mask >> i & 1 == 1).map(|i| SHAPES[i].to_string()).collect();
            for (seeding, name) in [(Seeding::Grown, "grown"), (Seeding::Acquired, "acquired")] {
                all.push((format!("function:{mask}-{name}"), Function { shapes: shapes.clone(), seeding }));
            }
        }
        all
    }
    fn live(e: &Entity, function: &str) -> bool {
        e.parts.values().any(|p| !p.severed && p.functions.contains(function))
    }
    fn drawn(seed: u64, slice: usize) -> (Genesis, BTreeSet<usize>) {
        let random = |domain: &str, i: u64| isocosm::draw(seed, domain, &[i]);
        let mut g = Founding { seed, sites: 2, population: 18, lineages: 2, cohort_size: 3, ..Default::default() }.generate().unwrap();
        let all = every_function();
        let mut chosen: BTreeSet<usize> = (slice..all.len()).step_by(SLICES).collect();
        chosen.extend((0..16).map(|k| random("extra", k) as usize % all.len()));
        g.rules.shapes = default_shapes();
        g.rules.functions = chosen.iter().map(|&i| all[i].clone()).collect();
        g.rules.traits.extend([USED.into(), WORN.into()]);
        let transition = Causation::Transition;
        for &i in &chosen {
            let (f, n) = (&all[i].0, i as u64);
            let needs = vec![expresses(f), Query::Account { who: Binding::Actor, key: ENERGY.into(), at_least: 1 }];
            let mut work = process(&format!("test:use-{i}"), transition, needs, vec![energy(1), mark(USED, true)]);
            work.period = Some(1 + random("period", n) % 3);
            work.priority = (random("priority", n) % 3) as i32;
            let mut rest = process(&format!("test:rest-{i}"), transition, vec![expresses(f), marked(USED)], vec![mark(USED, false), energy(-1)]);
            rest.period = Some(2 + random("rest", n) % 3);
            rest.priority = (random("rest-priority", n) % 3) as i32;
            for p in [work, rest] {
                g.rules.processes.insert(p.id.clone(), p);
            }
        }
        let first = &all[*chosen.first().unwrap()].0;
        let mut stumble = process("test:stumble", Causation::Choice, vec![expresses(first)], vec![]);
        stumble.risk = Some(Risk { per_million: 250_000, effects: vec![mark(WORN, true)] });
        stumble.period = Some(2);
        let mut mend = process("test:mend", transition, vec![expresses(first), marked(WORN)], vec![mark(WORN, false)]);
        mend.period = Some(5);
        for p in [stumble, mend] {
            g.rules.processes.insert(p.id.clone(), p);
        }
        let firsts: Vec<Id> = g.population.groups.iter().filter(|(_, c)| c.entity.kingdom != "kingdom:world").map(|(f, _)| *f).collect();
        for &member in &firsts {
            let mut parts = BTreeMap::new();
            for k in 0..1 + random("parts", member) % 4 {
                let at = member * 8 + k;
                let shape = SHAPES[(random("shape", at) % 8) as usize];
                let mut functions = g.rules.grown(shape);
                let acquired = g.rules.functions.iter().filter(|(f, def)| def.seeding == Seeding::Acquired && def.shapes.contains(shape) && random(f, at) % 3 == 0);
                functions.extend(acquired.map(|(f, _)| f.clone()));
                let mut p = part(shape, functions, random("severed", at) % 6 == 0);
                p.parent = (k > 0).then(|| random("parent", at) % k);
                parts.insert(k, p);
            }
            let e = &mut g.population.groups.get_mut(&member).unwrap().entity;
            e.parts = parts;
            e.accounts.insert(ENERGY.into(), 100 + random("energy", member) % 100);
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
    type Run = (Key, u128, [u64; 3], BTreeMap<Key, u64>);
    fn run(g: &Genesis, mode: Execution) -> (Run, u64) {
        let mut session = Session::new(g.clone(), mode).unwrap();
        for p in g.rules.processes.keys().filter(|p| p.starts_with("test:")) {
            session.sim.watch(p);
        }
        let work = session.advance(TICKS).unwrap();
        let mut acts: BTreeMap<Key, u64> = BTreeMap::new();
        for w in session.sim.take_watched() {
            *acts.entry(w.process).or_default() += w.count;
        }
        let sim = &session.sim;
        ((sim.state_hash(), sim.matter(), [work.represented, work.accepted, work.blocked], acts), work.evaluations)
    }

    pub fn run_all(master: u64, draws: u64) {
        let mut covered = vec![0u64; 510];
        let (mut agreed, mut individual, mut grouped) = (0u64, 0u64, 0u64);
        for d in 0..draws {
            let seed = isocosm::draw(master, "c5-parts", &[d]);
            let slice = (d % SLICES as u64) as usize;
            let (g, chosen) = drawn(seed, slice);
            let (one, e1) = run(&g, Execution::Individuals);
            let (all, e2) = run(&g, Execution::Grouped);
            let agree = one == all;
            agreed += u64::from(agree);
            individual += e1;
            grouped += e2;
            let used: Vec<usize> = chosen.iter().copied().filter(|i| one.3.get(&format!("test:use-{i}")).is_some_and(|n| *n > 0)).collect();
            for &i in &used {
                covered[i] += 1;
            }
            let rested = chosen.iter().filter(|i| one.3.get(&format!("test:rest-{i}")).is_some_and(|n| *n > 0)).count();
            println!("{}", json!({
                "draw": d, "seed": seed, "slice": slice, "agree": agree, "state_hash": one.0,
                "represented": one.2[0], "accepted": one.2[1], "blocked": one.2[2],
                "functions": chosen.len(), "bound": used.len(), "rested": rested,
                "evaluations_individual": e1, "evaluations_grouped": e2,
            }));
        }
        let never: Vec<usize> = (0..510).filter(|i| covered[*i] == 0).collect();
        println!("{}", json!({
            "summary": "parts", "master": master, "draws": draws, "agreed": agreed,
            "functions_never_bound": never, "least_bound": covered.iter().min(), "most_bound": covered.iter().max(),
            "evaluations_individual": individual, "evaluations_grouped": grouped,
        }));
    }
}

mod flows {
    use super::*;

    const STEPS: u64 = 60;
    type Books = BTreeMap<(Holder, Key), i128>;

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
    fn claimed(flows: &[Flow]) -> Books {
        let mut claims = Books::new();
        for f in flows {
            let amount = i128::from(f.amount);
            for ((holder, key), sign) in [(&f.from, -1i128), (&f.to, 1)] {
                let mut claim = |holder: Holder, moved: i128| *claims.entry((holder, key.clone())).or_default() += sign * moved;
                match *holder {
                    Holder::Entity(first) => (first..first + f.count).for_each(|id| claim(Holder::Entity(id), amount)),
                    Holder::Site(_) => claim(*holder, amount * i128::from(f.count)),
                    Holder::Dev => {},
                }
            }
        }
        claims.retain(|_, v| *v != 0);
        claims
    }
    fn reconcile(before: &Books, after: &Books, flows: &[Flow]) -> Result<(), String> {
        let claims = claimed(flows);
        let accounts: BTreeSet<&(Holder, Key)> = before.keys().chain(after.keys()).chain(claims.keys()).collect();
        for account in accounts {
            let get = |books: &Books| books.get(account).copied().unwrap_or(0);
            let (moved, claim) = (get(after) - get(before), get(&claims));
            if moved != claim {
                return Err(format!("{account:?} moved {moved}; the record accounts for {claim}"));
            }
        }
        Ok(())
    }
    fn ecology(seed: u64) -> Genesis {
        Founding { seed, sites: 3, population: 60, lineages: 3, cohort_size: 4, ecology: true, ..Default::default() }.generate().unwrap()
    }
    fn process(id: &str, requires: Vec<Query>, effects: Vec<Effect>, period: Tick) -> Process {
        Process { id: id.into(), causation: Causation::Transition, requires, commitments: vec![], effects, risk: None, target: None, period: Some(period), priority: 3, need_account: None, need_below: 0, glyphs: BTreeSet::new(), invariants: BTreeSet::new(), note: false }
    }
    fn own(i: u32, at_least: u64) -> Vec<Query> {
        vec![
            Query::Alive(Binding::Actor),
            Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{i}") },
            Query::Account { who: Binding::Actor, key: format!("matter:{i}-0"), at_least },
        ]
    }
    fn declared(seed: u64) -> Genesis {
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
        let mut graze = process("test:graze", own(1, 0), vec![
            Effect::Transfer { from: Binding::Target, to: Binding::Actor, account: "matter:0-0".into(), amount: 1 },
            Effect::Transform { who: Binding::Actor, take: Ledger::from([("matter:0-0".into(), 1)]), give: Ledger::from([("matter:1-0".into(), 1)]), conversion: Some(Conversion::Digestion) },
        ], 4);
        graze.causation = Causation::Choice;
        graze.requires.push(Query::Account { who: Binding::Target, key: "matter:0-0".into(), at_least: 1 });
        graze.target = Some(Target { same_place: true, alive: Some(true), lineage: Some("lineage:0".into()), among: BTreeSet::new(), weighted: false });
        g.rules.processes.insert(graze.id.clone(), graze);
        for i in 0..3 {
            let respire = Effect::Transform { who: Binding::Actor, take: Ledger::from([(format!("matter:{i}-0"), 1)]), give: Ledger::from([("world:soil".into(), 1)]), conversion: Some(Conversion::Mineralization) };
            let p = process(&format!("test:respire-{i}"), own(i, 3), vec![respire], 5);
            g.rules.processes.insert(p.id.clone(), p);
        }
        g
    }
    fn command(seed: u64, step: u64, session: &Session) -> Option<Command> {
        let pick = |domain: &str, n: u64| isocosm::draw(seed, domain, &[step]) % n.max(1);
        let roll = pick("roll", 100);
        if roll < 60 {
            return None;
        }
        if roll < 72 {
            let accounts = ["world:soil", "matter:0-0", "matter:1-0", "matter:2-0"];
            return Some(Command::PlaceMatter { site: pick("site", 3), account: accounts[pick("account", 4) as usize].into(), amount: 1 + pick("amount", 50) });
        }
        let next = session.sim.state().population.next_id;
        let processes: Vec<&Key> = session.sim.genesis().rules.processes.keys().collect();
        let actor = pick("actor", next);
        let target = (pick("targeted", 2) == 0).then(|| pick("target", next));
        let process = processes[pick("process", processes.len() as u64) as usize];
        Some(Command::Act { actor, target, process: process.clone(), cause: None })
    }
    struct Outcome {
        steps: Vec<(Key, Books)>,
        flows: Vec<Flow>,
        unreconciled: Vec<String>,
        unconserved: u64,
        issued: u128,
    }
    fn run(seed: u64, mode: Execution) -> Outcome {
        let mut session = Session::new(declared(seed), mode).unwrap();
        session.sim.keep_flows();
        let founded = session.sim.matter();
        let mut out = Outcome { steps: vec![], flows: vec![], unreconciled: vec![], unconserved: 0, issued: 0 };
        for step in 0..STEPS {
            let drawn = command(seed, step, &session);
            let ticks = 1 + isocosm::draw(seed, "ticks", &[step]) % 3;
            let before = books(&session);
            let _ = match drawn {
                None => session.advance(ticks).map(|_| ()),
                Some(c) => session.command(c).map(|_| ()),
            };
            let flows = session.sim.take_flows();
            if let Err(why) = reconcile(&before, &books(&session), &flows) {
                out.unreconciled.push(format!("step {step}: {why}"));
            }
            if session.sim.matter() != founded + session.sim.issued() {
                out.unconserved += 1;
            }
            out.steps.push((session.sim.state_hash(), claimed(&flows)));
            out.flows.extend(flows);
        }
        out.issued = session.sim.issued();
        out
    }
    fn kinds(flows: &[Flow]) -> BTreeSet<String> {
        let name = |f: &Flow| match &f.made_by {
            MadeBy::Process(p) => p.split('-').next().unwrap().to_string(),
            MadeBy::Command(c) => format!("command {c}"),
        };
        flows.iter().map(name).collect()
    }

    pub fn run_all(master: u64, draws: u64) {
        let (mut agreed, mut reconciled, mut conserved) = (0u64, 0u64, 0u64);
        let (mut cohort_flows, mut all_kinds) = (0usize, BTreeSet::new());
        for d in 0..draws {
            let seed = isocosm::draw(master, "c5-flows", &[d]);
            let one = run(seed, Execution::Individuals);
            let all = run(seed, Execution::Grouped);
            let agree = one.steps == all.steps && one.issued == all.issued;
            let ok = one.unreconciled.is_empty() && all.unreconciled.is_empty();
            let kept = one.unconserved == 0 && all.unconserved == 0;
            agreed += u64::from(agree);
            reconciled += u64::from(ok);
            conserved += u64::from(kept);
            let cohorts = all.flows.iter().filter(|f| f.count > 1).count();
            cohort_flows += cohorts;
            let made = kinds(&one.flows);
            all_kinds.extend(made.iter().cloned());
            let first: Vec<&String> = one.unreconciled.iter().chain(&all.unreconciled).take(2).collect();
            println!("{}", json!({
                "draw": d, "seed": seed, "agree": agree, "reconciled": ok, "conserved": kept,
                "state_hash": one.steps.last().map(|s| &s.0), "issued": one.issued.to_string(),
                "flows_individual": one.flows.len(), "flows_grouped": all.flows.len(), "cohort_flows": cohorts,
                "made_by": made, "unreconciled": first,
            }));
        }
        println!("{}", json!({
            "summary": "flows", "master": master, "draws": draws, "steps_per_draw": STEPS,
            "agreed": agreed, "reconciled": reconciled, "conserved": conserved,
            "cohort_flows": cohort_flows, "made_by": all_kinds,
        }));
    }
}
