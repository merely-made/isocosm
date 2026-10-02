// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's body family (ruling 262): the five natives as
//! definitions, checked act by act against Mesocosm's own formulas, copied
//! here from `mesocosm-core`'s `organism/ecology/rates.rs`, `ledger.rs` and
//! `flows.rs` as the reference, and run alike grouped and individually.

use isocosm::{
    Execution, Simulation,
    probe::BodyFounding,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::BTreeMap;

/// Mesocosm's integer square root and three-quarter power.
fn three_quarter(m: u64) -> u64 {
    let m = u128::from(m.max(1));
    (m * m.isqrt()).isqrt() as u64
}

fn part_ceiling(p: &Part) -> u64 {
    let voxels: u64 = p
        .half_extent
        .iter()
        .map(|h| 2 * u64::from(h.unsigned_abs()) + 1)
        .product();
    (voxels * 100 / 125).max(1)
}

/// `mass_ceiling_mg`, actuator span and secretory mass, as Mesocosm reads
/// them off a body.
fn build(e: &Entity) -> (u64, u64, u64) {
    let living = || e.parts.values().filter(|p| !p.severed);
    let ceiling = living().map(part_ceiling).sum();
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
    let gland = living()
        .map(|p| u64::from(p.cells.get("function:secrete").copied().unwrap_or(0)) * p.cell_mass)
        .sum();
    (ceiling, span, gland)
}

fn upkeep(m: u64, (ceiling, span, gland): (u64, u64, u64)) -> u64 {
    let c = ceiling.max(1);
    1 + three_quarter(m) * (c + span * 100 + gland) / (62 * c)
}

fn room(ceiling: u64, tissue: u64, reserve: u64) -> u64 {
    ceiling.saturating_sub(tissue) + ceiling.saturating_sub(reserve)
}

/// TD5's `earn_stock`: what lands as tissue and reserve and what spills.
fn land(stock: u64, (tissue, reserve): (u64, u64), ceiling: u64, hungry: bool) -> (u64, u64, u64) {
    if hungry {
        let r = stock.min(ceiling.saturating_sub(reserve));
        let t = (stock - r).min(ceiling.saturating_sub(tissue));
        (tissue + t, reserve + r, stock - r - t)
    } else {
        let t = stock.min(ceiling.saturating_sub(tissue));
        let r = (stock - t).min(ceiling.saturating_sub(reserve));
        (tissue + t, reserve + r, stock - t - r)
    }
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

fn first(g: &Genesis, lineage: &str) -> Id {
    let groups = g.population.groups.iter();
    groups
        .filter(|(_, x)| x.entity.lineage == lineage)
        .map(|(f, _)| *f)
        .next()
        .unwrap()
}

fn ledger(sim: &Simulation, id: Id) -> (u64, u64) {
    let e = sim.state().population.get(id).unwrap();
    let held = |k: &str| e.accounts.get(k).copied().unwrap_or(0);
    let i = e.lineage.trim_start_matches("lineage:");
    (held(&format!("tissue:{i}")), held(&format!("reserve:{i}")))
}

fn soil(sim: &Simulation) -> u64 {
    sim.state().sites[&0].accounts["world:soil"]
}

/// Every member of `lineage` set to `ledger`, so each act reads it.
fn set(g: &mut Genesis, lineage: &str, (tissue, reserve): (u64, u64)) {
    let i = lineage.trim_start_matches("lineage:");
    for group in g.population.groups.values_mut() {
        if group.entity.lineage == lineage {
            group.entity.accounts = BTreeMap::from([
                (format!("tissue:{i}"), tissue),
                (format!("reserve:{i}"), reserve),
            ]);
        }
    }
}

#[test]
fn rent_is_mesocosms_and_drains_the_reserve_first() {
    for seed in 0..6 {
        for lineage in ["lineage:0", "lineage:1"] {
            for held in [(30, 0), (90, 3), (150, 400), (400, 1000)] {
                let mut g = world(seed);
                set(&mut g, lineage, held);
                let i = lineage.trim_start_matches("lineage:");
                let id = first(&g, lineage);
                let owed = upkeep(held.0, build(g.population.get(id).unwrap()));
                let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
                let ground = soil(&sim);
                let r = sim.execute(id, None, &format!("body:upkeep-{i}"), None);
                assert_eq!(r.outcome, Outcome::Accepted);
                let from_reserve = held.1.min(owed);
                let from_tissue = (owed - from_reserve).min(held.0);
                let want = (held.0 - from_tissue, held.1 - from_reserve);
                assert_eq!(ledger(&sim, id), want, "seed {seed} {lineage} {held:?}");
                // Returned to the ground as soil at once.
                assert_eq!(soil(&sim), ground + from_reserve + from_tissue);
            }
        }
    }
}

#[test]
fn fixing_draws_mesocosms_income_and_lands_it_by_td5() {
    for seed in 0..6 {
        for held in [(30, 0), (60, 50), (120, 400), (150, 100), (190, 190)] {
            let mut g = world(seed);
            set(&mut g, "lineage:0", held);
            let id = first(&g, "lineage:0");
            let body = build(g.population.get(id).unwrap());
            let ceiling = body.0;
            let rate = (5 * three_quarter(held.0) / 31).max(1);
            let income = rate.min(room(ceiling, held.0, held.1));
            let hungry = held.1 < upkeep(held.0, body) * 100;
            let (tissue, reserve, spill) = land(income, held, ceiling, hungry);
            let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
            let ground = soil(&sim);
            let r = sim.execute(id, None, "body:fix-0", None);
            assert_eq!(r.outcome, Outcome::Accepted, "seed {seed} {held:?}");
            assert_eq!(ledger(&sim, id), (tissue, reserve), "seed {seed} {held:?}");
            assert_eq!(soil(&sim), ground - income + spill, "seed {seed} {held:?}");
        }
    }
}

#[test]
fn a_grazers_meal_is_mesocosms_mouthful_landed_and_dosed() {
    let (mut glanded, mut uncharged) = (0, 0);
    for seed in 0..12 {
        for (meal, prey) in [
            ((30, 0), (100, 5)),
            ((60, 400), (25, 0)),
            ((150, 50), (180, 90)),
        ] {
            let mut g = world(seed);
            set(&mut g, "lineage:1", meal);
            set(&mut g, "lineage:0", prey);
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
                let cells = frond.capacity.min(4);
                frond
                    .cells
                    .insert("function:fix".into(), frond.capacity - cells);
                frond.cells.insert("function:secrete".into(), cells);
                frond.functions.insert("function:secrete".into());
            }
            let body = build(g.population.get(id).unwrap());
            let (ceiling, span, _) = body;
            let c = ceiling.max(1);
            let rate = (3 * three_quarter(meal.0) * (c + span * 100) / (31 * c)).max(1);
            let mouthful = rate.min(room(ceiling, meal.0, meal.1));
            let taken = mouthful.min(prey.0);
            let hungry = meal.1 < upkeep(meal.0, body) * 100;
            let (tissue, reserve, spill) = land(taken, meal, ceiling, hungry);
            let (_, _, gland) = build(g.population.get(target).unwrap());
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
            let dose = charged * taken / prey.0.max(1);
            uncharged += u64::from(lean && charged == 0);
            glanded += u64::from(charged > 0);
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
            let paid = reserve.min(dose);
            let want = (tissue, reserve - paid);
            assert_eq!(ledger(&sim, id), want, "seed {seed} {meal:?} {prey:?}");
            assert_eq!(ledger(&sim, target), (prey.0 - taken, prey.1));
            let site = &sim.state().sites[&0].accounts;
            assert_eq!(site["world:soil"], ground + paid, "seed {seed}");
            assert_eq!(site.get("tissue:0").copied().unwrap_or(0), spill);
        }
    }
    // The controls: some bites were dosed, so the gland was read at all,
    // and some fell on ground too lean to charge it.
    assert!(glanded > 0 && uncharged > 0, "{glanded} {uncharged}");
}

#[test]
fn bodies_run_alike_grouped_and_individually() {
    for seed in 0..4 {
        let g = world(seed);
        let mut individuals = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        let mut grouped = Simulation::new(g, Execution::Grouped).unwrap();
        let before = individuals.matter();
        for tick in 1..=24 {
            individuals.advance(1).unwrap();
            grouped.advance(1).unwrap();
            let (a, b) = (individuals.state_hash(), grouped.state_hash());
            assert_eq!(a, b, "seed {seed}, tick {tick}");
        }
        assert_eq!(grouped.matter(), before, "seed {seed}");
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
        assert_eq!(
            a.iter().map(|m| m.1).sum::<u64>(),
            b.iter().map(|m| m.1).sum::<u64>(),
            "seed {seed}: no member is made or lost"
        );
    }
}
