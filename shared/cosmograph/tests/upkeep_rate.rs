// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Upkeep and growth are rates, not choices (execution plan D16, case 1076):
//! in the A1 played world (seed 9, Mesocosm's founding, lineage 1 played and
//! paced at its decision period), every body alive for a full upkeep period
//! has paid, and a producer grows at its own period, whatever its lineage's
//! decision period and whatever `(born + id)` is. Before the fix a body paid
//! only if `(born + id)` was a multiple of `gcd(period, decides)`, so most
//! never did and producers grew at about a tenth of their designed rate.

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::Rates,
    flows::MadeBy,
};
use std::collections::BTreeMap;

/// The run's length: 40 ticks (four grazer periods, a debug build's budget);
/// `UPKEEP_TICKS=100` runs the 100 ticks the control was measured over.
fn ticks() -> u64 {
    std::env::var("UPKEEP_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(40)
}
/// The ticks before the plants crowd the light and soil (measured: every due
/// plant grows through tick 28), where growth is the designed rate.
const OPEN: u64 = 25;

/// Mesocosm's A1 founding (as `played_world_decay` states it).
fn world(seed: u64) -> cosmograph::simulation::Genesis {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    Founding {
        seed,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies {
            hazard: cosmograph::harm::hazard::RATE,
            wound: cosmograph::harm::hazard::CELLS,
            ..Bodies::default()
        }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

fn index(lineage: &str) -> Option<u32> {
    lineage.strip_prefix("lineage:")?.parse().ok()
}

/// Members due and members that acted, over the ticks counted.
#[derive(Default, Clone, Copy)]
struct Tally {
    due: u64,
    acted: u64,
    /// Ticks on which fewer members acted than were due.
    short: u64,
}

impl Tally {
    fn note(&mut self, due: u64, acted: u64) {
        self.due += due;
        self.acted += acted;
        self.short += u64::from(acted < due);
    }
}

#[test]
fn every_living_body_pays_upkeep_and_every_producer_grows_at_its_period() {
    let genesis = world(9);
    let period = |id: String| genesis.rules.processes[&id].period.unwrap();
    let upkeep: Vec<u64> = (0..4).map(|i| period(format!("ecology:upkeep-{i}"))).collect();
    let produce: BTreeMap<u32, u64> =
        [0, 3].into_iter().map(|i| (i, period(format!("ecology:produce-{i}")))).collect();
    let decides: Vec<_> = (0..4).map(|i| genesis.lineages[&format!("lineage:{i}")].decides).collect();
    assert!(decides.iter().any(|d| d.is_some_and(|d| d > 1)), "the lineages are paced: {decides:?}");
    let mut s = Session::new(genesis, Execution::Individuals).unwrap();
    let mut paid = [Tally::default(); 4];
    let mut grew = Tally::default();
    let (mut member_ticks, mut open_ticks, mut open_soil) = (0u64, 0u64, 0u64);
    for _ in 0..ticks() {
        let now = s.sim.state().tick + 1;
        let before = member_ticks;
        let r = s.advance_tick_with_flows().unwrap();
        // Members whose own clock fell due now and who are alive at the
        // tick's end: a body that starved or was eaten first pays nothing.
        let mut due_upkeep = [0u64; 4];
        let mut due_grow = 0u64;
        for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
            let Some(i) = index(&g.entity.lineage) else { continue };
            let born = g.entity.born;
            if now > born && (now - born) % upkeep[i as usize] == 0 {
                due_upkeep[i as usize] += g.count;
            }
            if let Some(p) = produce.get(&i) {
                member_ticks += g.count;
                if now > born && (now - born) % p == 0 {
                    due_grow += g.count;
                }
            }
        }
        let (mut did, mut acts) = ([0u64; 4], 0u64);
        for f in &r.flows {
            let MadeBy::Process(p) = &f.made_by else { continue };
            // The body's matter paid into its soil: one flow an act.
            if let Some(i) = p.strip_prefix("ecology:upkeep-").and_then(|i| i.parse::<usize>().ok())
                && f.from.1.starts_with("matter:") && f.to.1 == "world:soil"
            {
                did[i] += f.count;
            }
            if p.starts_with("ecology:produce-") && f.from.0.body().is_none() && f.from.1 == "world:soil" {
                acts += f.count;
                if now <= OPEN {
                    open_soil += f.amount * f.count;
                }
            }
        }
        for i in 0..4 {
            paid[i].note(due_upkeep[i], did[i]);
        }
        if now <= OPEN {
            grew.note(due_grow, acts);
            open_ticks += member_ticks - before;
        }
    }
    for (i, t) in paid.iter().enumerate() {
        eprintln!("L{i} ({}): {} due, {} paid, {} short ticks", ["producer", "grazer", "decomposer", "producer"][i], t.due, t.acted, t.short);
        assert!(t.due > 0, "lineage {i} has bodies due upkeep");
        // Every body alive at the end of a tick its own clock fell due on paid.
        assert_eq!(t.short, 0, "lineage {i}: a tick with a due body unpaid");
    }
    // The designed rate: a producer takes two soil into tissue every second
    // tick, so one soil per member-tick where light and soil allow.
    let per = open_soil as f64 / open_ticks as f64;
    eprintln!("growth in the first {OPEN} ticks: {} due, {} acted, {per:.3} soil per producer member-tick", grew.due, grew.acted);
    assert!(grew.due > 0 && grew.acted * 100 >= grew.due * 98, "every due producer grows: {} of {}", grew.acted, grew.due);
    assert!(grew.acted <= grew.due + grew.due / 50, "and no faster than its period");
    // Two soil an act: the designed matter made by the due members.
    assert!(open_soil * 100 >= grew.due * 2 * 98, "matter made {open_soil} of the designed {}", grew.due * 2);
}
