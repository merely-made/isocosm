// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8h: the probe on recipes (rulings 513, 514, 517 and
//! 527 to 531). Each root gives reproduce a cell and the recipes vary by
//! the odds each world draws; with no reproduce cell, no variance and no
//! absences they develop checkpoint 7's bodies, 513's control; the probe
//! sets its own life history, semelparity drawn per cohort; and its run
//! is long enough for every birth and for milk.

use isocosm::{
    Execution, Simulation, anatomy, draw,
    probe::BodyFounding,
    rules::{BodyRules, Facing},
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

fn world(f: BodyFounding) -> Genesis {
    f.generate().unwrap().genesis
}

fn members<'a>(g: &'a Genesis, lineage: &'a str) -> impl Iterator<Item = &'a Entity> + 'a {
    g.population
        .groups
        .values()
        .map(|c| &c.entity)
        .filter(move |e| e.lineage == lineage)
}

fn cells(p: &Part, f: &str) -> u32 {
    p.cells.get(f).copied().unwrap_or(0)
}

/// Checkpoint 7's body plans for `f`, as it drew them: a frond all fixing;
/// a lump split between intake and store, its limbs and an eye. Its one
/// difference: checkpoint 7 drew each limb's reach, and a recipe's pair is
/// one kind, so the reach is the first limb's.
/// Each part's box, cells and parent, in part order.
type Plan = Vec<([i32; 3], BTreeMap<String, u32>, Option<PartId>)>;

fn checkpoint_7(f: &BodyFounding) -> [Plan; 2] {
    let pick =
        |domain: &str, i: u64, r: [u64; 2]| r[0] + draw(f.seed, domain, &[i]) % (r[1] - r[0] + 1);
    let capacity = anatomy::capacity;
    let one = |f: &str, n: u32| BTreeMap::from([(format!("function:{f}"), n)]);
    let frond = [0, 1].map(|axis| pick("body-frond", axis, f.frond.map(|h| h as u64)) as i32);
    let frond = [frond[0], frond[1], 1];
    let store = (pick("body-store", 0, f.store) as u32).min(7);
    let lump = BTreeMap::from([
        ("function:intake".to_string(), 8 - store),
        ("function:store".to_string(), store),
    ]);
    let reach = pick("body-limb", 0, f.limb.map(|h| h as u64)) as i32;
    let mut grazer = vec![([2, 2, 2], lump, None)];
    for _ in 0..pick("body-limbs", 0, f.limbs) {
        let limb = [reach, 1, 1];
        grazer.push((limb, one("contract", capacity(limb)), Some(PartId(0))));
    }
    grazer.push((
        [1, 1, 1],
        one("sense", capacity([1, 1, 1])),
        Some(PartId(0)),
    ));
    [vec![(frond, one("fix", capacity(frond)), None)], grazer]
}

#[test]
fn the_control_founds_checkpoint_7s_bodies() {
    let b = BodyRules::default();
    for seed in 1..=40 {
        let f = BodyFounding {
            seed,
            reproduce: [0, 0],
            variance: [0, 0],
            absence: [0, 0],
            ..Default::default()
        };
        let g = world(f.clone());
        let plans = checkpoint_7(&f);
        for (i, plan) in plans.iter().enumerate() {
            let lineage = format!("lineage:{i}");
            let d = g.lineages[&lineage].development.as_ref().unwrap();
            assert_eq!((d.recipe.variance, d.recipe.absence), (0, [0, 1]));
            for e in members(&g, &lineage) {
                let parts: Vec<_> = e
                    .parts
                    .iter()
                    .map(|(id, p)| (e.extent(*id), p.cells.clone(), e.parent_of(*id)))
                    .collect();
                assert_eq!(&parts, plan, "seed {seed}, {lineage}");
                assert!(e.soma.iter().all(|n| *n == 1));
                // Its tissue one share of every part's adult mass, its
                // reserve only where it stores, and no provision.
                let tissue = format!("tissue:{i}");
                let share = |m: u64| {
                    e.parts.iter().all(|(id, p)| {
                        p.matter[&tissue] == anatomy::ceiling(e.extent(*id), b) * m / 1000
                    })
                };
                let [lo, hi] = f.tissue;
                assert!((lo..=hi).any(share), "seed {seed}, {lineage}");
                for p in e.parts.values() {
                    let stores = cells(p, "function:store") > 0;
                    assert_eq!(p.matter.contains_key(&format!("reserve:{i}")), stores);
                    assert!(!p.matter.keys().any(|k| k.starts_with("provision:")));
                }
            }
        }
    }
}

#[test]
fn each_root_reproduces_and_recipes_vary_by_their_drawn_odds() {
    let (mut variance, mut absence, mut clutch) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut segments = BTreeSet::new();
    let mut absent = 0;
    for seed in 0..40 {
        let g = world(BodyFounding {
            seed,
            ..Default::default()
        });
        for k in ["kind:frond", "kind:lump"] {
            assert_eq!(cells_of(&g, k, "function:reproduce"), 1, "seed {seed}");
        }
        for (i, lineage) in ["lineage:0", "lineage:1"].iter().enumerate() {
            let d = g.lineages[*lineage].development.as_ref().unwrap();
            variance.insert(d.recipe.variance);
            absence.insert(d.recipe.absence);
            if i == 1 {
                clutch.insert(d.clutch);
                let limbs = members(&g, lineage)
                    .map(|e| {
                        e.parts
                            .values()
                            .filter(|p| p.functions.contains("function:contract"))
                            .count()
                    })
                    .max()
                    .unwrap();
                let lumps = members(&g, lineage)
                    .map(|e| e.soma[0] as usize)
                    .max()
                    .unwrap();
                assert!(limbs <= lumps * if d.policy.bilateral { 2 } else { 1 });
                assert_eq!(d.recipe.tagmata[0].socket, Facing::Right);
            }
            for e in members(&g, lineage) {
                segments.extend(e.soma.iter().copied());
                let bearing = e.soma[0] as usize * usize::from(i == 1);
                let limbs = e
                    .parts
                    .values()
                    .filter(|p| p.functions.contains("function:contract"))
                    .count();
                let per = if d.policy.bilateral { 2 } else { 1 };
                absent += usize::from(limbs < bearing * per);
            }
        }
    }
    assert_eq!(variance, BTreeSet::from([1, 2]));
    assert_eq!(absence, BTreeSet::from([[1, 12]]));
    assert_eq!(clutch, BTreeSet::from([1, 2, 3, 4]));
    assert!(
        segments.contains(&2),
        "a cohort drew more segments: {segments:?}"
    );
    assert!(absent > 0, "some cohort lacks a limb");
}

fn cells_of(g: &Genesis, kind: &str, function: &str) -> u32 {
    g.rules.kinds[kind]
        .cells
        .get(function)
        .copied()
        .unwrap_or(0)
}

#[test]
fn the_probe_sets_its_own_life_history_and_draws_semelparity_by_cohort() {
    let (mut once, mut many) = (0, 0);
    for seed in 0..8 {
        let g = world(BodyFounding {
            seed,
            ..Default::default()
        });
        let traits = |l: &str| g.lineages[l].traits.clone();
        assert!(traits("lineage:0").contains("strategy:bud"));
        for t in ["strategy:brood", "strategy:egg", "care:milk"] {
            assert!(traits("lineage:1").contains(t), "{t}");
        }
        for e in g.population.groups.values().map(|c| &c.entity) {
            if e.lineage == "world:ground" {
                continue;
            }
            let (a, b) = (
                e.traits.contains("life:semelparous"),
                e.traits.contains("life:iteroparous"),
            );
            assert!(a != b, "seed {seed}: one or the other");
            once += usize::from(a);
            many += usize::from(b);
        }
    }
    assert!(once > 0 && many > 0, "{once} once, {many} many");
}

/// Every birth and milk come within the probe's run (517), in the domain.
#[test]
fn every_birth_and_milk_come_within_the_run() {
    let watched = [
        "body:bud-0",
        "body:bud-once-0",
        "body:bear-1",
        "body:bear-once-1",
        "body:nurse-1",
    ];
    let mut acted: BTreeMap<String, u64> = BTreeMap::new();
    let mut born = BTreeMap::new();
    for seed in 0..8 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut sim = Simulation::new(w.genesis.clone(), Execution::Individuals).unwrap();
        for p in watched {
            sim.watch(p);
        }
        let before = sim.matter();
        sim.advance(w.ticks).unwrap();
        assert_eq!(sim.matter(), before, "seed {seed}");
        for a in sim.take_watched() {
            *acted.entry(a.process).or_default() += a.count;
        }
        for c in sim.state().population.groups.values() {
            if c.entity.born > 0 {
                let egg = c.entity.lineage == "lineage:1" && c.entity.parts.len() == 1;
                *born.entry((c.entity.lineage.clone(), egg)).or_insert(0) += c.count;
            }
        }
    }
    for p in watched {
        assert!(acted.get(p).copied().unwrap_or(0) > 0, "{p}: {acted:?}");
    }
    for kind in [
        ("lineage:0", false),
        ("lineage:1", false),
        ("lineage:1", true),
    ] {
        assert!(
            born.contains_key(&(kind.0.to_string(), kind.1)),
            "{kind:?}: {born:?}"
        );
    }
}
