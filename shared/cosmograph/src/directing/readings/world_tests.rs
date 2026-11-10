// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::lineage::boundary;
use crate::{Execution, Founding, Session, directing::found::Played, history::Command,
    light::SunRules, rules::year_ticks};

fn founding(seed: u64) -> Founding {
    Founding { seed, sites: 6, population: 72, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }
}

fn world(seed: u64) -> Session {
    Session::new(founding(seed).generate().unwrap(), Execution::Individuals).unwrap()
}

/// Ends every living body `doomed` names.
fn kill(session: &mut Session, doomed: impl Fn(&Entity) -> bool) {
    for group in session.sim.state.population.groups.values_mut() {
        if group.entity.alive && doomed(&group.entity) { group.entity_mut().alive = false; }
    }
}

fn line(session: &Session) -> Key {
    let state = session.sim.state();
    state.population.groups.values().filter(|g| g.entity.alive)
        .map(|g| &g.entity.lineage).find(|l| state.lineages.get(*l).is_some_and(|l| l.kingdom != "kingdom:world"))
        .unwrap().clone()
}

fn only(found: &[Raised], keep: fn(&Raised) -> bool) -> Vec<&Raised> {
    found.iter().filter(|r| keep(r)).collect()
}

#[test]
fn identical_moments_raise_nothing() {
    let s = world(5);
    assert!(raised(&s.sim, &s.sim).is_empty());
    assert!(raised(&s.sim, &s.clone().sim).is_empty());
}

#[test]
fn a_lineage_with_no_living_members_left_is_extinct() {
    let before = world(5);
    let lost = line(&before);
    let mut after = before.clone();
    kill(&mut after, |e| e.lineage == lost);
    let found = raised(&before.sim, &after.sim);
    assert_eq!(only(&found, |r| matches!(r, Raised::Extinct { .. })),
        vec![&Raised::Extinct { lineage: lost.clone() }]);
    assert!(only(&found, |r| matches!(r, Raised::Bloomed { .. } | Raised::Turned { .. })).is_empty());
    // What had no matter before cannot bloom, and nothing was extinct that was never alive.
    let back = raised(&after.sim, &before.sim);
    assert!(only(&back, |r| matches!(r, Raised::Bloomed { .. } | Raised::Extinct { .. }))
        .iter().all(|r| !matches!(r, Raised::Bloomed { lineage, .. } | Raised::Extinct { lineage } if *lineage == lost)));
}

#[test]
fn a_lineage_whose_living_matter_doubled_has_bloomed() {
    let before = world(5);
    let lost = line(&before);
    let mut after = before.clone();
    for group in after.sim.state.population.groups.values_mut().filter(|g| g.entity.lineage == lost) {
        group.count *= 3;
    }
    let found = raised(&before.sim, &after.sim);
    let held = boundary::standing(&after, &lost).0;
    assert!(held >= 2 * boundary::standing(&before, &lost).0);
    assert_eq!(only(&found, |r| matches!(r, Raised::Bloomed { .. })),
        vec![&Raised::Bloomed { lineage: lost, held }]);
    let mut slight = before.clone();
    let group = slight.sim.state.population.groups.values_mut().find(|g| g.count > 3).unwrap();
    group.count += 1;
    assert!(only(&raised(&before.sim, &slight.sim), |r| matches!(r, Raised::Bloomed { .. })).is_empty());
}

#[test]
fn a_region_that_stood_and_lost_every_body_has_fallen() {
    let before = world(5);
    let standing = regions(&before.sim);
    assert!(standing.iter().all(|r| !r.collapsed), "the founded world stands");
    let doomed = standing[0].sites.clone();
    let mut after = before.clone();
    kill(&mut after, |e| e.resident_site().is_some_and(|site| doomed.contains(&site)));
    let found = raised(&before.sim, &after.sim);
    let fell = only(&found, |r| matches!(r, Raised::Fell { .. }));
    assert!(!fell.is_empty());
    let sites = regions(&after.sim).into_iter().find(|r| r.collapsed).unwrap().sites;
    assert_eq!(fell[0], &Raised::Fell { sites });
    // A region already fallen is not a new fall.
    assert!(only(&raised(&after.sim, &after.sim), |r| matches!(r, Raised::Fell { .. })).is_empty());
    assert!(only(&raised(&after.sim, &after.clone().sim), |r| matches!(r, Raised::Fell { .. })).is_empty());
}

#[test]
fn a_site_whose_soil_ran_out_is_barren_and_one_without_soil_is_not() {
    let mut before = world(5);
    let ids: Vec<Id> = before.sim.state().sites.keys().copied().collect();
    before.sim.state.sites.get_mut(&ids[0]).unwrap().accounts.insert("world:soil".into(), 9);
    before.sim.state.sites.get_mut(&ids[1]).unwrap().accounts.remove("world:soil");
    before.sim.state.sites.get_mut(&ids[2]).unwrap().accounts.insert("world:soil".into(), 4);
    let mut after = before.clone();
    after.sim.state.sites.get_mut(&ids[0]).unwrap().accounts.insert("world:soil".into(), 0);
    after.sim.state.sites.get_mut(&ids[2]).unwrap().accounts.remove("world:soil");
    assert_eq!(raised(&before.sim, &after.sim),
        vec![Raised::Barren { site: ids[0] }, Raised::Barren { site: ids[2] }]);
}

#[test]
fn the_year_turns_only_across_its_halves() {
    let before = world(5);
    let year = year_ticks(before.sim.genesis().rules.tick_microseconds());
    let at = |start: Tick, end: Tick| {
        let (mut a, mut b) = (before.clone(), before.clone());
        a.sim.state.tick = start;
        b.sim.state.tick = end;
        raised(&a.sim, &b.sim)
    };
    assert_eq!(at(year / 4, year * 3 / 4), vec![Raised::Turned { rising: false }]);
    assert_eq!(at(year * 3 / 4, year + year / 4), vec![Raised::Turned { rising: true }]);
    assert!(at(year / 8, year / 4).is_empty());
    assert!(at(year * 5 / 8, year * 7 / 8).is_empty());
}

#[test]
fn a_tidally_locked_sun_never_turns() {
    let mut g = founding(5).generate().unwrap();
    g.rules.sun = Some(SunRules { tidal_lock: true, ..SunRules::default() });
    let year = year_ticks(g.rules.tick_microseconds());
    let before = Session::new(g, Execution::Individuals).unwrap();
    let mut after = before.clone();
    after.sim.state.tick = year / 2 + year / 4;
    assert!(raised(&before.sim, &after.sim).is_empty());
}

#[test]
fn the_world_kingdom_and_participants_raise_nothing() {
    let mut before = world(5);
    before.command(Command::Join).unwrap();
    let mut after = before.clone();
    kill(&mut after, |e| e.kingdom == "kingdom:world" || crate::directing::is_participant(e));
    for group in after.sim.state.population.groups.values_mut()
        .filter(|g| g.entity.kingdom == "kingdom:world" || crate::directing::is_participant(&g.entity)) {
        group.count += 100;
    }
    let found = raised(&before.sim, &after.sim);
    let named = |r: &&Raised| matches!(r, Raised::Extinct { lineage } | Raised::Bloomed { lineage, .. }
        if lineage.starts_with("world:") || lineage.starts_with("participant:"));
    assert!(found.iter().find(named).is_none(), "{found:?}");
}
