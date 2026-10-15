// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A carried item follows its carrier's one journey and conserves its matter.
use super::*;
use isocosm::custody::Action;

#[test]
fn a_carried_item_rides_the_carriers_journey_and_rests_where_dropped() {
    let w = custody_world(61);
    let [a, _, _] = w.bodies;
    let (mut s, players) = session(&w);
    let item = arrive_item(&mut s, &w, w.home, ITEM_MG);
    let mass = s.sim.matter();
    let hash = s.sim.state_hash();
    let took = custody_acted(s.command(custody(change(players[0], a, item, Action::Carry))));
    assert!(took.carried && took.owned.receipt.accepted());
    assert_eq!((took.matter, took.burden, took.capacity), (u128::from(ITEM_MG), u128::from(ITEM_MG), CAPACITY as u64));
    assert_ne!(s.sim.state_hash(), hash);
    assert_eq!(s.sim.item_carrier(item).unwrap(), Some(a));
    exact(&s, mass);

    depart(&mut s, a);
    let (body, held) = (s.sim.state().population.get(a).unwrap(), s.sim.state().population.get(item).unwrap());
    assert!(body.journey.is_some(), "the carrier is in transit");
    assert_eq!((held.journey.clone(), held.place), (body.journey.clone(), body.place), "one journey, never a second clock");
    assert_eq!(held.resident_site(), None);
    exact(&s, mass);
    arrive(&mut s, a, w.away);
    assert_eq!(s.sim.state().population.get(item).unwrap().resident_site(), Some(w.away));
    assert_eq!(s.sim.item_carrier(item).unwrap(), Some(a));
    exact(&s, mass);

    let dropped = custody_acted(s.command(custody(change(players[0], a, item, Action::Drop))));
    assert!(!dropped.carried);
    assert_eq!((dropped.burden, s.sim.item_carrier(item).unwrap()), (0, None));
    assert_eq!(s.sim.state().population.get(item).unwrap().resident_site(), Some(w.away));
    exact(&s, mass);
}

#[test]
fn equip_and_unequip_keep_the_exclusive_custody() {
    let w = custody_world(62);
    let [a, _, _] = w.bodies;
    let (mut s, players) = session(&w);
    let item = arrive_item(&mut s, &w, w.home, ITEM_MG);
    let mass = s.sim.matter();
    custody_acted(s.command(custody(change(players[0], a, item, Action::Carry))));
    // Wearing spends the actor's own timber, so it needs no title and no selection here.
    let worn = custody_acted(s.command(custody(change(players[0], a, item, Action::Equip))));
    assert!(worn.carried);
    assert_eq!(s.sim.item_carrier(item).unwrap(), Some(a));
    let doffed = custody_acted(s.command(custody(change(players[0], a, item, Action::Unequip))));
    assert!(doffed.carried, "unequipping still holds the item");
    exact(&s, mass);
}
