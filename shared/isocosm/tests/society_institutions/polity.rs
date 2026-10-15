// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A polity reading keeps its authored attributes and member-derived facts apart (867).
use super::*;
use isocosm::asserted::{Authored, Faction};

#[test]
fn a_founded_polity_reads_its_actual_members_and_their_acts() {
    let w = charter_world(141);
    let [a, b, c] = w.bodies;
    let (s, _, id) = founded(&w);
    let reading = s.sim.polity_reading(id.0).unwrap();
    assert_eq!(reading.members.iter().map(|m| (m.id, m.constitutional, m.relation)).collect::<Vec<_>>(),
        [(a, true, false), (b, true, false)]);
    assert!(reading.members.iter().all(|m| m.attributes.as_ref().is_some_and(|x| x.alive)));
    assert!(reading.members[0].acts.iter().any(|event| event.process == FOUND), "the founder's own act is read");
    assert!(reading.authored.is_none() && reading.disagreement.is_none() && reading.ended_by.is_none());
    assert!(reading.members.iter().all(|m| m.id != c));
    assert!(s.sim.polity_reading(c).is_err(), "only a native polity has a reading");
}

#[test]
fn an_authored_polity_without_members_claims_no_derived_disagreement() {
    let w = charter_world(142);
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let authored = Authored { key: "faction:hearth".into(), name: "Hearth".into(),
        tags: BTreeSet::from(["tag:kin".to_string()]), claims: BTreeSet::new() };
    s.command(Command::Assert(Assertion::Faction(Faction { authored: authored.clone(), governance: CHARTER.into(),
        focus: BTreeSet::new() }))).unwrap();
    let (id, _) = s.sim.authored_polity("faction:hearth").unwrap();
    let reading = s.sim.polity_reading(id).unwrap();
    assert_eq!(reading.authored, Some(authored));
    assert!(reading.members.is_empty() && reading.member_tags.is_empty());
    assert!(reading.disagreement.is_none(), "no members means no derived tag claim");
    exact(&s, s.sim.matter());
}
