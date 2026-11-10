// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Temperature, the comfort band and strain (case 1215; lineage-rounds D13
//! to D15).

use crate::{
    Execution, Founding, Simulation,
    simulation::Genesis,
    climate::{band, comfort, strain, temperature},
    light::LATITUDE,
    rules::*,
    schema::*,
};
use std::collections::BTreeMap;

/// Every driver nought, a flat 30 degrees, a band of 10..30 at the reference.
pub(crate) fn flat() -> Climate {
    Climate {
        base: 30,
        poleward: 0,
        lapse: 0,
        rise: 1,
        season: 0,
        weather: 0,
        spell: 10,
        centre: 20,
        width: 10,
        reference: 100,
        doubling: 2,
        strain: [1, 1],
        // Water and light cost nothing here; their own tests set a rate.
        thirst: 40,
        buffer: 30,
        drought: [0, 1],
        shade: 250,
        dim: [0, 1],
        turn: None,
    }
}

pub(crate) fn founded() -> Genesis {
    let founding = Founding {
        seed: 31,
        lineages: 3,
        bodies: Some(crate::bodied::Bodies::default()),
        ..Default::default()
    };
    founding.generate().unwrap()
}

pub(crate) fn site(latitude: Option<i64>, extra: &[(&str, i64)]) -> Site {
    let mut conditions: BTreeMap<Key, i64> = extra.iter().map(|(k, v)| ((*k).into(), *v)).collect();
    if let Some(l) = latitude {
        conditions.insert(LATITUDE.into(), l);
    }
    Site { terrain_seed: 0, conditions, accounts: Ledger::new(), routes: vec![], authored: None }
}

pub(crate) fn with(climate: Climate) -> Rules {
    let mut rules = founded().rules;
    rules.climate = Some(climate);
    rules
}

fn at(rules: &Rules, latitude: i64, tick: Tick) -> i64 {
    temperature(rules, 7, 1, &site(Some(latitude), &[]), tick).unwrap()
}

fn year(rules: &Rules) -> Tick {
    year_ticks(rules.tick_microseconds())
}

#[test]
fn unset_there_is_no_temperature_and_no_comfort() {
    let g = founded();
    assert!(g.rules.climate.is_none());
    assert_eq!(temperature(&g.rules, 7, 1, &site(Some(0), &[]), 5), None);
    let e = g.population.groups.values().next().unwrap().entity.as_ref();
    assert_eq!(comfort(&g.rules, e, &g.lineages), None);
    // Nothing of it reaches the wire or the rules' identity.
    let json = serde_json::to_string(&g.rules).unwrap();
    assert!(!json.contains("climate"), "{json}");
    let mut set = g.rules.clone();
    set.climate = Some(flat());
    assert_ne!(set.revision(), g.rules.revision());
    // Generating adds no strain process; a sim of it asks for no temperature.
    assert!(g.rules.processes.keys().all(|k| !k.starts_with("climate:")));
    let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
    assert_eq!(sim.temperature(0), None);
    assert_eq!(*sim.genesis(), g, "an unset climate leaves the genesis as it was");
    sim.advance(5).unwrap();
}

#[test]
fn temperature_is_pure_and_deterministic() {
    let mut c = flat();
    c.poleward = 50;
    c.weather = 4;
    let rules = with(c);
    let first = at(&rules, 3_000, 77);
    assert_eq!(at(&rules, 3_000, 77), first);
    let g = Genesis { rules: rules.clone(), ..founded() };
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let before = (0..6).map(|t| at(&rules, 1_000, t * 11)).collect::<Vec<_>>();
    sim.advance(8).unwrap();
    let after = (0..6).map(|t| at(&rules, 1_000, t * 11)).collect::<Vec<_>>();
    assert_eq!(before, after, "advancing a sim does not touch it");
    let now = sim.state().tick;
    let (id, found) = sim.state().sites.iter().next().unwrap();
    let want = temperature(&sim.genesis().rules, sim.genesis().seed, *id, found, now);
    assert_eq!(sim.temperature(*id), want);
    assert!(want.is_some());
    // The seed matters only through the weather.
    let a = (0..200).map(|t| temperature(&rules, 1, 1, &site(None, &[]), t * 10_000)).collect::<Vec<_>>();
    let b = (0..200).map(|t| temperature(&rules, 2, 1, &site(None, &[]), t * 10_000)).collect::<Vec<_>>();
    assert_ne!(a, b);
}

#[test]
fn it_falls_with_latitude_by_the_stated_amount() {
    let mut c = flat();
    c.poleward = 50;
    let rules = with(c);
    assert_eq!(at(&rules, 0, 0), 30);
    assert_eq!(at(&rules, 4_500, 0), 30 - 25);
    assert_eq!(at(&rules, -4_500, 0), 30 - 25, "either hemisphere");
    assert_eq!(at(&rules, 9_000, 0), 30 - 50);
    assert_eq!(at(&rules, 1_800, 0), 30 - 10);
    // A site with no latitude sits on the equator.
    assert_eq!(temperature(&rules, 7, 1, &site(None, &[]), 0), Some(30));
}

#[test]
fn it_falls_with_height_above_the_sea() {
    let mut c = flat();
    c.lapse = 6;
    c.rise = 256;
    let mut rules = with(c);
    rules.skeleton = Some(Skeleton { elevation: "e".into(), relief: "r".into(), water: "w".into() });
    {
        let t = |conditions: &[(&str, i64)]| temperature(&rules, 7, 1, &site(Some(0), conditions), 0).unwrap();
        assert_eq!(t(&[("e", 0), ("w", 0)]), 30);
        assert_eq!(t(&[("e", 256), ("w", 0)]), 24);
        assert_eq!(t(&[("e", 768), ("w", 0)]), 12);
        assert_eq!(t(&[("e", 1_000), ("w", 744)]), 24, "height is above the sea, not above nought");
    }
    // Without terrain there is no height.
    rules.skeleton = None;
    let bare = temperature(&rules, 7, 1, &site(Some(0), &[("e", 768), ("w", 0)]), 0);
    assert_eq!(bare, Some(30));
}

#[test]
fn the_year_swings_at_a_pole_and_not_at_the_equator() {
    let mut c = flat();
    c.season = 15;
    let rules = with(c);
    let y = year(&rules);
    let phases = [0, y / 4, y / 2, 3 * y / 4];
    let high: Vec<i64> = phases.iter().map(|&t| at(&rules, 9_000, t)).collect();
    assert_eq!(high, [30, 45, 30, 15], "a pole swings its full season");
    let south: Vec<i64> = phases.iter().map(|&t| at(&rules, -9_000, t)).collect();
    assert_eq!(south, [30, 15, 30, 45], "opposite across the equator");
    let mid: Vec<i64> = phases.iter().map(|&t| at(&rules, 6_000, t)).collect();
    assert_eq!(mid, [30, 40, 30, 20], "in proportion to the latitude");
    assert!(phases.iter().all(|&t| at(&rules, 0, t) == 30), "none at the equator");
    assert_eq!(at(&rules, 9_000, y), at(&rules, 9_000, 0), "the year repeats");
}

#[test]
fn weather_stays_in_range_holds_for_a_spell_and_differs_across_spells() {
    let mut c = flat();
    c.weather = 4;
    c.spell = 25;
    let rules = with(c);
    let reads: Vec<i64> = (0..2_500).map(|t| at(&rules, 0, t)).collect();
    assert!(reads.iter().all(|d| (26..=34).contains(d)), "within the weather");
    for spell in reads.chunks(25) {
        assert!(spell.iter().all(|d| *d == spell[0]), "constant in a spell");
    }
    let kinds: std::collections::BTreeSet<_> = reads.iter().collect();
    assert!(kinds.len() > 3, "snaps and waves: {kinds:?}");
    // Sites have weather of their own.
    let other = (0..100).map(|s| temperature(&rules, 7, s, &site(Some(0), &[]), 0).unwrap());
    assert!(other.collect::<std::collections::BTreeSet<_>>().len() > 1);
    // No weather, no anomaly.
    let mut calm = flat();
    calm.spell = 25;
    let calm = with(calm);
    assert!((0..200).all(|t| at(&calm, 0, t) == 30));
}

#[test]
fn the_band_follows_the_mass_and_the_offset() {
    let c = flat();
    assert_eq!(band(&c, 100, 0), [10, 30], "the reference");
    assert_eq!(band(&c, 199, 0), [10, 30], "not yet a doubling");
    assert_eq!(band(&c, 200, 0), [8, 28]);
    assert_eq!(band(&c, 400, 0), [6, 26]);
    assert_eq!(band(&c, 1_600, 0), [2, 22]);
    assert_eq!(band(&c, 50, 0), [12, 32], "a halving raises it");
    assert_eq!(band(&c, 25, 0), [14, 34]);
    assert_eq!(band(&c, 51, 0), [12, 32], "less than the reference is a halving");
    assert_eq!(band(&c, 100, 3), [13, 33]);
    assert_eq!(band(&c, 400, -5), [1, 21]);
    assert_eq!(band(&c, 0, 0), band(&c, 1, 0), "no mass is the least");
}

#[test]
fn strain_is_the_distance_beyond_the_nearer_edge() {
    let b = [10, 30];
    assert_eq!([strain(b, 10), strain(b, 20), strain(b, 30)], [0, 0, 0]);
    assert_eq!(strain(b, 31), 1);
    assert_eq!(strain(b, 45), 15);
    assert_eq!(strain(b, 9), 1);
    assert_eq!(strain(b, -5), 15);
    assert_eq!(strain(b, i64::MAX), u64::try_from(i128::from(i64::MAX) - 30).unwrap());
}

/// What `e` holds of matter, in its accounts and its parts.
pub(crate) fn held(rules: &Rules, e: &Entity) -> u128 {
    rules
        .accounts
        .keys()
        .filter(|k| crate::meaning::matter(rules, k))
        .map(|k| u128::from(crate::anatomy::held(e, rules, k)))
        .sum()
}

/// The soil, summed over the sites.
pub(crate) fn soil(sim: &Simulation) -> u128 {
    sim.state().sites.values().map(|s| u128::from(crate::meaning::value(&s.accounts, "world:soil"))).sum()
}

/// A world whose first living lineage sits `beyond` degrees above its band
/// under a flat climate paying `rate`, and that lineage's first member.
pub(crate) fn warm(g: &Genesis, beyond: i64, rate: [u32; 2]) -> (Genesis, Id) {
    let mut g = g.clone();
    let (first, e) = g.population.groups.iter()
        .find(|(_, c)| c.entity.lineage.starts_with("lineage:") && c.entity.alive)
        .map(|(id, c)| (*id, c.entity.as_ref().clone()))
        .unwrap();
    let mut c = flat();
    c.strain = rate;
    g.rules.climate = Some(c.clone());
    let edge = comfort(&g.rules, &e, &g.lineages).unwrap()[1];
    c.base = edge + beyond;
    g.rules.climate = Some(c);
    (g, first)
}

/// A flat-upkeep ecology world and a body-priced one.
pub(crate) fn worlds() -> Vec<(&'static str, Genesis)> {
    let ecology = Founding { seed: 5, lineages: 3, ecology: true, ..Default::default() };
    let bodied = crate::probe::BodyFounding { seed: 3, ..Default::default() };
    vec![("flat", ecology.generate().unwrap()), ("body", bodied.generate().unwrap().genesis)]
}

#[test]
fn strain_is_paid_to_the_soil_and_matter_is_only_moved() {
    for (name, g) in worlds() {
        let (a, first) = warm(&g, 3, [1, 1]);
        let (b, _) = warm(&g, 3, [0, 1]);
        let mut paying = Simulation::new(a, Execution::Individuals).unwrap();
        let mut free = Simulation::new(b, Execution::Individuals).unwrap();
        assert!(paying.genesis().rules.processes.keys().any(|k| k.starts_with("climate:strain-")), "{name}");
        let total = paying.matter();
        assert_eq!(free.matter(), total);
        for ticks in 1..=3u128 {
            paying.advance(1).unwrap();
            free.advance(1).unwrap();
            let rules = &paying.genesis().rules;
            let (p, f) = (paying.state().population.get(first), free.state().population.get(first));
            let (p, f) = (held(rules, p.unwrap()), held(rules, f.unwrap()));
            assert_eq!(f - p, 3 * ticks, "{name}: 3 degrees at a matter each, tick {ticks}");
            assert_eq!(paying.matter(), total, "{name}: matter is conserved");
            assert_eq!(paying.matter(), free.matter(), "{name}");
        }
        let (paid, spared) = (soil(&paying), soil(&free));
        assert!(paid > spared, "{name}: the matter arrives in the soil");
        // The soil gained what the bodies lost, entity by entity.
        let rules = &paying.genesis().rules;
        let lost: u128 = free.state().population.groups.iter()
            .map(|(id, g)| {
                let p = paying.state().population.get(*id).map_or(0, |e| held(rules, e));
                held(rules, &g.entity).saturating_sub(p) * u128::from(g.count)
            })
            .sum();
        assert!(lost > 0, "{name}");
    }
}

#[test]
fn inside_the_band_nothing_extra_is_paid() {
    for (name, g) in worlds() {
        // Centred on the band, and at its very edge.
        for beyond in [-10, 0] {
            let (a, _) = warm(&g, beyond, [1, 1]);
            let (b, _) = warm(&g, beyond, [0, 1]);
            let mut paying = Simulation::new(a, Execution::Individuals).unwrap();
            let mut free = Simulation::new(b, Execution::Individuals).unwrap();
            paying.advance(4).unwrap();
            free.advance(4).unwrap();
            let rules = &paying.genesis().rules;
            let held_by_each = |s: &Simulation| {
                s.state().population.groups.iter().map(|(id, g)| (*id, held(rules, &g.entity))).collect::<Vec<_>>()
            };
            assert_eq!(held_by_each(&paying), held_by_each(&free), "{name}: {beyond}");
            assert_eq!(soil(&paying), soil(&free), "{name}");
        }
    }
}

#[test]
fn a_small_strain_is_not_lost_to_rounding() {
    let (_, g) = worlds().remove(1);
    let (a, first) = warm(&g, 1, [3, 10]);
    let (b, _) = warm(&g, 1, [0, 1]);
    let mut paying = Simulation::new(a, Execution::Individuals).unwrap();
    let mut free = Simulation::new(b, Execution::Individuals).unwrap();
    paying.advance(20).unwrap();
    free.advance(20).unwrap();
    let rules = &paying.genesis().rules;
    let [p, f] = [&paying, &free].map(|s| held(rules, s.state().population.get(first).unwrap()));
    assert_eq!(f - p, 6, "three tenths of a matter a tick, over twenty ticks");
}

#[test]
fn a_run_with_the_rule_set_replays_to_the_same_hash() {
    for (name, g) in worlds() {
        let (g, _) = warm(&g, 2, [1, 4]);
        let run = |g: &Genesis| {
            let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
            sim.advance(30).unwrap();
            sim.state_hash()
        };
        assert_eq!(run(&g), run(&g), "{name}");
        // Round-tripped through its own serialization.
        let again: Genesis = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        assert_eq!(run(&again), run(&g), "{name}");
        let mut grouped = Simulation::new(g.clone(), Execution::Grouped).unwrap();
        grouped.advance(30).unwrap();
    }
}

#[test]
fn a_climate_that_could_not_be_lived_by_is_refused() {
    let mut g = founded();
    for ruin in [
        |c: &mut Climate| c.spell = 0,
        |c: &mut Climate| c.rise = 0,
        |c: &mut Climate| c.reference = 0,
        |c: &mut Climate| c.strain[1] = 0,
        |c: &mut Climate| c.width = -1,
    ] {
        let mut c = flat();
        ruin(&mut c);
        g.rules.climate = Some(c);
        assert!(Simulation::new(g.clone(), Execution::Individuals).is_err());
    }
    g.rules.climate = Some(Climate::default());
    assert!(Simulation::new(g, Execution::Individuals).is_ok());
}
