// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Water and light: store cells buffer drought, fixing cells set the light
//! a body needs, and the three strains add (case 1215; lineage-rounds D13,
//! D14).

use crate::{
    Execution, Simulation,
    climate::{light, needs, owed, thirst},
    climate_tests::{flat, held, site, soil, worlds},
    light::{LATITUDE, SunRules},
    rules::*,
    schema::*,
    simulation::Genesis,
};

/// The body world of `worlds()`: lineage 0 fixes light (16 of 18 cells),
/// lineage 1 stores (8 of 22).
fn bodied() -> Genesis {
    worlds().remove(1).1
}

/// The first group of `lineage`, and a member of it.
fn pick(g: &Genesis, lineage: &str) -> (Id, Entity) {
    g.population
        .groups
        .iter()
        .find(|(_, c)| c.entity.lineage == lineage && c.entity.alive)
        .map(|(id, c)| (*id, c.entity.as_ref().clone()))
        .unwrap()
}

/// Rates for the heat, water and light strains, in matter a point a tick.
fn rates(g: &mut Genesis, [heat, water, dim]: [[u32; 2]; 3]) {
    let c = Climate { strain: heat, drought: water, dim, ..flat() };
    g.rules.climate = Some(c);
}

/// Puts `first`'s site `beyond` degrees above its band's edge, and its
/// water `dry` points below its thirst (above it where negative).
fn place(g: &mut Genesis, first: Id, beyond: i64, dry: i64) {
    let e = g.population.groups[&first].entity.as_ref().clone();
    let edge = crate::climate::comfort(&g.rules, &e, &g.lineages).unwrap()[1];
    let c = g.rules.climate.as_mut().unwrap();
    c.base = edge + beyond;
    let [want, _] = needs(&g.rules, &e, &g.lineages).unwrap();
    g.rules.conditions.insert("world:water".into());
    let site = g.sites.get_mut(&e.place).unwrap();
    site.conditions.insert("world:water".into(), want - dry);
}

/// Matter `first` has lost to `free` after each of `ticks` ticks, and the
/// worlds' matter checked each time.
fn lost(paying: Genesis, free: Genesis, first: Id, ticks: u128) -> Vec<u128> {
    let mut a = Simulation::new(paying, Execution::Individuals).unwrap();
    let mut b = Simulation::new(free, Execution::Individuals).unwrap();
    let total = a.matter();
    assert_eq!(b.matter(), total);
    let mut out = vec![];
    for _ in 0..ticks {
        a.advance(1).unwrap();
        b.advance(1).unwrap();
        let rules = &a.genesis().rules;
        let [p, f] = [&a, &b].map(|s| held(rules, s.state().population.get(first).unwrap()));
        out.push(f - p);
        assert_eq!(a.matter(), total, "matter is conserved");
        assert_eq!(b.matter(), total);
    }
    assert!(soil(&a) >= soil(&b));
    out
}

#[test]
fn unset_there_are_no_needs_and_no_strain() {
    let g = bodied();
    assert!(g.rules.climate.is_none());
    let (_, e) = pick(&g, "lineage:1");
    assert_eq!(needs(&g.rules, &e, &g.lineages), None);
    let s = site(Some(0), &[("world:water", 0)]);
    assert_eq!(owed(&g.rules, 7, &g.lineages, &e, (e.place, &s), 5), 0);
}

#[test]
fn store_cells_buffer_the_thirst() {
    let c = Climate { thirst: 40, buffer: 30, ..flat() };
    assert_eq!(thirst(&c, 0, 12), 40, "no store cells");
    assert_eq!(thirst(&c, 12, 12), 10, "all store cells");
    assert_eq!(thirst(&c, 6, 12), 25, "half is the midpoint");
    assert_eq!(thirst(&c, 0, 0), 40, "no cells at all");
    assert_eq!(thirst(&Climate { buffer: 100, ..c.clone() }, 12, 12), 0, "never below nought");
    assert_eq!(thirst(&Climate { buffer: 100, ..c }, 9, 12), 0);
}

#[test]
fn fixing_cells_set_the_light_needed() {
    let c = Climate { shade: 250, ..flat() };
    assert_eq!(light(&c, 0, 12), 0, "no fixing cells need none");
    assert_eq!(light(&c, 12, 12), 250, "all fixing need the shade");
    assert_eq!(light(&c, 6, 12), 125, "half need half");
    assert_eq!(light(&c, 0, 0), 0, "no cells at all");
}

#[test]
fn a_body_needs_what_its_cells_say() {
    let mut g = bodied();
    rates(&mut g, [[0, 1]; 3]);
    let [sedge, plant] = ["lineage:1", "lineage:0"].map(|l| pick(&g, l).1);
    let c = g.rules.climate.clone().unwrap();
    assert_eq!(needs(&g.rules, &sedge, &g.lineages), Some([thirst(&c, 8, 22), 0]));
    assert_eq!(needs(&g.rules, &plant, &g.lineages), Some([c.thirst, light(&c, 16, 18)]));
    assert!(thirst(&c, 8, 22) < c.thirst && light(&c, 16, 18) > 0);
}

#[test]
fn drought_is_paid_below_the_thirst_and_not_at_or_above_it() {
    let g = bodied();
    let (first, _) = pick(&g, "lineage:1");
    let world = |rate: [u32; 2], dry: i64| {
        let mut g = g.clone();
        rates(&mut g, [[0, 1], rate, [0, 1]]);
        place(&mut g, first, -10, dry);
        g
    };
    for dry in [5, 12] {
        let lost = lost(world([1, 1], dry), world([0, 1], dry), first, 3);
        assert_eq!(lost, [dry, 2 * dry, 3 * dry].map(|n| n as u128), "{dry} points dry, a matter each");
    }
    for dry in [0, -7] {
        let lost = lost(world([1, 1], dry), world([0, 1], dry), first, 3);
        assert_eq!(lost, [0, 0, 0], "at or above the thirst nothing is paid");
    }
}

#[test]
fn dimness_is_paid_below_the_light_needed_and_not_in_full_light() {
    let g = bodied();
    let (first, e) = pick(&g, "lineage:0");
    let mut plain = g.clone();
    rates(&mut plain, [[0, 1]; 3]);
    assert_eq!(needs(&plain.rules, &e, &plain.lineages).unwrap()[1], 222, "250 for sixteen of eighteen");
    let world = |rate: [u32; 2], sun: Option<SunRules>| {
        let mut g = g.clone();
        rates(&mut g, [[0, 1], [0, 1], rate]);
        // 234 for sixteen cells in eighteen is 208: 8 above two fifths of 500.
        g.rules.climate.as_mut().unwrap().shade = 234;
        place(&mut g, first, -10, -50);
        g.rules.sun = sun;
        g
    };
    let at = |percent: u32| SunRules { tidal_lock: true, intensity_percent: percent, ..SunRules::default() };
    // Midnight at the start: the sun is down. A twentieth of a matter a unit
    // short pays 208/20 a tick, its fraction kept: the strain runs on ticks
    // 1 to 3, owing 10, 11, 10 (the whole due by tick 4, 41, less that by 1).
    let dark = lost(world([1, 20], None), world([0, 1], None), first, 3);
    assert_eq!(dark, [10, 21, 31]);
    // A sun at two fifths gives 200 of the 500: 8 short a tick.
    let dim = lost(world([1, 1], Some(at(40))), world([0, 1], Some(at(40))), first, 3);
    assert_eq!(dim, [8, 16, 24]);
    // At noon and in full light nothing is owed.
    let noon = Some(SunRules { start_hour: 12, ..SunRules::default() });
    for sun in [noon, Some(at(100)), Some(at(50))] {
        let bright = lost(world([1, 1], sun.clone()), world([0, 1], sun), first, 3);
        assert_eq!(bright, [0, 0, 0]);
    }
}

#[test]
fn the_three_strains_add() {
    // In a simulation: a body strained by heat and by drought pays what
    // each costs alone.
    let g = bodied();
    let (first, _) = pick(&g, "lineage:1");
    let world = |heat: [u32; 2], water: [u32; 2]| {
        let mut g = g.clone();
        rates(&mut g, [heat, water, [0, 1]]);
        place(&mut g, first, 3, 4);
        g
    };
    let off = world([0, 1], [0, 1]);
    let [only_heat, only_water, both] = [([1, 1], [0, 1]), ([0, 1], [1, 1]), ([1, 1], [1, 1])]
        .map(|(h, w)| lost(world(h, w), off.clone(), first, 3));
    assert_eq!(only_heat, [3, 6, 9]);
    assert_eq!(only_water, [4, 8, 12]);
    assert_eq!(both, [7, 14, 21]);
    // In the formula, with fractions: each part is its own closed form, so
    // the total is the sum of the parts exactly at every tick, for a body
    // that fixes, stores and is too hot.
    let (_, mut e) = pick(&g, "lineage:0");
    let part = e.parts.values_mut().next().unwrap();
    std::sync::Arc::make_mut(part).cells.insert("function:store".into(), 5);
    let rules = |heat: [u32; 2], water: [u32; 2], dim: [u32; 2]| {
        let mut r = g.rules.clone();
        r.climate = Some(Climate { base: 80, strain: heat, drought: water, dim, ..flat() });
        r
    };
    let dry = site(Some(2_000), &[("world:water", 3)]);
    let (h, w, d) = ([3, 10], [7, 20], [1, 6]);
    let off = [0, 1];
    let mut each = [0u64; 3];
    for tick in (0..4_000).step_by(37) {
        let at = |r: &Rules| owed(r, 9, &g.lineages, &e, (e.place, &dry), tick);
        let parts = [at(&rules(h, off, off)), at(&rules(off, w, off)), at(&rules(off, off, d))];
        assert_eq!(at(&rules(h, w, d)), parts.iter().sum::<u64>(), "tick {tick}");
        each.iter_mut().zip(parts).for_each(|(sum, p)| *sum += p);
    }
    assert!(each.iter().all(|n| *n > 0), "every strain was met: {each:?}");
}

#[test]
fn a_run_with_all_three_replays_to_the_same_hash() {
    let mut g = bodied();
    rates(&mut g, [[1, 4], [1, 4], [1, 50]]);
    let first = pick(&g, "lineage:1").0;
    place(&mut g, first, 4, 4);
    g.rules.conditions.extend(["world:water".into(), LATITUDE.into()]);
    for site in g.sites.values_mut() {
        site.conditions.insert("world:water".into(), 10);
        site.conditions.insert(LATITUDE.into(), 2_000);
    }
    let run = |g: &Genesis, mode| {
        let mut sim = Simulation::new(g.clone(), mode).unwrap();
        sim.advance(30).unwrap();
        sim.state_hash()
    };
    assert_eq!(run(&g, Execution::Individuals), run(&g, Execution::Individuals));
    let again: Genesis = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
    assert_eq!(run(&again, Execution::Individuals), run(&g, Execution::Individuals));
    // It is not a run in which nothing happened.
    let mut calm = g.clone();
    rates(&mut calm, [[0, 1]; 3]);
    assert_ne!(run(&calm, Execution::Individuals), run(&g, Execution::Individuals));
    assert_eq!(run(&g, Execution::Grouped), run(&g, Execution::Individuals));
}

#[test]
fn a_climate_with_no_denominators_or_a_thirst_off_the_scale_is_refused() {
    let mut g = bodied();
    for ruin in [
        |c: &mut Climate| c.drought[1] = 0,
        |c: &mut Climate| c.dim[1] = 0,
        |c: &mut Climate| c.thirst = 101,
        |c: &mut Climate| c.thirst = -1,
        |c: &mut Climate| c.buffer = -1,
        |c: &mut Climate| c.shade = -1,
    ] {
        let mut c = flat();
        ruin(&mut c);
        g.rules.climate = Some(c);
        assert!(Simulation::new(g.clone(), Execution::Individuals).is_err());
    }
    g.rules.climate = Some(Climate::default());
    assert!(Simulation::new(g, Execution::Individuals).is_ok());
}
