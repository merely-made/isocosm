// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founding, bodied::Bodies, directing::Played, rules::*};
use std::sync::mpsc;

mod matrix;

const EPOCH: Tick = 60;

#[derive(Clone, Copy)]
enum Cadence {
    Original,
    Candidate,
}

impl Cadence {
    fn birth(self) -> u64 {
        match self {
            Self::Original => 30,
            Self::Candidate => 12,
        }
    }

    fn birth_period(self, lineage: u64) -> Tick {
        match (self, lineage) {
            (Self::Candidate, 1) => 14,
            _ => 7,
        }
    }

    fn meal(self, lineage: u32) -> u64 {
        match (self, lineage) {
            (Self::Candidate, 1) => 3,
            _ => 1,
        }
    }

    fn lifespan(self, seed: u64, lineage: u64) -> Tick {
        let base = match self {
            Self::Original => 20,
            Self::Candidate => 80,
        };
        base + crate::draw(seed, "lifespan", &[lineage]) % 41
    }
}

fn world(seed: u64, bodied: bool, original: bool) -> Genesis {
    let mut g = Founding {
        seed,
        sites: 3,
        population: 60,
        cohort_size: 2,
        lineages: 3,
        ecology: true,
        bodies: bodied.then(Bodies::default),
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.epoch_ticks = EPOCH;
    // Absolute profiles preserve the unchanged old control after any later
    // promotion of generator defaults. These are the declared stock_min=8
    // rates from ecology.rs. The candidate consumer has meal 3 and birth
    // period 14; other birth periods, provision and upkeep stay fixed.
    let cadence = if original {
        Cadence::Original
    } else {
        Cadence::Candidate
    };
    for i in 0..3 {
        let birth = g
            .rules
            .processes
            .get_mut(&format!("ecology:birth-{i}"))
            .unwrap();
        birth.period = Some(cadence.birth_period(i));
        for q in &mut birth.requires {
            if let Query::Account { at_least, .. } = q {
                *at_least = cadence.birth();
            }
        }
        for q in &mut g
            .rules
            .processes
            .get_mut(&format!("ecology:age-{i}"))
            .unwrap()
            .requires
        {
            if let Query::Age { at_least } = q {
                *at_least = cadence.lifespan(seed, i);
            }
        }
        if i > 0 {
            for effect in &mut g
                .rules
                .processes
                .get_mut(&format!("ecology:feed-{i}"))
                .unwrap()
                .effects
            {
                if let Effect::Eat { amount, .. } = effect {
                    *amount = Amount::Fixed(cadence.meal(i as u32));
                }
            }
        }
    }
    for i in 0..3 {
        let birth = &g.rules.processes[&format!("ecology:birth-{i}")];
        assert_eq!(birth.period, Some(cadence.birth_period(i)));
        assert!(matches!(&birth.effects[..], [Effect::Birth { provision }]
            if provision == &std::collections::BTreeMap::from([(format!("matter:{i}-0"), 8)])));
        let upkeep = &g.rules.processes[&format!("ecology:upkeep-{i}")];
        assert_eq!(upkeep.period, Some(5));
        assert!(matches!(&upkeep.effects[0], Effect::Transform { take, .. }
            if take == &std::collections::BTreeMap::from([(format!("matter:{i}-0"), 1)])));
        if i > 0 {
            let feed = &g.rules.processes[&format!("ecology:feed-{i}")];
            assert_eq!(feed.period, Some(3));
            assert!(matches!(
                &feed.effects[..],
                [Effect::Eat {
                    amount: Amount::Fixed(amount),
                    ..
                }] if *amount == cadence.meal(i as u32)
            ));
        }
    }
    g
}

#[derive(Default, Debug, serde::Serialize)]
struct Outcome {
    genesis: String,
    tick: Tick,
    births: usize,
    heirs: usize,
    born_heirs: usize,
    boundaries: usize,
    living: u64,
    living_founded: u64,
    living_born: u64,
    peak_members: u64,
}

impl Outcome {
    fn qualifies(&self) -> bool {
        self.tick == EPOCH * 3
            && self.living_born > 0
            && self.living_founded == 0
            && self.births > 0
            && self.born_heirs > 0
            && self.boundaries == 3
    }
}

#[derive(Default, serde::Serialize)]
struct Pilot {
    founder: Id,
    firstborn: Option<Id>,
    holdings: Vec<serde_json::Value>,
    flows: Vec<crate::flows::Flow>,
    accepted: Vec<crate::watch::Watched>,
    happenings: Vec<String>,
}

fn receipt_directory() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(
        std::env::var_os("ISOCOSM_REPRO_RECEIPT_DIR")
            .expect("set ISOCOSM_REPRO_RECEIPT_DIR to the existing receipt directory"),
    );
    let owner = std::path::Path::new(
        r"C:\Users\mark_\Code\testing\isometry\after-pass-2026-10-10\q3-reproduction",
    );
    assert!(
        path == owner && path.is_absolute() && path.is_dir(),
        "ISOCOSM_REPRO_RECEIPT_DIR must name the existing primary reproduction owner"
    );
    path
}

fn retain<T: serde::Serialize>(name: &str, value: &T) {
    let path = receipt_directory().join(name);
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    if path.exists() {
        assert_eq!(
            std::fs::read(path).unwrap(),
            bytes,
            "preserve differing evidence"
        );
    } else {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true)
            .open(path).expect("fresh primary pilot receipt; never overwrite");
        file.write_all(&bytes).unwrap();
        file.sync_all().unwrap();
    }
}

fn played(g: Genesis, observe: bool) -> Outcome {
    played_recorded(g, observe, None)
}

fn played_recorded(g: Genesis, observe: bool, draw: Option<matrix::Draw>) -> Outcome {
    let mut record = draw.map(|draw| matrix::Record::begin(draw, &g));
    // Generated founders also carry Born provenance. Identity, not that
    // label, distinguishes births occurring during this played run.
    let first_birth = g.population.next_id;
    let genesis = crate::digest(&g);
    let mut i = Interim::found(
        g,
        Start::default(),
        Mode::Survival,
        Pace {
            round: 1,
            scoring: 1,
        },
        Execution::Individuals,
    )
    .unwrap();
    let mut pilot = observe.then(|| Pilot {
        founder: i.critter().unwrap(),
        ..Default::default()
    });
    if observe {
        let watches: Vec<_> = i
            .session
            .sim
            .genesis()
            .rules
            .processes
            .keys()
            .cloned()
            .collect();
        for process in watches {
            i.session.sim.watch(&process);
        }
    }
    let matter = i.session.sim.matter();
    if let Some(record) = &mut record {
        record.start(i.session.sim.state().tick, matter);
    }
    let mut out = Outcome {
        genesis,
        ..Default::default()
    };
    for _ in 0..EPOCH * 3 {
        let (happened, flows) = if observe {
            i.round_with_flows(&no_candidates).unwrap()
        } else {
            (i.round(&no_candidates).unwrap(), vec![])
        };
        if let Some(pilot) = &mut pilot {
            let tick = i.session.sim.state().tick;
            for h in &happened {
                if let Happening::Born { parent, child } = h {
                    if *parent == pilot.founder && pilot.firstborn.is_none() {
                        pilot.firstborn = Some(*child);
                    }
                }
                pilot.happenings.push(format!("tick={tick} {h:?}"));
            }
            for id in std::iter::once(pilot.founder).chain(pilot.firstborn) {
                let body = i.session.sim.state().population.get(id);
                let own = body
                    .map(|e| crate::anatomy::held(e, &i.session.sim.genesis().rules, "matter:1-0"));
                eprintln!("PILOT HELD tick={tick} id={id} own={own:?}");
                pilot
                    .holdings
                    .push(serde_json::json!({ "tick": tick, "id": id, "own": own, "body": body }));
            }
            pilot.flows.extend(flows);
            pilot.accepted.extend(i.session.sim.take_watched());
        }
        for h in happened {
            match h {
                Happening::Born { .. } => out.births += 1,
                Happening::Died {
                    next: Some(next), ..
                } => {
                    out.heirs += 1;
                    if next >= first_birth {
                        out.born_heirs += 1;
                    }
                },
                Happening::Boundary { .. } => out.boundaries += 1,
                _ => {},
            }
        }
        if let Some(record) = &mut record {
            record.tick(i.session.sim.state().tick, i.session.sim.matter());
        }
        assert_eq!(
            i.session.sim.matter(),
            matter,
            "tick {}",
            i.session.sim.state().tick
        );
        out.tick = i.session.sim.state().tick;
        out.peak_members = out
            .peak_members
            .max(i.session.sim.state().population.count());
        out.living = boundary::standing(&i.session, "lineage:1").1;
        out.living_founded = 0;
        out.living_born = 0;
        for (&first, g) in &i.session.sim.state().population.groups {
            if g.entity.alive && g.entity.lineage == "lineage:1" {
                for id in first..first + g.count {
                    if id < first_birth {
                        out.living_founded += 1;
                    } else {
                        out.living_born += 1;
                    }
                }
            }
        }
        if out.living == 0 {
            break;
        }
    }
    let saved = i.session.save();
    if let Some(pilot) = pilot {
        let prefix = format!("q3-primary-final-state-pilot-seed0-{}", out.genesis);
        retain(&format!("{prefix}-session.json"), &saved);
        retain(&format!("{prefix}-readings.json"), &pilot);
        retain(&format!("{prefix}-state.json"), i.session.sim.state());
        retain(&format!("{prefix}-outcome.json"), &out);
    }
    for mode in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(saved.clone(), mode).unwrap();
        let actual = replay.sim.state_hash();
        let expected = i.session.sim.state_hash();
        assert_eq!(actual, expected, "{mode:?} replay");
        if let Some(record) = &mut record {
            record.replay(mode, expected, actual);
        }
    }
    if let Some(record) = record {
        record.finish(&saved, i.session.sim.state(), &out);
    }
    out
}

fn sweep(bodied: bool, original: bool, pilot: Option<Outcome>) -> Vec<(u64, Outcome)> {
    let skip_zero = pilot.is_some();
    if let Some(out) = &pilot {
        assert!(
            !bodied && !original,
            "pilot is the fresh bare candidate seed 0"
        );
        assert_eq!(out.genesis, crate::digest(&world(0, false, false)));
        assert!(out.qualifies());
    }
    let (send, receive) = mpsc::channel();
    let mut rows = std::thread::scope(|scope| {
        for worker in 0..1 {
            let send = send.clone();
            scope.spawn(move || {
                for seed in (worker..20).step_by(1) {
                    if skip_zero && seed == 0 {
                        continue;
                    }
                    let g = world(seed, bodied, original);
                    eprintln!(
                        "BEGIN bodied={bodied} original={original} seed={seed} genesis={}",
                        crate::digest(&g)
                    );
                    let out = played_recorded(
                        g, false, Some(matrix::Draw { seed, bodied, original }),
                    );
                    send.send((seed, out)).unwrap();
                }
            });
        }
        drop(send);
        let mut rows = Vec::new();
        for (seed, out) in receive {
            eprintln!("completed bodied={bodied} original={original} seed={seed}: {out:?}");
            rows.push((seed, out));
        }
        rows
    });
    if let Some(out) = pilot {
        rows.push((0, out));
    }
    rows.sort_by_key(|(seed, _)| *seed);
    assert_eq!(rows.len(), 20);
    for (seed, out) in &rows {
        eprintln!(
            "SORTED bodied={bodied} original={original} seed={seed} genesis={}: {out:?}",
            out.genesis
        );
    }
    eprintln!(
        "TOTAL bodied={bodied} original={original}: {}/20",
        rows.iter().filter(|(_, out)| out.qualifies()).count()
    );
    rows
}

fn candidate_pilot() -> Outcome {
    eprintln!("PILOT receipt_dir={}", receipt_directory().display());
    let g = world(0, false, false);
    eprintln!(
        "PILOT BEGIN profile=meal3-birth14 seed=0 genesis={}",
        crate::digest(&g)
    );
    let pilot = played(g, true);
    eprintln!("PILOT completed seed=0: {pilot:?}");
    assert!(
        pilot.qualifies(),
        "classify the failed seed 0 pilot before a matrix"
    );
    pilot
}

#[test]
#[ignore = "bounded seed 0 gate before a fresh declared matrix"]
fn consumer_reproduction_candidate_pilot() {
    candidate_pilot();
}

#[test]
#[ignore = "fresh bounded original-rate seed 0 negative control"]
fn consumer_reproduction_original_pilot() {
    eprintln!("ORIGINAL receipt_dir={}", receipt_directory().display());
    let g = world(0, false, true);
    eprintln!("ORIGINAL BEGIN seed=0 genesis={}", crate::digest(&g));
    let out = played(g, true);
    eprintln!("ORIGINAL completed seed=0: {out:?}");
    assert!(!out.qualifies(), "original-rate seed 0 unexpectedly qualifies");
    assert_eq!(out.living, 0, "original-rate seed 0 lineage ends");
}

#[test]
#[ignore = "declared cadence trial, Reading not ruled; run explicitly"]
fn consumer_reproduction_profile_probe() {
    for bodied in [false, true] {
        let rows = sweep(bodied, false, None);
        assert!(
            rows.iter().all(|(_, out)| out.qualifies()),
            "candidate bodied={bodied}"
        );
    }
    // Once a candidate qualifies in both forms, rerun its unchanged original
    // rate controls on the same native implementation and declared seeds.
    for bodied in [false, true] {
        for (seed, out) in sweep(bodied, true, None) {
            assert!(!out.qualifies(), "original bodied={bodied} seed={seed}");
            assert_eq!(out.living, 0, "original lineage ends");
        }
    }
}

fn bodied_preparation_name(original: bool) -> String {
    let arm = if original { "original" } else { "candidate" };
    format!("q3-primary-final-state-bodied-seed0-prepared-{arm}-genesis.json")
}

fn prepared_bodied(original: bool) -> Genesis {
    let g = world(0, true, original);
    let path = receipt_directory().join(bodied_preparation_name(original));
    let prepared: Genesis = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(g, prepared, "fresh bodied Genesis must equal reviewed preparation");
    g
}

#[test]
#[ignore = "Genesis-only preparation; no Session, play, tick or replay"]
fn consumer_reproduction_bodied_prepare() {
    eprintln!("BODIED PREP receipt_dir={}", receipt_directory().display());
    for original in [true, false] {
        let g = world(0, true, original);
        assert_eq!(g.seed, 0);
        assert_eq!(g.founding.as_ref().unwrap().bodies, Some(Bodies::default()));
        assert_eq!(g.rules.body(), BodyRules::default());
        assert!(!g.rules.kinds.is_empty() && !g.rules.functions.is_empty()
            && !g.rules.shapes.is_empty(), "native body catalogue");
        for key in ["lineage:0", "lineage:1"] {
            let d = g.lineages[key].development.as_ref().expect("native body development");
            assert!(!d.recipe.tagmata.is_empty(), "native body recipe");
            assert!(d.recipe.kinds().is_subset(&d.lexicon), "recipe kind lexicon");
            assert!(d.lexicon.iter().all(|k| g.rules.kinds.contains_key(k)),
                "catalogued development kinds");
        }
        let mut founders = 0u64;
        let mut consumers = 0u64;
        for c in g.population.groups.values() {
            let e = &c.entity;
            if g.lineages[&e.lineage].development.is_none() {
                continue;
            }
            founders += c.count;
            if e.lineage == "lineage:1" {
                consumers += c.count;
            }
            assert!(e.body.is_some() && !e.parts.is_empty(), "attached founder anatomy");
            assert!(e.parts.keys().all(|id| e.bodied(*id) && e.situs(*id).is_some()),
                "sited founder parts");
            assert!(e.parts.values().flat_map(|p| &p.functions)
                .all(|f| g.rules.functions.contains_key(f)), "catalogued founder functions");
        }
        assert!(founders > 0 && consumers > 0, "native bodied founders and played consumers");
        let arm = if original { "original" } else { "candidate" };
        retain(&bodied_preparation_name(original), &g);
        eprintln!("BODIED PREPARED arm={arm} seed=0 genesis={}", crate::digest(&g));
    }
    eprintln!("BODIED PREP completed seed=0 no_play=true");
}

#[test]
#[ignore = "bounded bodied original seed0 after reviewed Genesis preparation"]
fn consumer_reproduction_bodied_original_pilot() {
    let g = prepared_bodied(true);
    eprintln!("BODIED ORIGINAL BEGIN seed=0 genesis={}", crate::digest(&g));
    let out = played(g, true);
    eprintln!("BODIED ORIGINAL completed seed=0: {out:?}");
    assert!(!out.qualifies(), "original-rate bodied seed0 unexpectedly qualifies");
    assert_eq!(out.living, 0, "original-rate bodied seed0 lineage ends");
}

#[test]
#[ignore = "bounded bodied candidate seed0 after exact original negative"]
fn consumer_reproduction_bodied_candidate_pilot() {
    let g = prepared_bodied(false);
    eprintln!("BODIED PILOT BEGIN profile=meal3-birth14 seed=0 genesis={}", crate::digest(&g));
    let out = played(g, true);
    eprintln!("BODIED PILOT completed seed=0: {out:?}");
    assert!(out.qualifies(), "classify the failed seed 0 pilot before a matrix");
}
