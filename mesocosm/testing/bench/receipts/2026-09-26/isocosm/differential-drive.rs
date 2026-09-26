// Differential driver: the same scenarios against two builds of the core.
// Prints one line per observation; diff the two outputs.
//
//   drive probe <master> <draws>     exact runs of drawn probe worlds
//   drive fuzz <seed> <worlds> <steps>  random acts and advances, receipts and hashes

use isocosm::{
    Execution, Founding, Simulation,
    probe::{ProbeFounding, run_exact},
    rules::*,
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

fn bare(sim: &Simulation) -> String {
    let mut state = sim.state().clone();
    state.population = state.population.normalized();
    isocosm::digest(&(sim.genesis().seed, &sim.genesis().world, &state))
}

fn probe(master: u64, draws: u64) {
    for k in 0..draws {
        let seed = isocosm::draw(master, "probe-world", &[k]);
        let world = ProbeFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let dynamics = isocosm::draw(master, "probe-dynamics", &[k, 0]);
        let run = run_exact(&world, dynamics, true).unwrap();
        println!(
            "draw {k} seed {seed} dynamics {dynamics} revision {} state_hash {} without_revision {} evaluations {} accepted {} groups {}",
            world.genesis.rules.revision(),
            run.sim.state_hash(),
            bare(&run.sim),
            run.work.evaluations,
            run.work.accepted,
            run.sim.state().population.groups.len()
        );
    }
}

fn process(id: &str, shape: Shape, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        shape,
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

/// Extra processes so every effect, binding and failure path is reached.
fn widen(g: &mut isocosm::simulation::Genesis, told: &str) {
    let r = &mut g.rules;
    r.traits.insert("fuzz:marked".into());
    r.accounts.insert("fuzz:skillful".into(), AccountKind::Energy);
    let soil = |from, to, amount| Effect::Transfer {
        from,
        to,
        account: "world:soil".into(),
        amount,
    };
    let mut add = |mut p: Process, note: bool, target: bool, period: Option<Tick>| {
        p.note = note;
        if target {
            p.target = Some(Target {
                same_place: false,
                alive: None,
                lineage: None,
            });
        }
        p.period = period;
        r.processes.insert(p.id.clone(), p);
    };
    add(process("fuzz:relate", Shape::Choice, vec![Effect::Relate { kind: "sim:owns".into(), present: true }]), true, true, None);
    add(process("fuzz:unrelate", Shape::Choice, vec![Effect::Relate { kind: "sim:owns".into(), present: false }]), false, false, None);
    add(process("fuzz:move", Shape::Choice, vec![Effect::Move { destination: 1 }]), true, false, None);
    add(process("fuzz:move-home", Shape::Choice, vec![Effect::Move { destination: 0 }]), false, false, None);
    add(process("fuzz:tell", Shape::Choice, vec![Effect::Tell { event: told.into() }]), true, false, None);
    add(process("fuzz:tell-twice", Shape::Choice, vec![Effect::Tell { event: told.into() }, Effect::Tell { event: told.into() }]), false, false, None);
    add(process("fuzz:mark", Shape::Choice, vec![Effect::Trait { who: Binding::Actor, key: "fuzz:marked".into(), present: true }, Effect::Trait { who: Binding::Target, key: "fuzz:marked".into(), present: false }]), false, false, None);
    add(process("fuzz:practice", Shape::Choice, vec![Effect::Practice { key: "fuzz:skill".into(), amount: 3 }]), false, false, Some(2));
    add(process("fuzz:weather", Shape::Choice, vec![Effect::Condition { key: "world:weather".into(), delta: -2 }]), false, false, None);
    add(process("fuzz:take-site", Shape::Choice, vec![soil(Binding::Place, Binding::Actor, 1)]), true, false, None);
    add(process("fuzz:give-site", Shape::Choice, vec![soil(Binding::Actor, Binding::Place, 1)]), false, false, None);
    add(process("fuzz:take-target", Shape::Choice, vec![soil(Binding::Target, Binding::Actor, 1)]), false, true, None);
    add(process("fuzz:swap", Shape::Choice, vec![soil(Binding::Actor, Binding::Target, 1), soil(Binding::Target, Binding::Actor, 2)]), true, false, None);
    add(process("fuzz:overdraw", Shape::Choice, vec![soil(Binding::Actor, Binding::Target, 1), soil(Binding::Actor, Binding::Target, 999)]), true, false, None);
    add(process("fuzz:leak", Shape::Choice, vec![Effect::Transform { who: Binding::Actor, take: BTreeMap::new(), give: BTreeMap::from([("fuzz:skillful".into(), 1)]) }]), false, false, None);
    add(process("fuzz:twins", Shape::Transition, vec![Effect::Birth { provision: BTreeMap::from([("world:soil".into(), 1)]) }, Effect::Move { destination: 2 }, Effect::Birth { provision: BTreeMap::from([("world:soil".into(), 1)]) }]), true, false, None);
    add(process("fuzz:found-twice", Shape::Transition, vec![Effect::FoundPolity { governance: "governance:consent".into(), focus: BTreeSet::new(), support: "world:soil".into() }, Effect::FoundPolity { governance: "governance:consent".into(), focus: BTreeSet::new(), support: "world:soil".into() }]), false, false, None);
    add(process("fuzz:found", Shape::Transition, vec![Effect::FoundPolity { governance: "governance:consent".into(), focus: BTreeSet::new(), support: "world:soil".into() }]), true, false, None);
    add(process("fuzz:reckon-twice", Shape::Transition, vec![Effect::Record { axis: "feat:reserve".into(), account: "world:soil".into() }, soil(Binding::Place, Binding::Actor, 1), Effect::Record { axis: "feat:reserve".into(), account: "world:soil".into() }]), true, false, None);
    add(process("fuzz:note-then-fail", Shape::Choice, vec![Effect::Note { kind: "sim:observed".into(), text: "x".into(), lifetime: Some(3) }, soil(Binding::Actor, Binding::Place, 999)]), true, false, None);
    add(process("fuzz:die-then-birth", Shape::Transition, vec![Effect::Death, Effect::Birth { provision: BTreeMap::new() }]), true, false, None);
    let mut risky = process("fuzz:risky", Shape::Choice, vec![soil(Binding::Actor, Binding::Place, 1)]);
    risky.risk = Some(Risk {
        per_million: 500_000,
        effects: vec![soil(Binding::Place, Binding::Actor, 1), Effect::Death],
    });
    add(risky, true, false, None);
    let mut committed = process("fuzz:committed", Shape::Choice, vec![soil(Binding::Actor, Binding::Target, 1)]);
    committed.commitments = vec![Effect::Condition { key: "world:habitable".into(), delta: 1 }];
    add(committed, false, false, None);
    let mut bulk = process("fuzz:bulk", Shape::Choice, vec![Effect::Trait { who: Binding::Actor, key: "fuzz:marked".into(), present: true }, Effect::Practice { key: "fuzz:bulk".into(), amount: 1 }]);
    bulk.requires.push(Query::Age { at_least: 2 });
    add(bulk, false, false, Some(3));
}

/// Worlds under tight operation budgets, so advances are refused part way,
/// every one of them played through a session: commands, advances across
/// epochs, and a save that must load again in the other mode.
fn tight(seed: u64, worlds: u64, steps: u64) {
    use isocosm::{Command, Session};
    let pick = |i: u64, domain: &str, n: u64| isocosm::draw(seed, domain, &[i]) % n.max(1);
    for w in 0..worlds {
        let founding = Founding {
            seed: seed ^ w,
            sites: 3,
            population: 12 + pick(w, "population", 40),
            cohort_size: 1 + pick(w, "cohort", 8),
            lineages: 1 + pick(w, "lineages", 4) as u32,
            ecology: w % 2 == 1,
            ..Default::default()
        };
        let mut g = founding.generate().unwrap();
        let told = format!(
            "event:{}",
            isocosm::digest(&(g.seed, 0u64, 0u64, 3u64, "sim:remember"))
        );
        widen(&mut g, &told);
        g.rules.limits.events_per_advance = 20 + pick(w, "budget", 400) as usize;
        g.rules.epoch_ticks = 1 + pick(w, "epoch", 6);
        if w % 3 == 2 {
            g.rules.limits.notes = 6 + pick(w, "notes", 30) as usize;
            g.rules.limits.history = 40 + pick(w, "history", 60) as usize;
            g.rules.limits.entities = g.population.count() + pick(w, "entities", 6);
        }
        let mode = if w % 4 < 2 { Execution::Individuals } else { Execution::Grouped };
        let mut session = match Session::new(g, mode) {
            Ok(s) => s,
            Err(why) => {
                println!("world {w} refused: {why}");
                continue;
            },
        };
        let processes: Vec<Key> = session.sim.genesis().rules.processes.keys().cloned().collect();
        let first = session.command(Command::Act {
            actor: 3,
            target: None,
            process: "sim:remember".into(),
            cause: None,
        });
        println!("world {w} first {first:?}");
        for s in 0..steps {
            let i = w * 1_000_000 + s;
            let roll = pick(i, "roll", 100);
            let next = session.sim.state().population.next_id;
            let outcome = if roll < 12 {
                let t = 1 + pick(i, "ticks", 9);
                format!("advance {t} {:?}", session.advance(t))
            } else if roll < 14 {
                format!("collect {:?}", session.command(Command::Collect))
            } else if roll < 16 {
                let id = pick(i, "inspect", next + 2);
                format!("inspect {id} {:?}", session.command(Command::Inspect(id)))
            } else if roll < 17 {
                let id = pick(i, "release", next + 2);
                format!("release {id} {:?}", session.command(Command::Release(id)))
            } else if roll < 19 {
                let entity = pick(i, "learner", next + 1);
                let event = told.clone();
                format!("learn {entity} {:?}", session.command(Command::Learn { entity, event }))
            } else {
                let actor = pick(i, "actor", next + 1);
                let target = match pick(i, "target-kind", 5) {
                    0 => None,
                    1 => Some(actor),
                    2 => Some(pick(i, "target-near", 3) + actor.saturating_sub(1)),
                    _ => Some(pick(i, "target", next + 1)),
                };
                let process = processes[pick(i, "process", processes.len() as u64) as usize].clone();
                let cause = match pick(i, "cause", 6) {
                    0 => Some("event:none".to_string()),
                    1 => session.sim.state().events.keys().next().cloned(),
                    _ => None,
                };
                format!("{:?}", session.command(Command::Act { actor, target, process, cause }))
            };
            println!(
                "{w}.{s} {outcome} {} groups {} checkpoints {}",
                session.sim.state_hash(),
                session.sim.state().population.groups.len(),
                session.checkpoints.len()
            );
        }
        let saved = session.save();
        let other = if mode == Execution::Grouped { Execution::Individuals } else { Execution::Grouped };
        let loaded = Session::load(saved.clone(), other).map(|s| s.sim.state_hash());
        println!("world {w} end {} saved {} loaded {loaded:?}", session.sim.state_hash(), isocosm::digest(&saved));
    }
}

fn fuzz(seed: u64, worlds: u64, steps: u64) {
    let pick = |i: u64, domain: &str, n: u64| isocosm::draw(seed, domain, &[i]) % n.max(1);
    for w in 0..worlds {
        let founding = Founding {
            seed: seed ^ w,
            sites: 3,
            population: 12 + pick(w, "population", 40),
            cohort_size: 1 + pick(w, "cohort", 8),
            lineages: 1 + pick(w, "lineages", 4) as u32,
            ecology: w % 2 == 1,
            ..Default::default()
        };
        let mut g = founding.generate().unwrap();
        // The first act is always actor 3's remember at tick 0, so its
        // event id is known before the rules are written.
        let told = format!(
            "event:{}",
            isocosm::digest(&(g.seed, 0u64, 0u64, 3u64, "sim:remember"))
        );
        widen(&mut g, &told);
        if w % 3 == 2 {
            g.rules.limits.notes = 6 + pick(w, "notes", 30) as usize;
            g.rules.limits.history = 4 + pick(w, "history", 20) as usize;
            g.rules.limits.entities = g.population.count() + pick(w, "entities", 6);
        }
        let mode = if w % 4 < 2 { Execution::Individuals } else { Execution::Grouped };
        let mut sim = match Simulation::new(g, mode) {
            Ok(s) => s,
            Err(why) => {
                println!("world {w} refused: {why}");
                continue;
            },
        };
        let processes: Vec<Key> = sim.genesis().rules.processes.keys().cloned().collect();
        let first = sim.execute(3, None, "sim:remember", None);
        println!("world {w} first {}", serde_json::to_string(&first).unwrap());
        for s in 0..steps {
            let i = w * 1_000_000 + s;
            let roll = pick(i, "roll", 100);
            if roll < 8 {
                let t = 1 + pick(i, "ticks", 3);
                match sim.advance(t) {
                    Ok(work) => println!("{w}.{s} advance {t} {work:?} {}", sim.state_hash()),
                    Err(why) => println!("{w}.{s} advance {t} refused {why} {}", sim.state_hash()),
                }
                continue;
            }
            if roll < 10 {
                let n = sim.collect();
                println!("{w}.{s} collect {n} {}", sim.state_hash());
                continue;
            }
            if roll < 12 {
                let id = pick(i, "inspect", sim.state().population.next_id + 2);
                let r = sim.inspect(id);
                println!("{w}.{s} inspect {id} {r:?} {}", sim.state_hash());
                continue;
            }
            if roll < 14 {
                let id = pick(i, "learner", sim.state().population.next_id + 1);
                let r = sim.learn(id, &told);
                println!("{w}.{s} learn {id} {r:?} {}", sim.state_hash());
                continue;
            }
            let next = sim.state().population.next_id;
            let actor = pick(i, "actor", next + 1);
            let target = match pick(i, "target-kind", 5) {
                0 => None,
                1 => Some(actor),
                2 => Some(pick(i, "target-near", 3) + actor.saturating_sub(1)),
                _ => Some(pick(i, "target", next + 1)),
            };
            let process = if pick(i, "bogus", 40) == 0 {
                "fuzz:absent".to_string()
            } else {
                processes[pick(i, "process", processes.len() as u64) as usize].clone()
            };
            let cause = match pick(i, "cause", 6) {
                0 => Some("event:none".to_string()),
                1 => sim.state().events.keys().next().cloned(),
                _ => None,
            };
            let r = sim.execute(actor, target, &process, cause);
            println!(
                "{w}.{s} {} {} groups {} matter {}",
                serde_json::to_string(&r).unwrap(),
                sim.state_hash(),
                sim.state().population.groups.len(),
                sim.matter()
            );
        }
        println!("world {w} end {} next_id {}", sim.state_hash(), sim.state().population.next_id);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = |i: usize| args[i].parse::<u64>().unwrap();
    match args[1].as_str() {
        "probe" => probe(n(2), n(3)),
        "fuzz" => fuzz(n(2), n(3), n(4)),
        "tight" => tight(n(2), n(3), n(4)),
        other => panic!("unknown mode {other}"),
    }
}
