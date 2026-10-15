// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founding, bodied::Bodies, directing::Played, rules::*};
use std::sync::mpsc;

mod matrix;
const EPOCH: Tick = 60;

mod profile;
use profile::{Form, world};

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
    /// Every fifth tick: living members by (lineage, site).
    census: Vec<(Tick, std::collections::BTreeMap<String, u64>)>,
}

fn receipt_directory() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(
        std::env::var_os("ISOCOSM_REPRO_RECEIPT_DIR")
            .expect("set ISOCOSM_REPRO_RECEIPT_DIR to the existing receipt directory"),
    );
    // The same owner on each machine: `~/Code/testing/isometry/...` (883).
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    let owner = std::path::PathBuf::from(home.expect("a home directory"))
        .join("Code/testing/isometry/after-pass-2026-10-10/q3-reproduction");
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
    played_recorded(g, observe.then(|| "seed0".to_string()), None)
}

fn played_named(g: Genesis, pilot: Option<String>) -> Outcome {
    played_recorded(g, pilot, None)
}

fn played_recorded(g: Genesis, pilot_name: Option<String>, draw: Option<matrix::Draw>) -> Outcome {
    let observe = pilot_name.is_some();
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
            if tick % 5 == 0 {
                let mut by = std::collections::BTreeMap::new();
                for g in i.session.sim.state().population.groups.values() {
                    if g.entity.alive && g.entity.lineage.starts_with("lineage:") {
                        *by.entry(format!("{}@{}", g.entity.lineage, g.entity.place))
                            .or_insert(0u64) += g.count;
                    }
                }
                pilot.census.push((tick, by));
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
        let name = pilot_name.unwrap();
        let prefix = if name == "seed0" {
            format!("q3-primary-final-state-pilot-seed0-{}", out.genesis)
        } else {
            format!("a1-pilot-v2-{name}-{}", out.genesis)
        };
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

fn sweep(form: Form, original: bool, pilot: Option<Outcome>) -> Vec<(u64, Outcome)> {
    let skip_zero = pilot.is_some();
    if let Some(out) = &pilot {
        assert!(
            form == Form::Bare && !original,
            "pilot is the fresh bare candidate seed 0"
        );
        assert_eq!(out.genesis, crate::digest(&world(0, Form::Bare, false)));
        assert!(out.qualifies());
    }
    let (send, receive) = mpsc::channel();
    let mut rows = std::thread::scope(|scope| {
        let workers = std::env::var("ISOCOSM_REPRO_WORKERS").map_or(1, |w| w.parse().unwrap());
        for worker in 0..workers {
            let send = send.clone();
            scope.spawn(move || {
                for seed in (worker..20).step_by(workers as usize) {
                    if skip_zero && seed == 0 {
                        continue;
                    }
                    let g = world(seed, form, original);
                    eprintln!(
                        "BEGIN form={} original={original} seed={seed} genesis={}", form.name(),
                        crate::digest(&g)
                    );
                    let out = played_recorded(
                        g, None, Some(matrix::Draw { seed, form, original }),
                    );
                    send.send((seed, out)).unwrap();
                }
            });
        }
        drop(send);
        let mut rows = Vec::new();
        for (seed, out) in receive {
            eprintln!("completed form={} original={original} seed={seed}: {out:?}", form.name());
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
            "SORTED form={} original={original} seed={seed} genesis={}: {out:?}",
            form.name(), out.genesis
        );
    }
    eprintln!(
        "TOTAL form={} original={original}: {}/20", form.name(),
        rows.iter().filter(|(_, out)| out.qualifies()).count()
    );
    rows
}

fn candidate_pilot() -> Outcome {
    eprintln!("PILOT receipt_dir={}", receipt_directory().display());
    let g = world(0, Form::Bare, false);
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
    let g = world(0, Form::Bare, true);
    eprintln!("ORIGINAL BEGIN seed=0 genesis={}", crate::digest(&g));
    let out = played(g, true);
    eprintln!("ORIGINAL completed seed=0: {out:?}");
    assert!(!out.qualifies(), "original-rate seed 0 unexpectedly qualifies");
    assert_eq!(out.living, 0, "original-rate seed 0 lineage ends");
}

#[test]
#[ignore = "declared cadence trial, Reading not ruled; run explicitly"]
fn consumer_reproduction_profile_probe() {
    for form in [Form::Bare, Form::Bodied] {
        let rows = sweep(form, false, None);
        assert!(
            rows.iter().all(|(_, out)| out.qualifies()),
            "candidate form={form:?}"
        );
    }
    // Once a candidate qualifies in both forms, rerun its unchanged original
    // rate controls on the same native implementation and declared seeds.
    for form in [Form::Bare, Form::Bodied] {
        for (seed, out) in sweep(form, true, None) {
            assert!(!out.qualifies(), "original form={form:?} seed={seed}");
            assert_eq!(out.living, 0, "original lineage ends");
        }
    }
}

fn bodied_preparation_name(original: bool) -> String {
    let arm = if original { "original" } else { "candidate" };
    format!("q3-primary-final-state-bodied-seed0-prepared-{arm}-genesis.json")
}

fn prepared_bodied(original: bool) -> Genesis {
    let g = world(0, Form::Bodied, original);
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
        let g = world(0, Form::Bodied, original);
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

fn arm() -> (Form, bool) {
    let arm = std::env::var("ISOCOSM_REPRO_ARM").expect("set ISOCOSM_REPRO_ARM, e.g. played-candidate");
    let (form, rate) = arm.split_once('-').expect("form-rate");
    let form = [Form::Bare, Form::Bodied, Form::Played]
        .into_iter().find(|f| f.name() == form).expect("a declared form");
    (form, match rate { "original" => true, "candidate" => false, _ => panic!("original or candidate") })
}

/// One declared arm of twenty draws. Candidate arms qualify on every draw;
/// original arms are the negative control and end on every draw.
#[test]
#[ignore = "one declared A1 arm; ISOCOSM_REPRO_ARM names it"]
fn consumer_reproduction_arm() {
    let (form, original) = arm();
    let rows = sweep(form, original, None);
    for (seed, out) in &rows {
        if original {
            assert!(!out.qualifies() && out.living == 0, "original {form:?} seed={seed}");
        }
    }
    if !original {
        assert!(rows.iter().all(|(_, out)| out.qualifies()), "candidate {form:?}");
    }
}

/// One seed-zero pilot of an arm, with full readings, before its twenty draws.
#[test]
#[ignore = "seed-0 pilot of one declared arm; ISOCOSM_REPRO_ARM names it"]
fn consumer_reproduction_arm_pilot() {
    let (form, original) = arm();
    let seed = std::env::var("ISOCOSM_REPRO_SEED").map_or(0, |s| s.parse().unwrap());
    let g = world(seed, form, original);
    eprintln!("ARM PILOT BEGIN form={} original={original} seed={seed} genesis={}", form.name(), crate::digest(&g));
    let rate = if original { "original" } else { "candidate" };
    let tweak = std::env::var("ISOCOSM_REPRO_TWEAK").unwrap_or_default().replace([',', '='], "_");
    let out = played_named(g, Some(format!("{}-{rate}{}-seed{seed}", form.name(),
        if tweak.is_empty() { String::new() } else { format!("-{tweak}") })));
    eprintln!("ARM PILOT completed: {out:?} qualifies={}", out.qualifies());
}

/// Exploratory selection pilots (declaration 2): seeds from
/// `ISOCOSM_REPRO_SEEDS` ("100-107"), the tweak from `ISOCOSM_REPRO_TWEAK`.
/// One line per seed; no per-draw receipts.
#[test]
#[ignore = "declared tuning battery; ISOCOSM_REPRO_ARM, _SEEDS, _TWEAK, _WORKERS"]
fn consumer_reproduction_battery() {
    let (form, original) = arm();
    let spec = std::env::var("ISOCOSM_REPRO_SEEDS").unwrap();
    let (lo, hi) = spec.split_once('-').unwrap();
    let (lo, hi): (u64, u64) = (lo.parse().unwrap(), hi.parse().unwrap());
    let workers: u64 = std::env::var("ISOCOSM_REPRO_WORKERS").map_or(1, |w| w.parse().unwrap());
    let tweak = std::env::var("ISOCOSM_REPRO_TWEAK").unwrap_or_default();
    let (send, receive) = mpsc::channel();
    std::thread::scope(|scope| {
        for worker in 0..workers {
            let send = send.clone();
            scope.spawn(move || {
                for seed in (lo + worker..=hi).step_by(workers as usize) {
                    send.send((seed, played_recorded(world(seed, form, original), None, None))).unwrap();
                }
            });
        }
        drop(send);
    });
    let mut rows: Vec<_> = receive.iter().collect();
    rows.sort_by_key(|(seed, _)| *seed);
    for (seed, out) in &rows {
        eprintln!("BATTERY tweak=[{tweak}] seed={seed} tick={} births={} born_heirs={} boundaries={} living={} q={}",
            out.tick, out.births, out.born_heirs, out.boundaries, out.living, out.qualifies());
    }
    eprintln!("BATTERY-TOTAL tweak=[{tweak}] form={} seeds={spec}: {}/{}", form.name(),
        rows.iter().filter(|(_, o)| o.qualifies()).count(), rows.len());
}
