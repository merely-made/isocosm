// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Each polity owner grants work under the strategy its lots author, never one inferred (844).
use super::*;
use isocosm::projects::{Consented, DecisionPolicy};

fn consented(result: isocosm::Result<String>) -> Consented { serde_json::from_str(&result.unwrap()).unwrap() }

#[test]
fn unanimous_members_must_each_consent_before_the_polity_grants() {
    let w = collective_world(171);
    let [a, b, c] = w.bodies;
    let mut o = owned(&w, DecisionPolicy::UnanimousMembers);
    assert_eq!(o.s.sim.state().projects.get(o.project).unwrap().owners, BTreeSet::from([Owner::Polity(o.polity)]));
    refused(&mut o.s, polity_consent(o.project, o.polity, o.p[2], c, a, None));
    assert!(!consented(o.s.command(polity_consent(o.project, o.polity, o.p[0], a, a, None))).authorized, "one member is not unanimity");
    refused(&mut o.s, act(o.p[0], a, WORK, Some(o.project), &[]));
    assert!(consented(o.s.command(polity_consent(o.project, o.polity, o.p[1], b, a, None))).authorized);
    acted(o.s.command(act(o.p[0], a, WORK, Some(o.project), &[])));
    exact(&o.s, o.mass);
}

#[test]
fn declared_representatives_grant_alone_and_others_cannot() {
    let w = collective_world(172);
    let [a, b, _] = w.bodies;
    let mut o = owned(&w, DecisionPolicy::Representatives { actors: BTreeSet::from([a]), approvals: 1 });
    refused(&mut o.s, polity_consent(o.project, o.polity, o.p[1], b, a, None));
    assert!(consented(o.s.command(polity_consent(o.project, o.polity, o.p[0], a, a, None))).authorized);
    exact(&o.s, o.mass);
}

#[test]
fn a_governance_decision_grants_through_its_authored_process_only() {
    let w = collective_world(173);
    let [a, b, _] = w.bodies;
    let mut o = owned(&w, DecisionPolicy::Governance { process: PROJECT_DECIDE.into() });
    refused(&mut o.s, polity_consent(o.project, o.polity, o.p[0], a, b, None));
    refused(&mut o.s, polity_consent(o.project, o.polity, o.p[0], a, b, Some(FEAST)));
    let hash = o.s.sim.state_hash();
    let granted = consented(o.s.command(polity_consent(o.project, o.polity, o.p[0], a, b, Some(PROJECT_DECIDE))));
    assert!(granted.authorized, "one admitted governance act by a member decides for the polity");
    assert_ne!(o.s.sim.state_hash(), hash);
    let proof = permission(&o, b).unwrap().proof;
    assert_eq!(proof.len(), 1);
    assert!(proof[0].action.is_some(), "the decision names its actual admitted native act");
    assert!(authorized(&o, b) && !authorized(&o, a), "the grant is for the named worker only");
    exact(&o.s, o.mass);
}

#[test]
fn a_member_who_is_no_longer_available_blocks_unanimity() {
    let w = collective_world(174);
    let [a, b, _] = w.bodies;
    let mut o = owned(&w, DecisionPolicy::UnanimousMembers);
    o.s.command(Command::Kill { entity: b }).unwrap();
    refused(&mut o.s, polity_consent(o.project, o.polity, o.p[0], a, a, None));
}
