// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! E10 step 5: a world founded with `--foreground-walks` steps its foreground
//! each world minute with no window, hands the poses back, and the round
//! replays to its hash (D51, D52). The founding path is the one the binary
//! takes (`startup::configured`).

use super::*;
use mesocosm_runtime::world_rules::WorldRules;
use cosmograph_host::drive::{Pace, Pacer, Speed};

const SEED: u64 = 18_181_373_014_846_284_518;
/// Real time each speed runs, in microseconds: one minute.
const REAL_US: u64 = 60_000_000;

fn found(rules: WorldRules) -> Runtime {
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let config = HostConfig { seed: SEED, play: true, world_rules: rules, ecology_rates: Some(cosmograph::ecology::Rates::A1),
        ..Default::default() };
    startup::configured(&config, &authored).unwrap_or_else(|why| panic!("did not found: {why}"))
}

fn handbacks(runtime: &Runtime) -> (usize, usize) {
    let records = &runtime.spatial().driver.as_ref().unwrap().records;
    let made = records.iter().map(|r| r.handbacks.len()).sum();
    let taken = records.iter().map(|r| r.handback_reports.iter().filter(|h| h.patch.is_ok()).count()).sum();
    (made, taken)
}

#[test]
fn the_flag_sets_the_world_rule_and_no_default_does() {
    assert!(!WorldRules::generated().foreground_walks && !WorldRules::all().foreground_walks);
    assert!(!HostConfig::default().world_rules.foreground_walks);
    assert!(!serde_json::to_string(&WorldRules::generated()).unwrap().contains("foreground_walks"));
    let rules = WorldRules { foreground_walks: true, ..WorldRules::generated() };
    assert!(found(rules).sim().genesis().rules.foreground_walks);
    assert!(!found(WorldRules::generated()).sim().genesis().rules.foreground_walks);
}

#[test]
fn a_round_with_the_rule_replays_to_its_hash_at_each_speed() {
    let rules = WorldRules { foreground_walks: true, ..WorldRules::generated() };
    let mut most = 0;
    for speed in Speed::ALL {
        let ticks = Pacer::new(Pace::Round { speed }).advance(REAL_US).ticks;
        assert_eq!(ticks, speed.factor());
        let mut runtime = found(rules);
        runtime.step(ticks);
        let (made, taken) = handbacks(&runtime);
        assert!(taken <= made, "{speed:?}");
        most = most.max(taken);
        let hash = runtime.sim().state_hash();
        let replayed = Runtime::replayed_game(runtime.save_game())
            .unwrap_or_else(|why| panic!("{speed:?}: a round with handbacks must replay: {why}"));
        assert_eq!(replayed.state_hash, hash, "{speed:?}: the round replays to its recorded hash");
    }
    assert!(most > 0, "some speed's round handed a body back: the replay checks nothing otherwise");
}

fn windowed(speed: Speed) -> Host {
    let config = HostConfig { seed: SEED, play: true, ecology_rates: Some(cosmograph::ecology::Rates::A1),
        world_rules: WorldRules { foreground_walks: true, ..WorldRules::generated() }, round_speed: speed,
        ..Default::default() };
    Host::new(config)
}

/// A window's frames, as the frame loop takes them.
const FRAME_US: u64 = 16_667;
/// World minutes the window runs below.
const MINUTES: u64 = 3;

#[test]
fn the_walks_the_window_draws_are_the_handbacks_the_clock_takes() {
    let mut runtime = found(WorldRules { foreground_walks: true, ..WorldRules::generated() });
    // What a window does before it draws minute 0 (D70).
    runtime.attach_foreground();
    let participant = runtime.interim().participant;
    let (mut walked, mut expected) = (0, Vec::new());
    for _ in 0..MINUTES {
        let sim = runtime.sim();
        let walks = cosmograph_host::drive::world_foreground_walks(&sim, participant);
        let minute = cosmograph_host::drive::world_foreground_minute(&sim, participant).unwrap();
        let made: Vec<_> = walks.iter().filter_map(|w| w.handback(&sim, 60).unwrap()).collect();
        assert_eq!(made, minute, "the window's walks hand back what the clock is given");
        walked += walks.len();
        expected.push(made);
        drop(sim);
        runtime.step(1);
    }
    let records = &runtime.spatial().driver.as_ref().unwrap().records;
    for (minute, record) in records.iter().enumerate() {
        assert_eq!(expected[minute], record.handbacks, "minute {minute}: the round hands back the walks read before it");
    }
    assert!(walked > 0, "some body walked: the comparison checks nothing otherwise");
}

#[test]
fn a_windowed_round_draws_each_step_and_replays_to_the_headless_hash() {
    let mut host = windowed(Speed::X8);
    let (mut moved, mut still_at_start) = (0, 0);
    // The last pose drawn of each walking body this minute.
    let mut last: std::collections::BTreeMap<u64, [i64; 3]> = Default::default();
    let mut tick = 0;
    let mut drawn = std::collections::BTreeMap::new();
    let frames = MINUTES * 7_500_000 / FRAME_US + 10;
    // Then wait out the minutes on their worker, still drawing.
    for frame in 0..frames + 20_000 {
        if frame < frames {
            host.advance_by(FRAME_US);
        } else {
            host.advance_by(0);
            if tick == MINUTES { break }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        host.refresh_scene();
        let now = host.runtime.sim().state().tick;
        if now != tick {
            // The minute's last drawn stance is where the sim now has the body.
            for (id, at) in std::mem::take(&mut last) {
                let sim = host.runtime.sim().state().spatial_poses.get(&id).map(|p| p.pose.at);
                assert_eq!(sim, Some(at), "body {id} is where the minute drew it: no jump at the minute");
            }
            tick = now;
        }
        let step = host.foreground_step().unwrap();
        let scene = host.foreground_scene();
        drawn.entry(now).or_insert_with(|| host.drawn_handbacks());
        let Some(scene) = scene else {
            still_at_start += 1;
            continue;
        };
        let source = host.scene.clone().unwrap();
        for body in &scene.bodies {
            let was = source.body(body.id).and_then(|b| b.physical.as_ref()).map(|p| p.at);
            let drawn = body.physical.as_ref().map(|p| p.at);
            if was != drawn {
                assert!(step >= 1, "a body stands at its first stance at step 0");
                moved += 1;
                last.insert(body.id, drawn.unwrap());
            }
        }
    }
    assert!(moved > 0 && still_at_start > 0, "bodies were drawn both standing and stepped");
    assert_eq!(host.runtime.sim().state().tick, MINUTES);
    let hash = host.runtime.sim().state_hash();
    let mut headless = found(WorldRules { foreground_walks: true, ..WorldRules::generated() });
    headless.step(MINUTES);
    assert_eq!(headless.sim().state_hash(), hash, "a windowed round and a headless one end alike");
    // What the window drew a minute from is what the clock was handed, from minute 0.
    let records = &host.runtime.spatial().driver.as_ref().unwrap().records;
    assert_eq!(records.len() as u64, MINUTES);
    for (minute, record) in records.iter().enumerate() {
        assert_eq!(drawn[&(minute as u64)], record.handbacks, "minute {minute}: drawn walks are the handbacks");
    }
    let replayed = Runtime::replayed_game(host.runtime.save_game()).unwrap();
    assert_eq!(replayed.state_hash, hash, "the windowed round replays to its recorded hash");
}

