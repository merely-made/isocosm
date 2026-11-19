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
