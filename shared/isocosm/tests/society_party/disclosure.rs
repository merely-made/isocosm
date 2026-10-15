// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Disclosed map facts are recorded, bounded by native knowledge, and outlive membership (854, 858).
use super::*;
use isocosm::party::{self, Outcome};

#[test]
fn a_disclosure_is_bounded_by_the_bodys_native_knowledge() {
    let w = party_world(111);
    let [a, b, _] = w.bodies;
    let (mut s, p) = table(&w);
    refused(&mut s, disclose(p[0], a, &[w.home]));
    outcome(s.command(join(p[0], a)));
    refused(&mut s, disclose(p[1], a, &[w.home]));
    refused(&mut s, disclose(p[0], a, &[w.home, w.away]));
    refused(&mut s, party(party::Change::Disclose { participant: p[0], party: BAND.into(), member: a,
        sites: BTreeSet::from([w.home]), events: BTreeSet::from(["event:unknown".into()]) }));
    assert!(party::knowledge(&s.sim, a).unwrap().sites.contains(&w.home));
    let Outcome::Disclosed(record) = outcome(s.command(disclose(p[0], a, &[w.home]))) else { panic!("accepted") };
    assert_eq!(record.places.iter().map(|place| place.site).collect::<Vec<_>>(), [w.home]);
    assert!(record.routes.is_empty(), "a route needs both ends disclosed");
    let map = party::shared_map(&s, BAND).unwrap();
    assert_eq!(map.places.keys().copied().collect::<Vec<_>>(), [w.home]);
    // The shared view is a read of records: no member gains native knowledge from it.
    outcome(s.command(join(p[1], b)));
    assert!(!party::knowledge(&s.sim, b).unwrap().sites.contains(&w.away));
    exact(&s, s.sim.matter());
}

#[test]
fn disclosed_places_and_routes_persist_after_the_member_leaves() {
    let w = party_world(112);
    let [a, _, _] = w.bodies;
    let (mut s, p) = table(&w);
    outcome(s.command(join(p[0], a)));
    s.advance(1).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&s.command(
        Command::Act { actor: a, target: None, process: GO.into(), cause: None }).unwrap()).unwrap();
    assert!(receipt.accepted());
    while s.sim.state().population.get(a).unwrap().resident_site() != Some(w.away) { s.advance(1).unwrap(); }
    // Having travelled, the body natively knows both ends and the route between them.
    let Outcome::Disclosed(record) = outcome(s.command(disclose(p[0], a, &[w.home, w.away]))) else { panic!("accepted") };
    assert_eq!(record.routes.iter().map(|r| (r.from, r.to, r.travel)).collect::<Vec<_>>(), [(w.home, w.away, 1)]);
    outcome(s.command(leave(p[0], a)));
    let map = party::shared_map(&s, BAND).unwrap();
    assert!(map.members.is_empty());
    assert_eq!(map.places.keys().copied().collect::<BTreeSet<_>>(), BTreeSet::from([w.home, w.away]));
    assert_eq!(map.routes.len(), 1);
    assert_eq!(map.records.len(), 1);
    refused(&mut s, disclose(p[0], a, &[w.home]));
    // A save replayed from genesis rebuilds the same shared record.
    let json = serde_json::to_vec(&s.save()).unwrap();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let restored = Session::load_json(&json, execution).unwrap();
        assert_eq!(restored.sim.state_hash(), s.sim.state_hash());
        assert_eq!(party::shared_map(&restored, BAND).unwrap(), map);
    }
}
