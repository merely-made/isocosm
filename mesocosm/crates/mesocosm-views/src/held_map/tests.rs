// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{Execution, Session, Founding, directing::Played, spread::{Identity, Substrate}};
use mesocosm_runtime::microbe;

/// A small hosting world, its microbe seeded at the site that holds the seed.
fn seeded() -> (Session, Id) {
    let mut genesis = Founding { seed: 5, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }.generate().unwrap();
    microbe::declare(&mut genesis).unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let site = *session.sim.state().sites.iter().find(|(_, s)| s.accounts.contains_key(microbe::ACCOUNT)).unwrap().0;
    for command in microbe::seeding(site) { session.command(command).unwrap(); }
    (session, site)
}

/// The same world with a few of the seed also placed on a body of the first cohort, and that
/// body's site.
fn seeded_with_a_host() -> (Session, Id, Id) {
    let mut genesis = Founding { seed: 5, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }.generate().unwrap();
    microbe::declare(&mut genesis).unwrap();
    let site = *genesis.sites.iter().find(|(_, s)| s.accounts.contains_key(microbe::ACCOUNT)).unwrap().0;
    // Three more matter for the host, from the same soil.
    let ledger = &mut genesis.sites.get_mut(&site).unwrap().accounts;
    *ledger.get_mut("world:soil").unwrap() -= 3;
    *ledger.get_mut(microbe::ACCOUNT).unwrap() += 3;
    let (host, home) = genesis.population.groups.iter()
        .find_map(|(id, g)| (g.entity.alive && g.count > 0 && g.entity.resident_site() == Some(site)).then_some((*id, site)))
        .expect("a living body at the seed's site");
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    for command in microbe::seeding(site) { session.command(command).unwrap(); }
    let identity = Identity::Surface(microbe::KEY.into());
    session.command(cosmograph::Command::Spread(cosmograph::spread::Command::Place(cosmograph::spread::Placement {
        source: cosmograph::spread::Supply::Matter { from: Substrate::Site(site), account: microbe::ACCOUNT.into() },
        identity, at: Substrate::Host(host), amount: 3 }))).unwrap();
    (session, site, home)
}

fn key() -> Vec<Key> { vec![microbe::KEY.to_string()] }

#[test]
fn a_seeded_surface_marks_the_site_it_lies_on_and_no_other() {
    let (session, site) = seeded();
    let holds = site_holds(&session.sim, &key());
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[&site], SiteHold { land: microbe::SEED_AMOUNT, hosts: 0 });
    assert!(site_holds(&session.sim, &[]).is_empty(), "no held surface, no mark");
}

#[test]
fn the_marks_add_up_to_what_the_held_reading_counts_as_it_spreads() {
    let (mut session, _) = seeded();
    for _ in 0..80 { session.advance(1).unwrap(); }
    let holds = site_holds(&session.sim, &key());
    let held = readings::held(&session.sim, microbe::KEY).unwrap();
    assert_eq!(holds.values().map(|h| h.land).sum::<u64>(), held.sites.total);
    // Each body of a cohort is a host, the sum the reading counts.
    assert_eq!(holds.values().map(|h| h.hosts).sum::<u64>(), held.hosts.count);
    let identity = Identity::Surface(microbe::KEY.into());
    for (site, hold) in &holds {
        let on = cosmograph::spread::held(session.sim.state(), Substrate::Site(*site), &identity).unwrap();
        assert_eq!(on, hold.land);
    }
}

#[test]
fn a_body_carrying_the_matter_is_told_from_one_that_is_not() {
    let (mut session, _) = seeded();
    for _ in 0..80 { session.advance(1).unwrap(); }
    let pop = &session.sim.state().population;
    let carriers = pop.groups.values().filter(|g| carries_held(&g.entity, &key())).count();
    let held = readings::held(&session.sim, microbe::KEY).unwrap();
    assert_eq!(carriers > 0, held.hosts.count > 0);
    assert!(pop.groups.values().any(|g| !carries_held(&g.entity, &key())), "not every body carries it");
    assert!(!pop.groups.values().any(|g| carries_held(&g.entity, &[])));
}

#[test]
fn only_a_held_surface_that_has_not_ended_is_marked() {
    let (mut session, _) = seeded();
    session.command(cosmograph::Command::Join).unwrap();
    let me = *session.sim.state().population.groups.iter()
        .find(|(_, g)| cosmograph::directing::is_participant(&g.entity)).unwrap().0;
    assert!(held_surfaces(&session.sim, me).is_empty(), "nothing held yet");
    session.command(cosmograph::Command::Hold { participant: me, lineage: microbe::KEY.into() }).unwrap();
    assert_eq!(held_surfaces(&session.sim, me), key());
}

#[test]
fn strength_runs_from_faint_to_full_and_is_none_for_nothing() {
    assert_eq!(mark_strength(0, 10), 0.0);
    assert_eq!(mark_strength(5, 0), 0.0);
    assert!(mark_strength(1, 100) > 0.3 && mark_strength(1, 100) < mark_strength(50, 100));
    assert_eq!(mark_strength(100, 100), 1.0);
    assert_eq!(mark_strength(500, 100), 1.0, "past the peak is still full");
}

#[test]
fn the_legend_names_the_held_surface_and_both_marks_in_words() {
    let kingdom = |_: &str| "kingdom:micro".to_string();
    let words = hold_legend_words(&key(), &kingdom).unwrap();
    assert!(words.contains("microbe micro") && words.contains("ring") && words.contains("diamond"), "{words}");
    assert!(!words.to_lowercase().contains("flora"));
    assert_eq!(hold_legend_words(&[], &kingdom), None);
}

#[test]
fn banked_seeds_are_counted_in_words_only_when_there_are_some() {
    let (session, _) = seeded();
    assert_eq!(banked_words(&session.sim, None), None, "a world with no dormancy banks none");
    assert_eq!(banked_words(&session.sim, Some("lineage:1")), None);
}

#[test]
fn the_minimap_marks_the_held_site_with_no_body_and_no_player_mark() {
    let (session, site) = seeded();
    let sim = &session.sim;
    let marks = crate::minimap::hold_marks(sim, &key());
    assert_eq!(marks.len(), 1);
    let at = crate::minimap::site_points(sim)[&site];
    assert_eq!((marks[0].at, marks[0].land, marks[0].hosts), ((at.x, at.y), 1.0, 0.0));
    // A holder with no body: no played critter, and the map is still built, with the mark.
    let leaf = crate::minimap::minimap_leaf_holding(sim, None, &key());
    assert_eq!(leaf.holds(), marks.as_slice());
    assert!(crate::minimap::minimap_leaf(sim, None).holds().is_empty(), "nothing held, nothing marked");
}

#[test]
fn the_vitals_panel_says_seeds_and_the_map_key_only_when_it_has_them() {
    let (session, _) = seeded();
    let mut vitals = crate::vitals_of(&session.sim, None, 30, None, None);
    assert!(!crate::vital_rows(&vitals).iter().any(|(k, _)| *k == "seeds" || *k == "map"));
    vitals.banked = Some("3 seeds banked".into());
    vitals.map_key = Some("ring: where microbe micro lies".into());
    let rows = crate::vital_rows(&vitals);
    assert!(rows.contains(&("seeds", "3 seeds banked".to_string())));
    assert!(rows.iter().any(|(k, v)| *k == "map" && v.starts_with("ring")));
}

#[test]
fn a_site_with_hosts_carrying_it_counts_them_and_matches_the_reading() {
    let (session, site, home) = seeded_with_a_host();
    let holds = site_holds(&session.sim, &key());
    let held = readings::held(&session.sim, microbe::KEY).unwrap();
    assert_eq!(held.hosts.total, 3);
    assert_eq!(holds[&site].land, microbe::SEED_AMOUNT);
    assert_eq!(holds[&home].hosts, held.hosts.count);
    assert!(holds.values().map(|h| h.hosts).sum::<u64>() == held.hosts.count);
    let marks = crate::minimap::hold_marks(&session.sim, &key());
    assert!(marks.iter().any(|m| m.hosts > 0.0), "a diamond is drawn for the carrying bodies");
}

#[test]
fn a_dormant_world_banks_seeds_and_says_how_many() {
    let mut founding = mesocosm_runtime::Founded::generated(9, 256).unwrap().genesis.founding.unwrap();
    founding.ecology_rates = cosmograph::ecology::Rates::A1;
    founding.dormancy = Some(cosmograph::rules::Dormancy::flat());
    let mut session = Session::new(founding.generate().unwrap(), Execution::Individuals).unwrap();
    let mut words = None;
    for _ in 0..120 {
        session.advance(1).unwrap();
        words = banked_words(&session.sim, None);
        if words.is_some() { break; }
    }
    let words = words.expect("a producer's young lie dormant within 120 ticks");
    let all: u64 = readings::propagules(&session.sim).values().map(|p| p.count).sum();
    assert_eq!(words, format!("{all} {} banked", if all == 1 { "seed" } else { "seeds" }));
    let (lineage, own) = readings::propagules(&session.sim).into_iter().next().map(|(k, p)| (k, p.count)).unwrap();
    assert_eq!(banked_words(&session.sim, Some(&lineage)), Some(format!("{own} {} banked", if own == 1 { "seed" } else { "seeds" })));
    assert_eq!(banked_words(&session.sim, Some("lineage:none")), None);
}
