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

/// Minutes run for a journey to depart and land: a traveller leaves in the
/// minute the sim decides its walk and lands at the end of the next one, so
/// two minutes at least; four let a second decision or two come due.
const MINUTES: u32 = 4;

/// E10 step 3 over the seeds whose worlds put a hungry body with a route in
/// the foreground. For each, from one starting state, the body is stepped and
/// handed back each of `MINUTES` minutes, and the same state is run with the
/// participant's window closed so the body travels out of view (D60).
///
/// A release run, a few minutes:
/// `cargo test --release --lib stepper_tests -- --ignored --nocapture`
///
/// The stepped minutes are accepted and deterministic, and some traveller
/// moves (the control's positive control). The two displacements do not agree
/// within a cell and the test does not assert that they do: see the plan's
/// Findings of 2026-11-18 for the two causes.
#[test]
#[ignore = "a release measurement over many founded worlds"]
fn stepped_bodies_are_compared_with_out_of_view_travellers() {
    const WAITS: u32 = 15;
    let (mut rows, mut stepper, mut travel, mut moved) = (0, Vec::new(), Vec::new(), 0);
    println!("seed body edges | stepped cells | travelled cells | same meal | native 1 min");
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

        // (a) stepped and handed back each minute: each handback accepted.
        let mut stepped = w.s.clone();
        for m in 0..MINUTES {
            let held = at(&stepped, id);
            let Some(h) = Walk::begin(&stepped.sim, id, &w.walk).ok().flatten()
                .and_then(|walk| walk.handback(&stepped.sim, STEPS_PER_MINUTE).unwrap()) else {
                minute(&mut stepped, &[]);
                continue;
            };
            assert!(cells(held, h.geometry.pose.at) <= reach, "seed {seed} minute {m}: within a tick's walk");
            minute(&mut stepped, std::slice::from_ref(&h));
            let report = stepped.sim.spatial_handbacks();
            assert!(report.iter().any(|r| r.actor == id && r.patch.is_ok()), "seed {seed} minute {m}: {report:?}");
        }

        // The control: the same state with no window, so the body travels.
        let mut away = w.s.clone();
        away.close_attention(w.p, w.window).unwrap();
        let mut meal = None;
        for _ in 0..MINUTES {
            minute(&mut away, &[]);
            meal = meal.or(away.sim.state().spatial_poses[&id].travel.as_ref().map(|t| t.target));
        }
        // The sim's own in-view minute, no handback.
        let mut free = w.s.clone();
        minute(&mut free, &[]);

        let (s, t) = (cells(start, at(&stepped, id)), cells(start, at(&away, id)));
        println!("{seed:>4} {id:>4} {:>5} | {s:>13.1} | {t:>15.1} | {:>9} | {:.1}",
            walk.route().path.len() - 1,
            meal.map_or("none".into(), |m| (m == walk.route().target).to_string()), cells(start, at(&free, id)));
        stepper.push(s);
        travel.push(t);
        moved += usize::from(t > 0.0);
        rows += 1;
        if rows == 10 { break }
    }
    assert!(rows >= 8, "{rows} seeds put a walker in the foreground");
    assert!(moved > 0, "the control never moved a body: its zeros prove nothing");
    println!("MEDIANS over {rows} seeds, {MINUTES} minutes: stepped {:.1} cells, travelled {:.1} cells",
        median(&mut stepper.clone()), median(&mut travel.clone()));
}
