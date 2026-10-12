// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Command, Execution, Founding, Session, map::{Grid, Layout}};

fn session(mode: Execution) -> Session {
    let profile = Founding { seed: 83, sites: 4, population: 8, lineages: 2, cohort_size: 2,
        map: Some(Layout::Grid(Grid { shape: "shape:plane".into(), width: 2, height: 2,
            side: 256, elevation: [0, 0], relief: [0, 0], sea_per_mille: 0 })),
        ..Default::default() };
    Session::new(profile.generate().unwrap(), mode).unwrap()
}

fn place(key: &str) -> Place {
    Place { key: key.into(), name: "Attached place".into(), position: Some((-900, 701)),
        map: Some("inert-board".into()), ..Default::default() }
}

fn geometry(s: &Session) -> Site {
    let mut site = s.sim.state().sites[&0].clone();
    site.accounts.clear(); site.routes.clear(); site.authored = None;
    site.terrain_seed = 991;
    site
}

#[test]
fn existing_attachment_preserves_geometry_population_and_native_replay() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = session(mode);
        let before = s.sim.state().sites[&0].clone();
        let population = s.sim.state().population.clone();
        s.command(Command::AttachSite { place: place("ford"), attachment: Attachment::Existing(0) }).unwrap();
        let mut expected = before; expected.authored = Some(place("ford"));
        assert_eq!(s.sim.state().sites[&0], expected, "attachment replaced existing native geometry");
        assert_eq!(s.sim.state().population, population, "attachment moved or allocated a body");
        let saved = s.save();
        for replay in [Execution::Individuals, Execution::Grouped] {
            assert_eq!(Session::load(saved.clone(), replay).unwrap().save(), saved,
                "existing native attachment changed on save/load replay");
        }
    }
}

#[test]
fn supplied_geometry_is_native_and_never_converted_from_the_table_cell() {
    let mut s = session(Execution::Grouped);
    let geometry = geometry(&s);
    s.command(Command::AttachSite { place: place("shrine"),
        attachment: Attachment::New { site: 1000, geometry: geometry.clone() } }).unwrap();
    let mut expected = geometry; expected.authored = Some(place("shrine"));
    assert_eq!(s.sim.state().sites[&1000], expected, "supplied native geometry was not admitted");
    assert_eq!(crate::map::position(s.sim.genesis(), 1000), None,
        "table cell was promoted into native map geometry");
    assert!(s.sim.state().population.next_id > 1000);
    assert!(s.assisted());
    let view = crate::terrain::View::over(s.sim.genesis(), &s.sim.state().sites).unwrap();
    use isometer_space::Atlas;
    assert!(view.lattice(1000).is_ok(), "native attachment cannot be lifted");
    let saved = s.save();
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(Session::load(saved.clone(), mode).unwrap().save(), saved);
    }
}

#[test]
fn invalid_geometry_topology_and_aliases_refuse_before_native_publication() {
    let mut s = session(Execution::Grouped);
    s.command(Command::AttachSite { place: place("ford"), attachment: Attachment::Existing(0) }).unwrap();
    let before = s.save();
    let mut negative = geometry(&s);
    negative.conditions.insert("terrain:relief".into(), -1);
    let mut missing = geometry(&s); missing.conditions.remove("terrain:elevation");
    let mut broken = geometry(&s);
    broken.routes = s.sim.state().sites[&0].routes.clone();
    for (place, attachment, reason) in [
        (place("shrine"), Attachment::New { site: 1000, geometry: negative }, "negative relief"),
        (place("shrine"), Attachment::New { site: 1000, geometry: missing }, "without its skeleton"),
        (place("shrine"), Attachment::New { site: 1000, geometry: broken }, "without its reverse"),
        (place("ford"), Attachment::Existing(1), "ambiguous authored place attachment"),
        (place("absent"), Attachment::Existing(9999), "unknown native site"),
    ] {
        let error = s.command(Command::AttachSite { place, attachment })
            .expect_err("invalid attachment was admitted");
        assert!(error.contains(reason), "wrong attachment refusal: {error}");
        assert_eq!(s.save(), before, "attachment refusal mutated native history or state");
    }
}

#[test]
fn conflicting_site_or_body_identity_and_injected_stocks_are_refused() {
    let mut s = session(Execution::Individuals);
    let before = s.save();
    assert!(s.sim.state().population.get(5).is_some(), "body collision control has no generated member");
    assert!(!s.sim.state().sites.contains_key(&5), "body collision control overlaps an existing site");
    let mut stocks = geometry(&s); stocks.accounts.insert("injected".into(), 1);
    for (site, geometry, reason) in [
        (0, geometry(&s), "already occupied"),
        (5, geometry(&s), "names another native noun"),
        (1000, stocks, "authored identity or stocks"),
    ] {
        let error = s.command(Command::AttachSite { place: place("extra"),
            attachment: Attachment::New { site, geometry } })
            .expect_err("conflicting native site or body identity was accepted");
        assert!(error.contains(reason)); assert_eq!(s.save(), before);
    }
    s.command(Command::AttachSite { place: place("first"), attachment: Attachment::Existing(0) }).unwrap();
    let attached = s.save();
    let error = s.command(Command::AttachSite { place: place("second"), attachment: Attachment::Existing(0) })
        .expect_err("conflicting authored identity was accepted");
    assert!(error.contains("another authored place")); assert_eq!(s.save(), attached);
}
