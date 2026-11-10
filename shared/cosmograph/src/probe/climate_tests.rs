// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The crowd charges the strain of a climate world as the individuals do
//! (case 1215; lineage-rounds D13, D14).

use super::{BodyFounding, aggregate::{self, normalize}};
use crate::{
    Execution, Simulation,
    climate_tests::flat,
    meaning,
    rules::Climate,
    schema::Entity,
    simulation::Outcome,
};

/// A body world under a climate that strains it in heat, water and light.
fn strained(rates: [u32; 3]) -> super::ProbeWorld {
    let mut w = BodyFounding { seed: 3, ..Default::default() }.generate().unwrap();
    w.ticks = 6;
    let c = Climate {
        base: 80,
        strain: [rates[0], 4],
        drought: [rates[1], 4],
        dim: [rates[2], 50],
        ..flat()
    };
    w.genesis.rules.climate = Some(c);
    w.genesis.rules.conditions.insert("world:water".into());
    for site in w.genesis.sites.values_mut() {
        site.conditions.insert("world:water".into(), 0);
    }
    // The strain processes are made when a simulation is built.
    let built = Simulation::new(w.genesis.clone(), Execution::Individuals).unwrap();
    w.genesis = built.genesis().clone();
    assert!(w.genesis.rules.processes.keys().any(|k| k.starts_with("climate:strain-")));
    w
}

fn soil(accounts: &crate::schema::Ledger) -> u64 {
    meaning::value(accounts, "world:soil")
}

/// The strain process of `entity`'s lineage.
fn process_of<'a>(w: &'a super::ProbeWorld, entity: &Entity) -> &'a crate::rules::Process {
    let i = entity.lineage.trim_start_matches("lineage:");
    &w.genesis.rules.processes[&format!("climate:strain-{i}")]
}

/// A bin of members pays the strain as each of its members does one by one,
/// at the same site and tick, as the other bin-against-members controls
/// compare it: the same member afterwards, the same ground afterwards, and
/// what they pay reaches the soil. The crowd's whole run is not compared
/// here: its breeding draws differently from the individuals' in any
/// bodied world, strain or not.
#[test]
fn a_bin_pays_the_strain_its_members_pay_one_by_one() {
    let w = strained([3, 7, 2]);
    let mut paid = 0u64;
    for before in [0, 4, 9] {
        let mut sim = Simulation::new(w.genesis.clone(), Execution::Individuals).unwrap();
        sim.advance(before).unwrap();
        let tick = sim.state().tick;
        let groups: Vec<_> = sim.state().population.groups.iter()
            .filter(|(_, g)| g.entity.lineage.starts_with("lineage:") && g.entity.alive)
            .map(|(first, g)| (*first, g.entity.as_ref().clone(), g.count))
            .collect();
        for (first, entity, count) in groups {
            let p = process_of(&w, &entity);
            let mut core = sim.clone();
            let start = sim.state().sites[&entity.place].clone();
            let mut site = start.clone();
            let draws = std::collections::BTreeMap::new();
            let a = aggregate::Act {
                start: &start,
                count,
                tick,
                rules: &w.genesis.rules,
                lineages: Some(&sim.state().lineages),
                draws: &draws,
                seed: Some(w.genesis.seed),
                meal: None,
            };
            let moved = aggregate::act(p, &entity, &mut site, aggregate::Run::Free, a).unwrap();
            let moved = moved.expect("the strain is accepted").member;
            for id in first..first + count {
                let outcome = core.execute(id, None, &p.id, None).outcome;
                assert_eq!(outcome, Outcome::Accepted, "{}", p.id);
                let member = normalize(core.state().population.get(id).unwrap().clone());
                assert_eq!(member, moved, "{} at tick {tick}", p.id);
            }
            assert_eq!(core.state().sites[&entity.place].accounts, site.accounts, "{}", p.id);
            paid += soil(&site.accounts).saturating_sub(soil(&start.accounts));
        }
    }
    assert!(paid > 0, "the strain reached the soil");
}

/// A caller with no seed to give cannot read strain, and a read that fails
/// blocks the act, as it does in the core: nothing is charged.
#[test]
fn without_the_seed_the_strain_is_blocked_and_nothing_is_charged() {
    let w = strained([1, 1, 1]);
    let e = w.genesis.population.groups.values().find(|g| g.entity.lineage.starts_with("lineage:")).unwrap();
    let p = process_of(&w, &e.entity);
    let start = w.genesis.sites[&e.entity.place].clone();
    let mut site = start.clone();
    let run = aggregate::Run::Free;
    let bodies = (&w.genesis.rules, Some(&w.genesis.lineages));
    let blocked = aggregate::apply(p, &e.entity, &start, &mut site, 1, 1, bodies, run).unwrap();
    assert_eq!(blocked, None);
    assert_eq!(site.accounts, start.accounts);
}
