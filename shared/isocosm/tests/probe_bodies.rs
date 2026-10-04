// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's body family (ruling 262): the five natives as
//! definitions, checked act by act against Mesocosm's own formulas, copied
//! here from `mesocosm-core`'s `organism/ecology/rates.rs`, `ledger.rs` and
//! `flows.rs` as the reference, and run alike grouped and individually.
//! Since checkpoint 7 the matter sits in parts (rulings 459, 463, 464 and
//! 504): the reserve only in what stores, income and the mouthful read
//! fixing's area and intake's volume at ruling 505's calibrated rates.
//! Since checkpoint 8 the bodies grow from recipes (rulings 513 and 518):
//! each root reproduces in a cell whose provision fills after the tissue,
//! and the formula tests found worlds without absences, so that no body
//! lacks a part and growth builds nothing.

use isocosm::{
    Execution, Simulation, anatomy,
    probe::BodyFounding,
    rules::{BodyRules, Measure},
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::BTreeMap;

/// Mesocosm's integer square root and three-quarter power.
fn three_quarter(m: u64) -> u64 {
    let m = u128::from(m.max(1));
    (m * m.isqrt()).isqrt() as u64
}

/// What a body reads off its living parts: its ceiling, its actuators'
/// span, its glands' and its stores' mass, fixing's area and intake's
/// volume.
struct Build {
    ceiling: u64,
    span: u64,
    gland: u64,
    store: u64,
    provision: u64,
    area: u64,
    volume: u64,
}

fn build(e: &Entity) -> Build {
    let b = BodyRules::default();
    let living = || e.parts.values().filter(|p| !p.severed);
    let cells = |p: &Part, f: &str| u64::from(p.cells.get(f).copied().unwrap_or(0));
    let span = living()
        .filter(|p| p.functions.contains("function:contract"))
        .map(|p| {
            u64::from(
                p.half_extent
                    .iter()
                    .map(|h| h.unsigned_abs())
                    .max()
                    .unwrap_or(0),
            )
        })
        .sum();
    let measured =
        |f: &str, m: Measure| -> u64 { living().map(|p| anatomy::share_of(p, f, m) as u64).sum() };
    Build {
        ceiling: living().map(|p| anatomy::ceiling(p, b)).sum(),
        span,
        gland: living()
            .map(|p| cells(p, "function:secrete") * anatomy::cell_mass(p, b))
            .sum(),
        store: living()
            .map(|p| cells(p, "function:store") * anatomy::cell_mass(p, b))
            .sum(),
        provision: living()
            .map(|p| cells(p, "function:reproduce") * anatomy::cell_mass(p, b))
            .sum(),
        area: measured("function:fix", Measure::Area),
        volume: measured("function:intake", Measure::Volume),
    }
}

fn upkeep(m: u64, b: &Build) -> u64 {
    let c = b.ceiling.max(1);
    1 + three_quarter(m) * (c + b.span * 100 + b.gland) / (62 * c)
}

/// TD5's hunger: fewer than a hundred ticks of upkeep in reserve, the
/// horizon capped by what the stores may hold (ruling 551).
fn hungry(b: &Build, (tissue, reserve): (u64, u64)) -> bool {
    reserve < (upkeep(tissue, b) * 100).min(b.store)
}

/// What the body has room for: below its ceiling in tissue, below its
/// stores' mass in reserve (ruling 463) and below its reproduce cells' mass
/// in provision (518), its provision empty.
fn room(b: &Build, tissue: u64, reserve: u64) -> u64 {
    b.ceiling.saturating_sub(tissue) + b.store.saturating_sub(reserve) + b.provision
}

/// TD5's `earn_stock`, the provision filled once the parts are (520, 535):
/// what lands as tissue, reserve and provision, and what spills.
fn land(stock: u64, (tissue, reserve): (u64, u64), b: &Build, hungry: bool) -> [u64; 4] {
    let (t_room, r_room) = (
        b.ceiling.saturating_sub(tissue),
        b.store.saturating_sub(reserve),
    );
    let mut left = stock;
    let mut take = |room: u64| {
        let n = left.min(room);
        left -= n;
        n
    };
    let (t, r, p) = if hungry {
        let r = take(r_room);
        let t = take(t_room);
        (t, r, take(b.provision))
    } else {
        let t = take(t_room);
        let p = take(b.provision);
        (t, take(r_room), p)
    };
    [tissue + t, reserve + r, p, left]
}

fn world(seed: u64) -> Genesis {
    BodyFounding {
        seed,
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis
}

/// A world whose bodies lack nothing their recipes name.
fn whole(seed: u64) -> Genesis {
    BodyFounding {
        seed,
        absence: [0, 0],
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis
}

fn provision(sim: &Simulation, id: Id) -> u64 {
    let e = sim.state().population.get(id).unwrap();
    let i = e.lineage.trim_start_matches("lineage:");
    anatomy::held(e, &sim.genesis().rules, &format!("provision:{i}"))
}

fn first(g: &Genesis, lineage: &str) -> Id {
    let groups = g.population.groups.iter();
    groups
        .filter(|(_, x)| x.entity.lineage == lineage)
        .map(|(f, _)| *f)
        .next()
        .unwrap()
}

/// A body's tissue and reserve, read over its parts.
fn totals(g: &Genesis, e: &Entity) -> (u64, u64) {
    let i = e.lineage.trim_start_matches("lineage:");
    let held = |k: String| anatomy::held(e, &g.rules, &k);
    (held(format!("tissue:{i}")), held(format!("reserve:{i}")))
}

fn ledger(sim: &Simulation, id: Id) -> (u64, u64) {
    totals(sim.genesis(), sim.state().population.get(id).unwrap())
}

fn soil(sim: &Simulation) -> u64 {
    sim.state().sites[&0].accounts["world:soil"]
}

/// Every member of `lineage` emptied and given `ledger` through its parts,
/// as far as they hold it; returns what its first member holds.
fn set(g: &mut Genesis, lineage: &str, (tissue, reserve): (u64, u64)) -> (u64, u64) {
    let i = lineage.trim_start_matches("lineage:");
    let rules = g.rules.clone();
    let mut held = None;
    for group in g.population.groups.values_mut() {
        let e = &mut group.entity;
        if e.lineage != lineage {
            continue;
        }
        for p in e.parts.values_mut() {
            p.matter.clear();
        }
        for (key, amount) in [
            (format!("tissue:{i}"), tissue),
            (format!("reserve:{i}"), reserve),
        ] {
            let fits = amount.min(anatomy::room(e, &rules, &key));
            anatomy::give(e, &rules, &key, fits).unwrap().unwrap();
        }
        held = held.or(Some(totals_with(&rules, e)));
    }
    held.unwrap()
}

fn totals_with(rules: &isocosm::rules::Rules, e: &Entity) -> (u64, u64) {
    let i = e.lineage.trim_start_matches("lineage:");
    (
        anatomy::held(e, rules, &format!("tissue:{i}")),
        anatomy::held(e, rules, &format!("reserve:{i}")),
    )
}

#[test]
fn rent_is_mesocosms_and_drains_the_reserve_first() {
    let mut drained = 0;
    for seed in 0..6 {
        for lineage in ["lineage:0", "lineage:1"] {
            for asked in [(30, 0), (90, 3), (150, 40), (400, 1000)] {
                let mut g = whole(seed);
                let held = set(&mut g, lineage, asked);
                let i = lineage.trim_start_matches("lineage:");
                let id = first(&g, lineage);
                let owed = upkeep(held.0, &build(g.population.get(id).unwrap()));
                let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
                let ground = soil(&sim);
                let r = sim.execute(id, None, &format!("body:upkeep-{i}"), None);
                assert_eq!(r.outcome, Outcome::Accepted);
                let from_reserve = held.1.min(owed);
                let from_tissue = (owed - from_reserve).min(held.0);
                drained += u64::from(from_reserve > 0);
                let want = (held.0 - from_tissue, held.1 - from_reserve);
                assert_eq!(ledger(&sim, id), want, "seed {seed} {lineage} {held:?}");
                // Returned to the ground as soil at once.
                assert_eq!(soil(&sim), ground + from_reserve + from_tissue);
            }
        }
    }
    // The control: some rent was drawn from a store's reserve.
    assert!(drained > 0);
}

/// A body starves when its tissue and reserve cannot pay its rent (523),
/// and not before: rent it can just pay keeps it alive.
#[test]
fn a_body_starves_only_when_it_cannot_pay_its_rent() {
    let (mut starved, mut kept) = (0, 0);
    for seed in 0..4 {
        for lineage in ["lineage:0", "lineage:1"] {
            let i = lineage.trim_start_matches("lineage:");
            for asked in [(0, 0), (1, 0), (40, 0), (150, 40)] {
                let mut g = whole(seed);
                let held = set(&mut g, lineage, asked);
                let id = first(&g, lineage);
                let owed = upkeep(held.0, &build(g.population.get(id).unwrap()));
                let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
                let r = sim.execute(id, None, &format!("body:starve-{i}"), None);
                let starves = held.0 + held.1 < owed;
                let what = format!("seed {seed} {lineage} {held:?} owes {owed}");
                assert_eq!(r.outcome == Outcome::Accepted, starves, "{what}");
                assert_eq!(sim.state().population.get(id).unwrap().alive, !starves);
                starved += u64::from(starves);
                kept += u64::from(!starves && held.0 + held.1 == owed);
            }
        }
    }
    // The controls: some bodies starved, and some that could just pay
    // their rent were kept.
    assert!(starved > 0 && kept > 0, "{starved} starved, {kept} kept");
}

#[test]
fn a_producer_keeps_no_reserve_and_a_grazer_keeps_it_in_its_lump() {
    let g = world(1);
    for (lineage, stores) in [("lineage:0", false), ("lineage:1", true)] {
        let e = g.population.get(first(&g, lineage)).unwrap();
        let held = totals(&g, e).1;
        let b = build(e);
        assert_eq!(b.store > 0, stores, "{lineage}");
        assert!(held <= b.store, "{lineage}: {held} above {}", b.store);
        // Every milligram of the body's own matter sits in a part.
        assert!(
            e.accounts
                .keys()
                .all(|k| !k.starts_with("tissue:") && !k.starts_with("reserve:"))
        );
    }
}

#[test]
fn fixing_draws_its_income_by_area_and_lands_it_by_td5() {
    for seed in 0..6 {
        for asked in [(30, 0), (60, 50), (120, 400), (150, 100), (190, 190)] {
            let mut g = whole(seed);
            let held = set(&mut g, "lineage:0", asked);
            assert_eq!(held.1, 0, "a frond stores nothing");
            let id = first(&g, "lineage:0");
            let b = build(g.population.get(id).unwrap());
            // Ruling 505's rate, calibrated again for 513: eleven
            // milligrams for each 144 faces fixing.
            let rate = (b.area * 11 / 144).max(1);
            let income = rate.min(room(&b, held.0, held.1));
            let hungry = hungry(&b, held);
            let [tissue, reserve, sown, spill] = land(income, held, &b, hungry);
            let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
            let ground = soil(&sim);
            let r = sim.execute(id, None, "body:fix-0", None);
            assert_eq!(r.outcome, Outcome::Accepted, "seed {seed} {held:?}");
            assert_eq!(ledger(&sim, id), (tissue, reserve), "seed {seed} {held:?}");
            assert_eq!(provision(&sim, id), sown, "seed {seed} {held:?}");
            assert_eq!(soil(&sim), ground - income + spill, "seed {seed} {held:?}");
        }
    }
}

#[test]
fn a_grazers_meal_is_its_mouthful_by_volume_landed_and_dosed() {
    let (mut glanded, mut uncharged, mut stored, mut fed, mut taken_whole) = (0, 0, 0, 0, 0);
    for seed in 0..12 {
        for (meal, prey) in [
            ((30, 0), (100, 0)),
            ((60, 40), (25, 0)),
            ((150, 5), (180, 0)),
        ] {
            let mut g = whole(seed);
            let meal = set(&mut g, "lineage:1", meal);
            let prey = set(&mut g, "lineage:0", prey);
            let (id, target) = (first(&g, "lineage:1"), first(&g, "lineage:0"));
            // Half the draws graze a frond that has grown its gland.
            let grown = seed % 2 == 0;
            if grown {
                let frond = g
                    .population
                    .lift(target)
                    .unwrap()
                    .parts
                    .get_mut(&0)
                    .unwrap();
                let fixing = frond.cells["function:fix"];
                let cells = fixing.min(4);
                frond.cells.insert("function:fix".into(), fixing - cells);
                frond.cells.insert("function:secrete".into(), cells);
                frond.functions.insert("function:secrete".into());
            }
            let b = build(g.population.get(id).unwrap());
            let c = b.ceiling.max(1);
            // Ruling 505's rate, calibrated again for 513: twelve
            // milligrams for each 269 voxels of intake, scaled by TD9's build
            // multiple.
            let rate = (12 * b.volume * (c + b.span * 100) / (269 * c)).max(1);
            let mouthful = rate.min(room(&b, meal.0, meal.1));
            let hungry = hungry(&b, meal);
            fed += u64::from(!hungry);
            let gland = build(g.population.get(target).unwrap()).gland;
            let parts = |e: &Entity| -> Vec<u64> {
                let held = |p: &Part| p.matter.get("tissue:0").copied().unwrap_or(0);
                e.parts.values().map(held).collect()
            };
            let before = parts(g.population.get(target).unwrap());
            // A quarter of the draws graze over ground too lean to charge it.
            let lean = grown && seed % 4 == 0;
            if lean {
                let site = g.sites.get_mut(&0).unwrap();
                site.accounts.insert("world:soil".into(), gland - 1);
            }
            let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
            let ground = soil(&sim);
            let charged = if gland > 0 && ground >= gland {
                gland
            } else {
                0
            };
            let r = sim.execute(id, Some(target), "body:graze-1", None);
            if mouthful == 0 {
                assert!(matches!(r.outcome, Outcome::Blocked(_)), "no room, no meal");
                continue;
            }
            assert_eq!(
                r.outcome,
                Outcome::Accepted,
                "seed {seed} {meal:?} {prey:?}"
            );
            // A fed grazer whose bite would take all of a part takes it
            // whole, where affinity lets it land (516).
            let after = parts(sim.state().population.get(target).unwrap());
            if after.len() < before.len() {
                assert!(
                    !hungry && before.iter().any(|t| *t <= mouthful),
                    "seed {seed}"
                );
                let eater = sim.state().population.get(id).unwrap();
                let landed = eater
                    .parts
                    .values()
                    .any(|p| p.matter.contains_key("tissue:0"));
                assert!(landed, "seed {seed}: the part lands on the eater");
                taken_whole += 1;
                continue;
            }
            // A bite lands on one part (459): the mouthful, or all that part
            // holds, the others untouched.
            let bitten: Vec<usize> = (0..before.len())
                .filter(|k| after[*k] != before[*k])
                .collect();
            assert_eq!(bitten.len(), 1, "seed {seed}: {before:?} to {after:?}");
            let k = bitten[0];
            let taken = mouthful.min(before[k]);
            assert_eq!(before[k] - after[k], taken, "seed {seed}");
            let [tissue, reserve, sown, spill] = land(taken, meal, &b, hungry);
            stored += u64::from(reserve > meal.1);
            let dose = charged * taken / prey.0.max(1);
            uncharged += u64::from(lean && charged == 0);
            glanded += u64::from(charged > 0);
            let paid = reserve.min(dose);
            let want = (tissue, reserve - paid);
            assert_eq!(ledger(&sim, id), want, "seed {seed} {meal:?} {prey:?}");
            assert_eq!(provision(&sim, id), sown, "seed {seed} {meal:?} {prey:?}");
            assert_eq!(ledger(&sim, target), (prey.0 - taken, prey.1));
            let site = &sim.state().sites[&0].accounts;
            assert_eq!(site["world:soil"], ground + paid, "seed {seed}");
            assert_eq!(site.get("tissue:0").copied().unwrap_or(0), spill);
        }
    }
    // The controls: some bites were dosed, so the gland was read at all;
    // some fell on ground too lean to charge it; some meals stored; and
    // some grazers ate with their stores full, fed, and some of those took
    // a part whole.
    assert!(
        glanded > 0 && uncharged > 0 && stored > 0 && fed > 0 && taken_whole > 0,
        "{glanded} {uncharged} {stored} {fed} {taken_whole}"
    );
}

#[test]
fn bodies_run_alike_grouped_and_individually() {
    for seed in 0..4 {
        let g = world(seed);
        let mut individuals = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        let mut grouped = Simulation::new(g, Execution::Grouped).unwrap();
        for p in ["body:fix-0", "body:upkeep-0"] {
            individuals.watch(p);
        }
        let before = individuals.matter();
        for tick in 1..=24 {
            individuals.advance(1).unwrap();
            grouped.advance(1).unwrap();
            let (a, b) = (individuals.state_hash(), grouped.state_hash());
            assert_eq!(a, b, "seed {seed}, tick {tick}");
        }
        assert_eq!(grouped.matter(), before, "seed {seed}");
        // The control: matter moved, producers fixing and paying rent, so
        // the world whose matter held was one that changed.
        let acted: std::collections::BTreeSet<String> = individuals
            .take_watched()
            .into_iter()
            .map(|a| a.process)
            .collect();
        assert_eq!(acted.len(), 2, "seed {seed}: {acted:?}");
    }
}

#[test]
fn every_native_acts_somewhere_in_the_domain() {
    let mut acted: BTreeMap<String, u64> = BTreeMap::new();
    for seed in 0..8 {
        let g = world(seed);
        let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        for id in g.rules.processes.keys() {
            sim.watch(id);
        }
        sim.advance(24).unwrap();
        for act in sim.take_watched() {
            // A graze is accepted only on a prey holding tissue, which its
            // parts keep, so the watch weighs them (ruling 504).
            if act.process == "body:graze-1" {
                assert!(act.target_matter > 0, "seed {seed}: {act:?}");
            }
            *acted.entry(act.process).or_default() += act.count;
        }
    }
    for p in [
        "body:upkeep-0",
        "body:upkeep-1",
        "body:fix-0",
        "body:graze-1",
        "body:gland-0",
        "body:starve-0",
        "body:mineralize",
    ] {
        assert!(acted.get(p).copied().unwrap_or(0) > 0, "{p}: {acted:?}");
    }
}

#[test]
fn the_crowd_runs_bodies_without_refusing_any() {
    use isocosm::probe::{Crowd, Variant, readings, run_exact};
    let mut births = 0;
    for seed in 0..6 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        // The crowd checks every tick that it keeps the world's matter.
        let crowd = Crowd::new(&w, 7, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let exact = run_exact(&w, 7, true).unwrap();
        let (a, b) = (
            readings::crowd_members(&crowd),
            readings::exact_members(&exact),
        );
        let (alive, states) = readings::alive_states(&a);
        assert!(
            alive > 0 && states > 0 && states as u64 <= alive,
            "seed {seed}"
        );
        // Births make members, so the runners' counts agree only in
        // distribution, the certification's to judge (checkpoint 8).
        let born = |m: &[(&isocosm::schema::Entity, u64)]| m.iter().any(|(e, _)| e.born > 0);
        births += usize::from(born(&a) && born(&b));
    }
    assert!(births > 0, "no world bore in both runners");
}

/// The averaged crowd flattens every account a lineage owns at each site,
/// tissue and reserve alike (ruling 507), so a producer, which keeps no
/// reserve, is averaged too. Since checkpoint 8 a lineage's bodies vary, so
/// the class is filled to one level as far as each body may hold: every
/// member not at its bound holds the level or one above, and none at its
/// bound could hold more than that. Kin are members like any other.
#[test]
fn averaging_flattens_every_own_account_at_each_site() {
    use isocosm::probe::{Crowd, Variant};
    let mut spread = 0;
    for seed in 0..4 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let crowd = Crowd::new(&w, 5, Variant::Averaged).unwrap().run().unwrap();
        let rules = &w.genesis.rules;
        let mut classes: BTreeMap<(String, Id), Vec<(u64, u64)>> = BTreeMap::new();
        let members = crowd.bins.keys().chain(crowd.kin.values());
        for e in members.filter(|e| e.alive) {
            let i = e.lineage.trim_start_matches("lineage:");
            for key in [format!("tissue:{i}"), format!("reserve:{i}")] {
                let held = anatomy::held(e, rules, &key);
                let bound = held + anatomy::room(e, rules, &key);
                classes
                    .entry((key, e.place))
                    .or_default()
                    .push((held, bound));
            }
        }
        for ((key, site), held) in classes {
            let open: Vec<u64> = held.iter().filter(|(h, b)| h < b).map(|h| h.0).collect();
            if let Some(&level) = open.iter().min() {
                let flat = open.iter().all(|h| *h <= level + 1);
                let full = held.iter().all(|(h, b)| h < b || *b <= level + 1);
                assert!(flat && full, "seed {seed}, {key} at {site}: {held:?}");
            }
            spread += u64::from(key.starts_with("tissue:0") && held.len() > 1);
        }
    }
    // The control: some site held producers in more than one state.
    assert!(spread > 0, "no class to flatten");
}
