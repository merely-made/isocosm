// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tint a body takes for carrying a held surface's matter.

use super::*;
use cosmograph::{Command, Execution, Session,
    spread::{self, Identity, Placement, Substrate, Supply}};
use mesocosm_runtime::microbe;

/// A hosting world whose microbe is seeded, with three of it also on the body
/// at the seed's site; returns the session, that body's id and the site.
fn world() -> (Session, Id, Id) {
    let mut genesis = mesocosm_runtime::Founded::generated(9, 256).unwrap().genesis;
    microbe::declare(&mut genesis).unwrap();
    let site = *genesis.sites.iter().find(|(_, s)| s.accounts.contains_key(microbe::ACCOUNT)).unwrap().0;
    let ledger = &mut genesis.sites.get_mut(&site).unwrap().accounts;
    *ledger.get_mut("world:soil").unwrap() -= 3;
    *ledger.get_mut(microbe::ACCOUNT).unwrap() += 3;
    let host = *genesis.population.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.resident_site() == Some(site) && g.entity.body.is_some())
        .expect("a body at the seed's site").0;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    for command in microbe::seeding(site) { session.command(command).unwrap(); }
    session.command(Command::Spread(spread::Command::Place(Placement {
        source: Supply::Matter { from: Substrate::Site(site), account: microbe::ACCOUNT.into() },
        identity: Identity::Surface(microbe::KEY.into()), at: Substrate::Host(host), amount: 3 }))).unwrap();
    (session, host, site)
}

#[test]
fn a_carrier_is_tinted_toward_the_badge_colour_and_the_rest_are_left_alone() {
    let (session, host, site) = world();
    let sim = &session.sim;
    let volume = lift(sim, site).unwrap();
    let mut bodies = placed(sim, site, &volume);
    assert!(bodies.iter().any(|b| b.id == host), "the host is drawn");
    let before: Vec<_> = bodies.iter().map(|b| (b.id, b.tint)).collect();
    badge_carriers(&mut bodies, sim, &[]);
    assert!(bodies.iter().map(|b| (b.id, b.tint)).eq(before.iter().copied()), "nothing held, nothing badged");
    badge_carriers(&mut bodies, sim, &[microbe::KEY.to_string()]);
    for (body, (_, tint)) in bodies.iter().zip(&before) {
        if body.id == host { assert_eq!(body.tint, carrier_tint(*tint)); assert_ne!(body.tint, *tint); }
        else { assert_eq!(body.tint, *tint, "body {} carries none", body.id); }
    }
}

#[test]
fn the_badge_colour_is_mostly_the_carrier_colour_whatever_the_body_was() {
    for tint in [[0.0; 3], [1.0; 3], [0.2, 0.7, 0.3]] {
        let badged = carrier_tint(tint);
        for (b, c) in badged.iter().zip(CARRIER) { assert!((b - c).abs() < 0.36, "{tint:?} -> {badged:?}"); }
    }
}
