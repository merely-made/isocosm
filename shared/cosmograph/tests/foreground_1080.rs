// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round's foreground (cases 1080 to 1084): the set a participant's round
//! steps, capped by a world rule, and the minute's handback, checked by the
//! sim's invariants, published when accepted and recorded either way.

use cosmograph::{
    Command, Execution, Founding, Session,
    attention::{Region, WindowAttention},
    bodied::Bodies,
    directing::Played,
    map::{Grid, Layout},
    schema::{Id, Method},
    spatial::{ArrivalGeometry, BirthResolution, Handback, Publication, SpatialAdvance, embodiment},
};
use std::collections::BTreeMap;

/// Seed 8's played world (seed 7's under case 1106's floor founds no posed
/// deliberative critter) with whole-body stands (`embodiment::declare`),
/// a posed deliberative founder taken up and its tile framed.
fn framed(cap: Option<u32>) -> (Session, Id, Id) {
    framed_with(cap, false)
}

/// The same, on the whole-site habitat's stands where `habitat` (v6).
fn framed_with(cap: Option<u32>, habitat: bool) -> (Session, Id, Id) {
    framed_ruled(cap, habitat, |_| {})
}

/// The same, its genesis rules then set by `rule`.
fn framed_ruled(cap: Option<u32>, habitat: bool, rule: impl FnOnce(&mut cosmograph::simulation::Genesis)) -> (Session, Id, Id) {
    let mut g = Founding {
        seed: 8, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(8) })),
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default()
    }.generate().unwrap();
    if habitat { embodiment::declare_site(&mut g).unwrap() } else { embodiment::declare(&mut g).unwrap() }
    g.rules.limits.foreground = cap;
    rule(&mut g);
    let t = g.rules.spatial_templates.clone().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let p: u64 = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    let critter = s.sim.state().spatial_poses.keys().copied()
        .find(|id| s.sim.state().population.get(*id).is_some_and(|e| e.alive && e.method == Method::Deliberative))
        .expect("a posed deliberative founder");
    s.command(Command::Take { participant: p, critter }).unwrap();
    let at = s.sim.state().spatial_poses[&critter].clone();
    let w = embodiment::site_window(&t, at.pose.at);
    let frame = Region::Frame { site: at.site, min: w.min, max: w.max };
    s.attach_window(p, "mesocosm".into(), WindowAttention { examined: Some(frame), ..Default::default() }).unwrap();
    (s, p, critter)
}

/// One spatial minute, refusing every birth placement it asks for.
fn minute(s: &mut Session, handbacks: &[Handback]) {
    let mut births = Vec::new();
    loop {
        match s.try_advance_round_spatial(1, &[], &[], &[], &births, handbacks).unwrap() {
            SpatialAdvance::Advanced(_) => return,
            SpatialAdvance::Birth(p) => births.push(BirthResolution { request: p.request, geometry: Err("test".into()) }),
            _ => panic!("the fixture needs only birth answers"),
        }
    }
}

/// The critter's handback to another supported stance in its site.
fn moved(s: &Session, critter: Id) -> Handback {
    let sim = &s.sim;
    let t = sim.genesis().rules.spatial_templates.clone().unwrap();
    let held = sim.state().spatial_poses[&critter].clone();
    let placed: BTreeMap<Id, _> = sim.state().spatial_poses.iter()
        .filter(|(id, r)| **id != critter && r.site == held.site).map(|(id, r)| (*id, r.pose.clone())).collect();
    let pose = embodiment::place_within(sim, &t, critter, held.site, &t.window, &placed).unwrap();
    let e = sim.state().population.get(critter).unwrap();
    Handback {
        actor: critter, site: held.site, body_revision: e.body_revision, terrain_revision: sim.terrain_revision(),
        geometry: ArrivalGeometry { window: embodiment::site_window(&t, pose.at),
            eye: std::array::from_fn(|k| pose.at[k] + t.eye[k]), pose, places: t.places },
        contacts: Vec::new(),
    }
}

#[test]
fn the_foreground_is_the_frames_set_played_first_and_capped() {
    let (s, p, critter) = framed(None);
    let set = s.sim.foreground(p);
    assert_eq!(set.first(), Some(&critter), "the played body leads");
    assert!(set.len() > 1 && set.len() <= 16, "{} bodies", set.len());
    assert_eq!(set, s.sim.foreground(p), "the same world gives the same set");
    for id in &set { assert!(s.sim.state().spatial_poses.contains_key(id) && s.sim.near(*id)); }
    let (s, p, critter) = framed(Some(2));
    assert_eq!(s.sim.foreground(p).len(), 2);
    assert_eq!(s.sim.foreground(p)[0], critter);
    let (s, _, _) = framed(None);
    assert!(s.sim.foreground(9_999_999).is_empty(), "no participant, no foreground");
}

#[test]
fn an_accepted_handback_is_published_and_replays() {
    let (mut s, _, critter) = framed(None);
    let h = moved(&s, critter);
    minute(&mut s, std::slice::from_ref(&h));
    assert!(s.sim.spatial_handbacks().iter().all(|r| r.actor == critter && r.patch.is_ok()), "{:?}", s.sim.spatial_handbacks());
    assert!(s.sim.spatial_order().iter().any(|o| matches!(o, Publication::Drawn { actor, pose, .. }
        if *actor == critter && *pose == h.geometry.pose)), "the accepted pose is published as a draw");
    let entry = s.entries.last().unwrap();
    assert!(entry.outcome.contains("handbacks"), "the handback is recorded: {}", entry.outcome);
    let restored = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(restored.sim.state_hash(), s.sim.state_hash(), "the handback replays by command");
}

#[test]
fn a_refused_handback_leaves_the_source_unchanged_and_is_recorded() {
    let (s, _, critter) = framed(None);
    let mut stale = moved(&s, critter);
    stale.body_revision += 1;
    let mut far = moved(&s, critter);
    // The farthest other body posed at the critter's site.
    let site = s.sim.state().spatial_poses[&critter].site;
    let at = far.geometry.pose.at;
    let food = s.sim.state().spatial_poses.iter().filter(|(id, r)| **id != critter && r.site == site)
        .max_by_key(|(_, r)| r.pose.at[0].abs_diff(at[0]).max(r.pose.at[2].abs_diff(at[2])))
        .map(|(id, _)| *id).expect("another body at the site");
    far.contacts = vec![food];
    let mut plain = s.clone();
    minute(&mut plain, &[]);
    for (refused, why) in [(stale, "source changed"), (far, "beyond feeding reach")] {
        let mut tried = s.clone();
        minute(&mut tried, std::slice::from_ref(&refused));
        let report = tried.sim.spatial_handbacks();
        assert!(report.len() == 1 && report[0].patch.as_ref().is_err_and(|e| e.contains(why)), "{report:?}");
        assert_eq!(tried.sim.state_hash(), plain.sim.state_hash(), "a refusal publishes nothing");
        assert!(tried.entries.last().unwrap().outcome.contains("handbacks"), "a refusal is recorded");
        let restored = Session::load(tried.save(), Execution::Individuals).unwrap();
        assert_eq!(restored.sim.state_hash(), tried.sim.state_hash());
    }
}

/// Twelve spatial minutes recorded before rounds existed (at `c8df9563`):
/// the optional handback field leaves an older save's bytes and hash as
/// they were.
#[test]
fn an_older_spatial_save_keeps_its_bytes_and_hash() {
    let bytes = include_bytes!("fixtures/spatial_minutes_before_1080.json");
    let saved: cosmograph::history::Saved = serde_json::from_slice(bytes).unwrap();
    assert!(saved.entries.iter().all(|e| matches!(&e.command,
        Command::AdvanceBirthSpatial { handbacks, .. } if handbacks.is_empty())));
    let s = Session::load_json(bytes, Execution::Grouped).unwrap();
    assert_eq!(s.sim.state_hash(), saved.state_hash);
    assert_eq!(serde_json::to_vec(&s.save()).unwrap(), bytes.to_vec(), "saving again writes the same bytes");
}

/// The cost of one foreground step per body (case 1082), in release:
/// `cargo test --release --offline --test foreground_1080 -- --ignored --nocapture step_cost`.
/// A step here is what each body needs each sixtieth of a second: the
/// stances a column offers (`places::steps`), the whole-body stance check on
/// the next pose (`embodiment::check_member`) and its clearance from the
/// other bodies posed beside it.
#[test]
#[ignore = "a release measurement; run explicitly"]
fn step_cost() {
    let (s, p, _) = framed_with(Some(256), true);
    let sim = &s.sim;
    let t = sim.genesis().rules.spatial_templates.clone().unwrap();
    let bodies = sim.foreground(p);
    let (mut columns, mut stances, mut clear) = (std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO);
    let rounds = 60u32;
    let mut steps = 0u32;
    for &id in &bodies {
        let r = &sim.state().spatial_poses[&id];
        let e = sim.state().population.get(id).unwrap();
        let doc = e.body.as_ref().unwrap();
        let geometry = sim.site_geometry(r.site, &embodiment::site_window(&t, r.pose.at), t.places).unwrap();
        let others: Vec<_> = sim.state().spatial_poses.iter().filter(|(o, q)| **o != id && q.site == r.site)
            .filter_map(|(o, q)| embodiment::boxes(sim.state().population.get(*o)?.body.as_ref()?, &q.pose).ok())
            .flatten().collect();
        // The held pose passes every check, so each check runs whole rather
        // than stopping at an early refusal.
        assert!(embodiment::check_member(sim, &geometry.volume, e, &r.pose).is_ok(), "body {id} stands");
        for _ in 0..rounds {
            let next = r.pose.clone();
            let a = std::time::Instant::now();
            std::hint::black_box(isometer_space::places::steps(&geometry.volume, t.places, r.pose.at));
            let b = std::time::Instant::now();
            std::hint::black_box(embodiment::check_member(sim, &geometry.volume, e, &next).is_ok());
            let c = std::time::Instant::now();
            let own = embodiment::boxes(doc, &next).unwrap();
            std::hint::black_box(embodiment::intersects(&own, &others));
            let d = std::time::Instant::now();
            columns += b - a; stances += c - b; clear += d - c;
            steps += 1;
        }
    }
    // An upper bound for a step that must re-plant its feet: the nearest
    // balanced stance by the search arrivals and settling use (D38).
    let mut planted = std::time::Duration::ZERO;
    for &id in &bodies {
        let r = &sim.state().spatial_poses[&id];
        let placed: BTreeMap<Id, _> = sim.state().spatial_poses.iter().filter(|(o, q)| **o != id && q.site == r.site)
            .map(|(o, q)| (*o, q.pose.clone())).collect();
        let a = std::time::Instant::now();
        std::hint::black_box(embodiment::settle(sim, &t, id, r.site, &r.pose, &placed).is_ok());
        planted += a.elapsed();
    }
    eprintln!("STEP_REPLANT {:.1} us a body", planted.as_secs_f64() * 1e6 / bodies.len().max(1) as f64);
    let per = |d: std::time::Duration| d.as_secs_f64() * 1e6 / f64::from(steps.max(1));
    let total = per(columns) + per(stances) + per(clear);
    eprintln!("STEP_COST bodies {} steps {steps}: columns {:.1} us, stance {:.1} us, clearance {:.1} us, total {:.1} us a body a step; \
        {:.0} bodies fill a 60 Hz frame", bodies.len(), per(columns), per(stances), per(clear), total, 1e6 / 60.0 / total);
}

/// Cases 1098 and 1099: in a world whose templates admit at cell grain, a
/// body that holds a patch but no pose is posed by its first handback, on
/// a clear root and eye inside one patch; without a patch it is refused.
#[test]
fn a_first_handback_poses_a_patched_body_at_cell_grain() {
    let g = Founding {
        seed: 8, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(8) })),
        ..Default::default()
    }.generate().unwrap();
    let t = g.rules.spatial_templates.clone().unwrap();
    assert!(t.version < 2, "these templates admit at cell grain");
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let (site, stand) = t.stands.iter().find_map(|(site, st)| st.first().map(|x| (*site, x.clone()))).unwrap();
    let actor = s.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.body.is_some() && g.entity.resident_site() == Some(site))
        .map(|(id, _)| *id).expect("an embodied resident");
    assert!(!s.sim.state().spatial_poses.contains_key(&actor));
    let handback = |s: &Session| {
        let e = s.sim.state().population.get(actor).unwrap();
        Handback { actor, site, body_revision: e.body_revision, terrain_revision: s.sim.terrain_revision(),
            geometry: ArrivalGeometry { window: t.window.clone(), eye: std::array::from_fn(|k| stand.at[k] + t.eye[k]),
                pose: cosmograph::spatial::Pose { at: stand.at, yaw: stand.yaw, feet: Default::default() }, places: t.places },
            contacts: Vec::new() }
    };
    let mut bare = s.clone();
    let h = handback(&bare);
    minute(&mut bare, &[h]);
    assert!(bare.sim.spatial_handbacks()[0].patch.as_ref().is_err_and(|e| e.contains("no pose or patch")));
    let patch = s.sim.site_geometry(site, &t.window, t.places).unwrap().places.place_at(stand.at).unwrap();
    s.command(Command::Patch { entity: actor, patch: Some(patch) }).unwrap();
    let h = handback(&s);
    minute(&mut s, &[h]);
    assert!(s.sim.spatial_handbacks()[0].patch.is_ok(), "{:?}", s.sim.spatial_handbacks());
    assert_eq!(s.sim.state().spatial_poses[&actor].pose.at, stand.at, "the first handback poses the body");
    let restored = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(restored.sim.state_hash(), s.sim.state_hash());
}

/// Case 1113: with world speeds a handback lands no farther from the held
/// pose than the body walks in a tick; without them it keeps no such bound.
#[test]
fn a_handback_beyond_a_ticks_walk_is_refused() {
    let crawl = cosmograph::speed::Speed { baseline: 1, leg: 1, legless: 1 };
    let (mut slow, _, critter) = framed_ruled(None, false, |g| g.rules.speed = Some(crawl));
    let (mut free, _, _) = framed_ruled(None, false, |g| g.rules.speed = None);
    // The same handback, moved to a stand of its site four or more cells off.
    let near = moved(&slow, critter);
    let held = slow.sim.state().spatial_poses[&critter].pose.at;
    let t = slow.sim.genesis().rules.spatial_templates.clone().unwrap();
    let stand = t.stands[&near.site].iter()
        .find(|x| (0..3).step_by(2).any(|k| x.at[k].abs_diff(held[k]) >= 4)).expect("a far stand").clone();
    let mut far = near.clone();
    far.geometry.pose = cosmograph::spatial::Pose { at: stand.at, yaw: stand.yaw, feet: Default::default() };
    far.geometry.eye = std::array::from_fn(|k| stand.at[k] + t.eye[k]);
    far.geometry.window = embodiment::site_window(&t, stand.at);
    let mut moved_slow = slow.clone();
    minute(&mut moved_slow, std::slice::from_ref(&near));
    assert!(moved_slow.sim.spatial_handbacks()[0].patch.is_ok(), "a step within reach: {:?}", moved_slow.sim.spatial_handbacks());
    minute(&mut slow, std::slice::from_ref(&far));
    assert!(slow.sim.spatial_handbacks()[0].patch.as_ref().is_err_and(|e| e.contains("reach in a tick")),
        "{:?}", slow.sim.spatial_handbacks());
    assert_eq!(slow.sim.state().spatial_poses[&critter].pose.at, held, "the body stays where the sim had it");
    minute(&mut free, std::slice::from_ref(&far));
    assert!(!free.sim.spatial_handbacks()[0].patch.as_ref().is_err_and(|e| e.contains("reach")),
        "a world without speeds keeps no reach bound: {:?}", free.sim.spatial_handbacks());
    let restored = Session::load(slow.save(), Execution::Individuals).unwrap();
    assert_eq!(restored.sim.state_hash(), slow.sim.state_hash());
}
