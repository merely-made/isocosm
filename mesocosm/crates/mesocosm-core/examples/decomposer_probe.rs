// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement for the decomposer-reach assessment (2026-09-16), not a receipt.
//!
//! Builds the generation door's played world the way `matter_ledger_probe`
//! does, then steps one epoch tick by tick and follows every founding
//! decomposer: its body, each move and meal, the carrion around it, its
//! reserve, and how it died.
//!
//! Two modes. `deep` reproduces `World::run_deep_time` (`deep_time.rs:112`)
//! from public API: the hand is released by editing `controlled` out of the
//! postcard snapshot (the release door is crate-private), every living body is set
//! far through the public `tier` field, and each tick is idle, recorded,
//! drained and reckoned. A clone runs the real `run_deep_time` for one epoch
//! and the two must agree on `state_hash` and `History`. `hand` keeps the
//! played critter under an idling hand and releases nothing.
//!
//! A third mode, `near`, releases the hand but leaves tiers where genesis put
//! them (all near at tick 0): deep time as it ran before ruling 19.
//!
//! Usage: decomposer_probe [seeds..] [organisms=N] [deep|near|hand]

use std::collections::{BTreeMap, BTreeSet};

use mesocosm_core::flow::Process as Flow;
use mesocosm_core::places::Tier;
use mesocosm_core::rules::{DeepTimeSpan, WorldRules};
use mesocosm_core::world::generation::Request;
use mesocosm_core::{
    BodyProcesses, Event, Founding, History, Intent, Kingdom, MealKind, OrganismId, Process, Role,
    Stage, World, classify, restore, snapshot, state_hash,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    if seeds.is_empty() {
        seeds = vec![7, 1, 42];
    }
    let organisms = args
        .iter()
        .find_map(|a| a.strip_prefix("organisms=")?.parse().ok())
        .unwrap_or(24u32);
    let only = |mode: &str| {
        !args
            .iter()
            .any(|a| ["deep", "near", "hand"].contains(&a.as_str()))
            || args.iter().any(|a| a == mode)
    };
    for seed in seeds {
        if only("deep") {
            run(seed, organisms, "deep");
        }
        if only("near") {
            run(seed, organisms, "near");
        }
        if only("hand") {
            run(seed, organisms, "hand");
        }
    }
}

fn chebyshev(a: [i32; 3], b: [i32; 3]) -> i32 {
    (0..3).map(|i| (a[i] - b[i]).abs()).max().unwrap_or(0)
}

#[derive(Default)]
struct Track {
    id: u32,
    start: [i32; 3],
    last: [i32; 3],
    moves: u32,
    travelled: u64,
    largest_move: u64,
    budget: u64,
    last_move: Option<(u64, u64)>,
    closing: u32,
    visited: BTreeSet<[i32; 3]>,
    places: BTreeSet<u16>,
    first_moves: Vec<String>,
    meals: Vec<(u64, u64, usize, Option<i32>)>,
    reserve: Vec<(u64, u64, u64)>,
    death: Option<String>,
    // nearest carrion per lived tick: <=6, <=12, <=24, <=48, >48, none
    hist: [u32; 6],
    hungry: u32,
    near_ticks: u32,
    far_ticks: u32,
    closest: Option<i32>,
    pre: Option<([i32; 3], Option<i32>, usize, u64)>,
    target: Option<[i32; 3]>,
}

/// `rates::dispersal_for` with the hunger bonus taken, the most a body may
/// move in one tick; zero for a body with nothing that contracts.
fn hungry_budget(actuator_span: u32) -> u64 {
    match actuator_span {
        0 => 0,
        span => u64::from((span / 4).max(1)) + 1,
    }
}

fn nearest(at: [i32; 3], carrion: &[(u32, [i32; 3], u64)]) -> Option<i32> {
    carrion.iter().map(|(_, p, _)| chebyshev(at, *p)).min()
}

fn kingdom_name(k: Kingdom) -> &'static str {
    match k {
        Kingdom::Producer => "P",
        Kingdom::Consumer => "C",
        Kingdom::Decomposer => "D",
    }
}

/// `deep`: deep time as ruled (released, every body far). `near`: released
/// with tiers left where genesis put them, which on a door world at tick 0 is
/// every body near; this is deep time before ruling 19, and it has no control.
/// `hand`: the played critter idles under a hand and nothing is released.
fn run(seed: u64, organisms: u32, mode: &str) {
    let deep = mode == "deep";
    let request = Request {
        seed,
        organisms,
        ..Request::default()
    };
    let entered = request
        .prepare(Founding::Drawn.palette())
        .expect("a prepared habitat")
        .enter(0)
        .expect("the first admitted candidate");
    let rules = entered.rules();
    let entered = entered.with_rules(WorldRules {
        deep_time: DeepTimeSpan { epochs: 1 },
        ..rules
    });

    let mut world = match mode {
        "deep" => {
            let mut released = released(&entered);
            for o in released.organisms.iter_mut().filter(|o| o.is_alive()) {
                o.tier = Tier::Far;
            }
            released
        },
        "near" => {
            let released = released(&entered);
            assert!(
                released.living().all(|o| o.tier == Tier::Near),
                "a door world at tick 0 is all near"
            );
            released
        },
        _ => entered.clone(),
    };
    let played = entered.organisms[0].kingdom();
    println!(
        "\n== seed {seed} mode {mode} organisms {organisms}+1, entered body reads {}",
        kingdom_name(played)
    );
    println!(
        "played body at {:?} place {:?}, {} mg",
        entered.organisms[0].position,
        entered
            .places()
            .at(entered.organisms[0].position)
            .map(|p| p.0),
        entered.organisms[0].biomass_mg()
    );

    let mut tracks: Vec<Track> = world
        .organisms
        .iter()
        .filter(|o| o.is_alive() && o.kingdom() == Kingdom::Decomposer)
        .map(|o| {
            let body = o.body();
            let mut roles: BTreeMap<&str, u32> = BTreeMap::new();
            let (mut contract, mut sense) = (0, 0);
            for part in body.living() {
                let name = match classify(part.half_extent) {
                    Role::Mass => "mass",
                    Role::Limb => "limb",
                    Role::Plate => "plate",
                    Role::Sensor => "sensor",
                };
                *roles.entry(name).or_default() += 1;
                contract += u32::from(body.processes(part.id).contains(&Process::Contract));
                sense += u32::from(body.processes(part.id).contains(&Process::Sense));
            }
            let ceiling = o.mass_ceiling_mg();
            let sight = 8 * (ceiling + u64::from(o.sensor_span()) * 100) / ceiling.max(1);
            println!(
                "decomposer #{} species {} at {:?} place {:?} tier {:?}: parts {} {:?}, \
                 contract parts {contract}, sense parts {sense}, actuator_span {}, sensor_span {}, \
                 reach {}, biomass {} mg, ceiling {} mg, reserve {} mg, upkeep {} mg/tick, \
                 age {}, stage {:?}, feeding {:?}, near sight {} (rates.rs:287), shape radius {}",
                o.id.0,
                o.species.0,
                o.position,
                world.places().at(o.position).map(|p| p.0),
                o.tier,
                body.living().count(),
                roles,
                o.actuator_span(),
                o.sensor_span(),
                body.reach(),
                o.biomass_mg(),
                ceiling,
                o.energy_mg,
                o.upkeep_mg(),
                o.age,
                o.stage,
                o.feeding_mode(),
                sight,
                o.walker_shape().radius(),
            );
            Track {
                id: o.id.0,
                start: o.position,
                last: o.position,
                budget: hungry_budget(o.actuator_span()),
                ..Track::default()
            }
        })
        .collect();

    let mut control = entered.clone();
    let mut control_history = History::new();
    if deep {
        control
            .run_deep_time(&mut control_history)
            .expect("one epoch of deep time");
    }

    let mut history = History::new();
    let mut seen = world.epoch;
    let start_epoch = world.epoch;
    let (mut decayed, mut scavenged, mut died_by) = (0u64, 0u64, BTreeMap::new());
    let mut births_by: BTreeMap<&str, u32> = BTreeMap::new();
    let mut corpses: BTreeSet<u32> = BTreeSet::new();
    let mut corpse_mg_made = 0u64;
    let mut max_standing = 0usize;
    let mut made_by_century: BTreeMap<u64, BTreeMap<&str, (u32, u64)>> = BTreeMap::new();
    let mut travel_by: BTreeMap<&str, (u32, u64)> = BTreeMap::new();
    // Moves beyond the hungry dispersal budget, and the largest single move.
    let mut over_budget_by: BTreeMap<&str, (u32, u64)> = BTreeMap::new();
    let mut spent_alive_by: BTreeMap<&str, u32> = BTreeMap::new();
    let carrion_now = |world: &World| -> Vec<(u32, [i32; 3], u64)> {
        world
            .organisms
            .iter()
            .filter(|o| o.stage == Stage::Carrion)
            .map(|o| (o.id.0, o.position, o.biomass_mg()))
            .collect()
    };
    let start_carrion = carrion_now(&world);
    corpses.extend(start_carrion.iter().map(|c| c.0));
    println!(
        "tick 0 carrion {} bodies {} mg at {:?}",
        start_carrion.len(),
        start_carrion.iter().map(|c| c.2).sum::<u64>(),
        start_carrion
            .iter()
            .map(|c| (c.0, c.1, c.2))
            .collect::<Vec<_>>()
    );

    let clock = std::time::Instant::now();
    while world.epoch == start_epoch {
        let tick = world.tick;
        let kingdoms: BTreeMap<u32, &str> = world
            .organisms
            .iter()
            .filter(|o| o.is_alive())
            .map(|o| (o.id.0, kingdom_name(o.kingdom())))
            .collect();
        // Read before the tick: a body its travel spends to nothing leaves the roster.
        let spans: BTreeMap<u32, u32> = world
            .organisms
            .iter()
            .filter(|o| o.is_alive())
            .map(|o| (o.id.0, o.actuator_span()))
            .collect();
        let carrion = carrion_now(&world);
        max_standing = max_standing.max(carrion.len());
        corpses.extend(carrion.iter().map(|c| c.0));
        if tick % 100 == 0 {
            context(&world, tick, &carrion);
        }
        for track in tracks.iter_mut().filter(|t| t.death.is_none()) {
            let Some(o) = world.organisms.iter().find(|o| o.id.0 == track.id) else {
                continue;
            };
            let d = nearest(o.position, &carrion);
            let bucket = match d {
                None => 5,
                Some(d) if d <= 6 => 0,
                Some(d) if d <= 12 => 1,
                Some(d) if d <= 24 => 2,
                Some(d) if d <= 48 => 3,
                Some(_) => 4,
            };
            track.hist[bucket] += 1;
            track.closest = match (track.closest, d) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            track.hungry += u32::from(o.budget_below(8));
            if o.tier == Tier::Near {
                track.near_ticks += 1;
            } else {
                track.far_ticks += 1;
            }
            if tick % 100 == 0 {
                track.reserve.push((tick, o.energy_mg, o.biomass_mg()));
            }
            if let Some(p) = world.places().at(o.position) {
                track.places.insert(p.0);
            }
            track.pre = Some((o.position, d, carrion.len(), o.biomass_mg()));
            track.target = carrion
                .iter()
                .min_by_key(|(_, p, _)| chebyshev(o.position, *p))
                .map(|c| c.1);
        }

        world.apply(Intent::Idle);
        let events = world.drain_events();
        for envelope in &events {
            match envelope.record {
                Event::Fed {
                    eater,
                    mass_mg,
                    kind: MealKind::Scavenging,
                    ..
                } => {
                    scavenged += mass_mg;
                    if let Some(t) = tracks.iter_mut().find(|t| t.id == eater.0) {
                        let (_, d, n, _) = t.pre.expect("a meal follows a lived tick");
                        t.meals.push((envelope.tick, mass_mg, n, d));
                    }
                },
                Event::Moved { organism, from, to } => {
                    let voxels = chebyshev(from, to) as u64;
                    if let Some(k) = kingdoms.get(&organism.0) {
                        let entry = travel_by.entry(k).or_default();
                        entry.0 += 1;
                        entry.1 += voxels;
                        let span = spans.get(&organism.0).copied().unwrap_or(0);
                        let over = over_budget_by.entry(k).or_default();
                        over.0 += u32::from(voxels > hungry_budget(span));
                        over.1 = over.1.max(voxels);
                    }
                    if let Some(t) = tracks.iter_mut().find(|t| t.id == organism.0) {
                        t.moves += 1;
                        t.travelled += voxels;
                        t.largest_move = t.largest_move.max(voxels);
                        t.last_move = Some((envelope.tick, voxels));
                        t.visited.insert(to);
                        t.last = to;
                        let before = nearest(from, &carrion);
                        let after = nearest(to, &carrion);
                        if let (Some(b), Some(a)) = (before, after) {
                            t.closing += u32::from(a < b);
                        }
                        if t.first_moves.len() < 8 {
                            let after_mg = world
                                .organisms
                                .iter()
                                .find(|o| o.id == organism)
                                .map(|o| o.biomass_mg());
                            t.first_moves.push(format!(
                                "t{} {:?}(pl {:?})->{:?}(pl {:?}) nearest {:?}->{:?}, target carrion in pl {:?}, body {:?}->{:?} mg",
                                envelope.tick,
                                from,
                                world.places().at(from).map(|p| p.0),
                                to,
                                world.places().at(to).map(|p| p.0),
                                before,
                                after,
                                t.target.and_then(|c| world.places().at(c)).map(|p| p.0),
                                t.pre.map(|p| p.3),
                                after_mg
                            ));
                        }
                    }
                },
                Event::Died { organism, .. } => {
                    let corpse = world.organisms.iter().find(|o| o.id == organism);
                    let (kingdom, mg) =
                        corpse.map_or(("?", 0), |o| (kingdom_name(o.kingdom()), o.biomass_mg()));
                    *died_by.entry(kingdom).or_insert(0u32) += 1;
                    corpses.insert(organism.0);
                    corpse_mg_made += mg;
                    let slot = made_by_century
                        .entry(envelope.tick / 100 * 100)
                        .or_default()
                        .entry(kingdom)
                        .or_default();
                    slot.0 += 1;
                    slot.1 += mg;
                    if let Some(t) = tracks.iter_mut().find(|t| t.id == organism.0) {
                        let (pos, d, n, _) = t.pre.expect("a death follows a lived tick");
                        let o = corpse.expect("a fresh corpse is in the roster");
                        let cause = if o.biomass_mg() <= 20 {
                            "starved"
                        } else {
                            "aged"
                        };
                        let moved = match t.last_move {
                            Some((tick, voxels)) if tick == envelope.tick => {
                                format!("moved {voxels} voxels that tick")
                            },
                            _ => "did not move that tick".to_string(),
                        };
                        t.death = Some(format!(
                            "tick {} cause {cause} (corpse {} mg, age {}), {moved}, carrion standing {n}, \
                             nearest carrion {d:?} voxels, at {pos:?}",
                            envelope.tick,
                            o.biomass_mg(),
                            o.age
                        ));
                        t.reserve.push((envelope.tick, o.energy_mg, o.biomass_mg()));
                    }
                },
                Event::Returned { organism } => {
                    if let Some(k) = kingdoms.get(&organism.0) {
                        *spent_alive_by.entry(k).or_default() += 1;
                    }
                    if let Some(t) = tracks
                        .iter_mut()
                        .find(|t| t.id == organism.0 && t.death.is_none())
                    {
                        t.death = Some(format!(
                            "tick {} biomass reached 0 while alive (spent, no corpse)",
                            envelope.tick
                        ));
                    }
                },
                Event::Born {
                    organism,
                    parent: Some(_),
                    ..
                } => {
                    if let Some(o) = world.organisms.iter().find(|o| o.id == organism) {
                        *births_by.entry(kingdom_name(o.kingdom())).or_default() += 1;
                    }
                },
                _ => {},
            }
        }
        history.record_all(events);
        decayed += world
            .drain_flows()
            .iter()
            .filter(|f| f.record.process == Flow::Decay)
            .map(|f| f.record.amount_mg)
            .sum::<u64>();
        if world.epoch != seen {
            seen = world.epoch;
            world.reckon(&history);
        }
    }

    let stepped_ms = clock.elapsed().as_millis();
    let end_carrion = carrion_now(&world);
    context(&world, world.tick, &end_carrion);
    if deep {
        let hash = state_hash(&world) == state_hash(&control);
        let past = history == control_history;
        println!(
            "faithful to run_deep_time: state_hash {} history {}",
            if hash { "MATCH" } else { "DIFFER" },
            if past { "MATCH" } else { "DIFFER" }
        );
    }
    for t in &tracks {
        println!("-- decomposer #{}", t.id);
        println!(
            "   moved {} times, {} voxels travelled, largest move {} (hungry budget {}), net {} from \
             start, {} distinct stances, places {:?}, moves closing on nearest carrion {}",
            t.moves,
            t.travelled,
            t.largest_move,
            t.budget,
            chebyshev(t.start, t.last),
            t.visited.len(),
            t.places,
            t.closing
        );
        for m in &t.first_moves {
            println!("   move {m}");
        }
        println!(
            "   ticks lived near {} far {}, hungry (budget<8 ticks) {}, closest carrion ever {:?}",
            t.near_ticks, t.far_ticks, t.hungry, t.closest
        );
        println!(
            "   nearest-carrion per lived tick: <=6 {} | 7-12 {} | 13-24 {} | 25-48 {} | >48 {} | none {}",
            t.hist[0], t.hist[1], t.hist[2], t.hist[3], t.hist[4], t.hist[5]
        );
        println!(
            "   meals {} totalling {} mg: {:?}",
            t.meals.len(),
            t.meals.iter().map(|m| m.1).sum::<u64>(),
            t.meals
        );
        println!("   reserve (tick, reserve mg, biomass mg): {:?}", t.reserve);
        println!(
            "   death: {}",
            t.death.as_deref().unwrap_or("alive at epoch end")
        );
    }
    println!(
        "epoch summary: corpses standing at start {}, distinct corpses seen {}, max standing {}, \
         standing at end {} ({} mg), deaths by kingdom {:?} ({} mg of corpse made), \
         births by kingdom {:?}, scavenged {} mg, carrion decayed to soil {} mg; \
         spent while alive (biomass 0, no corpse) by kingdom {:?}, moves (count, voxels) by kingdom {:?}, \
         moves over the hungry budget and largest move by kingdom {:?}",
        start_carrion.len(),
        corpses.len(),
        max_standing,
        end_carrion.len(),
        end_carrion.iter().map(|c| c.2).sum::<u64>(),
        died_by,
        corpse_mg_made,
        births_by,
        scavenged,
        decayed,
        spent_alive_by,
        travel_by,
        over_budget_by
    );
    println!("corpses made per 100 ticks (kingdom: count, mg): {made_by_century:?}");
    println!("stepped epoch wall time {stepped_ms} ms (instrumented, release)");
}

/// The world with `controlled` cleared, through its postcard snapshot.
///
/// Postcard writes fields in declaration order with no names: `tick`, `epoch`,
/// `rules`, then `rng` (one `u64` varint), then `controlled` (`world.rs:153-198`;
/// `ruleset` is skipped). The prefix is re-encoded from public reads, so the
/// field is located rather than searched for, and every step is checked.
fn released(world: &World) -> World {
    let bytes = snapshot(world).expect("a world encodes");
    let same = restore(&bytes).expect("and decodes");
    assert_eq!(
        state_hash(&same),
        state_hash(world),
        "the round trip is exact"
    );
    let prefix = postcard::to_allocvec(&(world.tick, world.epoch, world.rules())).unwrap();
    assert!(
        bytes.starts_with(&prefix),
        "tick, epoch and rules lead the snapshot"
    );
    let rng_len = 1 + bytes[prefix.len()..]
        .iter()
        .position(|b| b & 0x80 == 0)
        .expect("the rng varint ends");
    let at = prefix.len() + rng_len;
    let id = world.controlled_id().expect("a world opens under the hand");
    let some = postcard::to_allocvec(&Some(id)).unwrap();
    assert_eq!(
        &bytes[at..at + some.len()],
        &some[..],
        "controlled follows rng"
    );
    let mut edited = bytes[..at].to_vec();
    edited.extend(postcard::to_allocvec(&None::<OrganismId>).unwrap());
    edited.extend(&bytes[at + some.len()..]);
    let released = restore(&edited).expect("the released world decodes");
    assert_eq!(released.controlled_id(), None);
    assert_eq!(released.organisms, world.organisms, "no body moved");
    assert_eq!(released.soil(), world.soil(), "no soil moved");
    assert_eq!(snapshot(&released).unwrap(), edited, "and nothing else did");
    released
}

fn context(world: &World, tick: u64, carrion: &[(u32, [i32; 3], u64)]) {
    let mut living: BTreeMap<&str, u32> = BTreeMap::new();
    for o in world.organisms.iter().filter(|o| o.is_alive()) {
        *living.entry(kingdom_name(o.kingdom())).or_default() += 1;
    }
    println!(
        "   [t{tick}] living {living:?} carrion {} ({} mg) soil {} mg",
        carrion.len(),
        carrion.iter().map(|c| c.2).sum::<u64>(),
        world.soil().total_mg()
    );
}
