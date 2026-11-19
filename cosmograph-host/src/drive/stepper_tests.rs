// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The foreground stepper against the sim (E10 step 3). A world is the
//! foreground test world at walking speed (`cosmograph/tests/foreground_1080.rs`);
//! a native minute answers the sim's step requests with its own provider,
//! as a game does, so a body walks without a handback.

use super::stepper::*;
use cosmograph::{
    Command, Execution, Founding, Session,
    attention::{Region, WindowAttention},
    bodied::Bodies,
    directing::Played,
    map::{Grid, Layout},
    schema::{Id, Method},
    spatial::{BirthResolution, Handback, Resolution, SpatialAdvance, TerrainResolution, check_step, embodiment, foraging},
};

struct World { s: Session, p: Id, window: cosmograph::attention::WindowId, walk: String }

fn world(seed: u64) -> Option<World> {
    let mut g = Founding {
        seed, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(seed) })),
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default()
    }.generate().ok()?;
    embodiment::declare(&mut g).ok()?;
    let t = g.rules.spatial_templates.clone()?;
    let walk = t.foraging.keys().next()?.clone();
    let mut s = Session::new(g, Execution::Individuals).ok()?;
    let p: u64 = s.command(Command::Join).ok()?.strip_prefix("participant:")?.parse().ok()?;
    let critter = s.sim.state().spatial_poses.keys().copied()
        .find(|id| s.sim.state().population.get(*id).is_some_and(|e| e.alive && e.method == Method::Deliberative))?;
    s.command(Command::Take { participant: p, critter }).ok()?;
    let at = s.sim.state().spatial_poses[&critter].clone();
    let w = embodiment::site_window(&t, at.pose.at);
    let frame = Region::Frame { site: at.site, min: w.min, max: w.max };
    let window = s.attach_window(p, "mesocosm".into(), WindowAttention { examined: Some(frame), ..Default::default() }).ok()?;
    Some(World { s, p, window, walk })
}

/// One minute with the sim's own steps answered by its provider, as a game's
/// would be, and `handbacks` applied.
pub(super) fn minute(s: &mut Session, handbacks: &[Handback]) {
    let (mut births, mut steps, mut terrain) = (Vec::new(), Vec::new(), Vec::new());
    loop {
        match s.try_advance_round_spatial(1, &terrain, &steps, &[], &births, handbacks).unwrap() {
            SpatialAdvance::Advanced(_) => return,
            SpatialAdvance::Birth(p) => births.push(BirthResolution { request: p.request, geometry: Err("test".into()) }),
            SpatialAdvance::Terrain(p) => terrain.push(TerrainResolution { request: p.request, geometry: Err("test".into()) }),
            SpatialAdvance::Step(p) => {
                let window = embodiment::site_window(p.sim.genesis().rules.spatial_templates.as_ref().unwrap(),
                    p.sim.state().spatial_poses[&p.request.actor].pose.at);
                let geometry = foraging::geometry(&p.sim, &p.request, &window)
                    .and_then(|g| check_step(&p.sim, &p.request, &g).map(|_| g));
                steps.push(Resolution { request: p.request, geometry });
            }
        }
    }
}

pub(super) fn cells(a: [i64; 3], b: [i64; 3]) -> f64 {
    let [dx, dz] = [0, 2].map(|k| (a[k] - b[k]) as f64);
    (dx * dx + dz * dz).sqrt()
}


/// A foreground body of the world with a walk, once the world has run up to
/// `waits` minutes.
fn hungry(w: &mut World, waits: u32) -> Option<(Id, Walk)> {
    for _ in 0..=waits {
        let played = w.s.sim.played(w.p);
        for id in w.s.sim.foreground(w.p) {
            if played.contains(&id) { continue }
            if let Ok(Some(walk)) = Walk::begin(&w.s.sim, id, &w.walk) { return Some((id, walk)) }
        }
        minute(&mut w.s, &[]);
    }
    None
}

fn at(s: &Session, id: Id) -> [i64; 3] { s.sim.state().spatial_poses[&id].pose.at }

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// E10 step 3 over the seeds whose worlds put a hungry body with a route in
/// the foreground (at most `WAITS` minutes in): the stepper's minute is
/// accepted by the sim and replays to the same handback, and its
/// displacement is set beside the sim's own minute for the same body.
///
/// A release run, a few minutes:
/// `cargo test --release --lib stepper_tests -- --ignored --nocapture`
///
/// The distribution does not agree today, so this test measures and bounds
/// rather than asserting agreement: native walking publishes the first edge
/// of the route each minute (`foraging::geometry`), while the stepper walks
/// the body's speed (49.5 cells a second at these cells). Which of the two
/// moves is the open fork in the report (the plan's unruled reach).
#[test]
#[ignore = "a release measurement over many founded worlds"]
fn stepped_minutes_are_accepted_and_compared_with_native_ones() {
    const WAITS: u32 = 15;
    let (mut rows, mut stepper, mut native) = (0, Vec::new(), Vec::new());
    for seed in 1..=80u64 {
        let Some(mut w) = world(seed) else { continue };
        let Some((id, walk)) = hungry(&mut w, WAITS) else { continue };
        let start = at(&w.s, id);
        let reach = {
            let rules = &w.s.sim.genesis().rules;
            let doc = w.s.sim.state().population.get(id).unwrap().body.as_ref().unwrap();
            (u128::from(rules.speed.unwrap().of(doc)) * u128::from(rules.tick_microseconds()) / 1_000_000_000) as f64 + 1.0
        };

        // (c) the same inputs give the same handback, step by step and whole.
        let again = Walk::begin(&w.s.sim, id, &w.walk).unwrap().unwrap();
        let h = walk.handback(&w.s.sim, STEPS_PER_MINUTE).unwrap().expect("a walk that leaves its stance");
        assert_eq!(Some(&h), again.handback(&w.s.sim, STEPS_PER_MINUTE).unwrap().as_ref(), "seed {seed}");
        let mut stepping = again.clone();
        let mut last = 0;
        while let Some(step) = stepping.step() {
            assert!(step.edge >= last, "a body never walks back");
            last = step.edge;
        }
        assert_eq!(stepping.taken(), STEPS_PER_MINUTE);
        assert_eq!(stepping.stance(STEPS_PER_MINUTE), &h.geometry.pose);

        // (a) the sim accepts the minute's handback.
        let mut stepped = w.s.clone();
        minute(&mut stepped, std::slice::from_ref(&h));
        let report = stepped.sim.spatial_handbacks();
        assert!(report.iter().any(|r| r.actor == id && r.patch.is_ok()), "seed {seed}: {report:?}");
        assert!(cells(start, h.geometry.pose.at) <= reach, "seed {seed}: within a tick's walk");

        // The same world, body and minute, no handback: the sim's own step.
        let mut free = w.s.clone();
        minute(&mut free, &[]);
        let (s, n) = (cells(start, at(&stepped, id)), cells(start, at(&free, id)));
        println!("seed {seed} tick {} body {id}: route {} edges, displacement stepper {s:.1} native {n:.1} cells",
            w.s.sim.state().tick, walk.route().path.len() - 1);
        stepper.push(s);
        native.push(n);
        rows += 1;
        if rows == 10 { break }
    }
    assert!(rows >= 8, "{rows} seeds put a walker in the foreground");
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    println!("DISPLACEMENT over {rows} seeds: stepper median {:.1} mean {:.1}, native median {:.1} mean {:.1}",
        median(&mut stepper.clone()), mean(&stepper), median(&mut native.clone()), mean(&native));
}
