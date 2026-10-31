// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "attention/fixture.rs"]
mod fixture;
use cosmograph::{Session, Execution, attention::{Region, WindowAttention}, directing::readings::{self, Mode}};
use std::collections::BTreeMap;

#[test]
fn local_close_up_cannot_borrow_another_windows_region_or_survival_awareness() {
    let (mut session, participants, subjects) = fixture::fixture();
    let site = session.sim.state().population.get(subjects[0]).unwrap().place;
    let hidden = *readings::view(&session.sim, subjects[0], Mode::Creative).sites
        .difference(&readings::view(&session.sim, subjects[0], Mode::Survival).sites)
        .next().expect("declared fixture supplies a site outside native Survival awareness");
    let first = session.attach_window(participants[0], "mesocosm".into(), WindowAttention::default()).unwrap();
    let second = session.attach_window(participants[0], "vtt".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    let other = session.attach_window(participants[0], "eponym".into(), WindowAttention {
        examined: Some(Region::Site(hidden)), ..Default::default() }).unwrap();
    assert!(session.sim.player_attention(participants[0]).unwrap().examined.contains(&Region::Site(site)));
    assert!(session.sim.region_reading(participants[0], first.id(), Mode::Creative, site).is_err(),
        "another active window cannot lend its local close-up region");
    let region = session.sim.region_reading(participants[0], second.id(), Mode::Survival, site).unwrap();
    assert_eq!(region.played, Some(subjects[0]));
    assert!(region.bodies.iter().any(|b| b.id == subjects[0]));
    assert!(session.sim.region_reading(participants[0], other.id(), Mode::Survival, hidden).is_err(),
        "a watched site cannot grant Survival awareness");
    assert!(session.sim.region_reading(participants[0], other.id(), Mode::Creative, hidden).is_ok());
    let before = session.save();
    assert!(session.sim.region_reading(participants[1], second.id(), Mode::Creative, site).is_err());
    assert_eq!(session.save(), before, "foreign reads do not mutate the actual native Session");
}

#[test]
fn far_crowds_stay_aggregate_and_reads_preserve_native_history_and_replay() {
    let (mut session, participants, subjects) = fixture::fixture();
    let site = session.sim.state().population.get(subjects[0]).unwrap().place;
    let first = session.attach_window(participants[0], "mesocosm".into(), WindowAttention::default()).unwrap();
    let before = session.sim.far_crowds(participants[0], first.id(), Mode::Creative).unwrap();
    let second = session.attach_window(participants[0], "vtt".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    assert_eq!(session.sim.far_crowds(participants[0], first.id(), Mode::Creative).unwrap(), before,
        "far maps remain crowd readings after another window materializes a close-up");
    let mut expected = BTreeMap::new();
    for (first, count) in readings::view(&session.sim, subjects[0], Mode::Creative).things {
        let e = session.sim.state().population.get(first).unwrap();
        *expected.entry((e.place, e.lineage.clone())).or_insert(0u64) += count;
    }
    assert_eq!(before.iter().map(|c| ((c.site, c.lineage.clone()), c.count)).collect::<BTreeMap<_,_>>(), expected);
    let saved = session.save();
    let visible = session.sim.region_reading(participants[0], second.id(), Mode::Survival, site).unwrap();
    let inspected = session.sim.inspected_body(participants[0], second.id(), Mode::Survival, site, subjects[0]).unwrap();
    assert_eq!(inspected.body.id, subjects[0]);
    let parts = &session.sim.state().population.get(subjects[0]).unwrap().parts;
    assert_eq!(inspected.parts, parts.iter().map(|(id, p)| (*id, (**p).clone())).collect::<BTreeMap<_, _>>());
    assert_eq!(session.save(), saved, "camera, far map and examiner reads never write replay history");
    for execution in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(saved.clone(), execution).unwrap();
        assert_eq!(replay.save(), saved);
        assert_eq!(replay.sim.region_reading(participants[0], second.id(), Mode::Survival, site).unwrap(), visible);
        assert_eq!(replay.sim.far_crowds(participants[0], first.id(), Mode::Creative).unwrap(), before);
    }
}

#[test]
fn closing_withdraws_only_its_own_region_and_stale_examiner_refuses() {
    let (mut session, participants, subjects) = fixture::fixture();
    let site = session.sim.state().population.get(subjects[0]).unwrap().place;
    let first = session.attach_window(participants[0], "mesocosm".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    let second = session.attach_window(participants[0], "eponym".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    session.close_attention(participants[0], first).unwrap();
    assert!(session.sim.player_attention(participants[0]).unwrap().examined.contains(&Region::Site(site)));
    let saved = session.save();
    assert!(session.sim.region_reading(participants[0], first.id(), Mode::Creative, site).is_err());
    assert!(session.sim.inspected_body(participants[0], first.id(), Mode::Creative, site, subjects[0]).is_err());
    assert!(session.sim.inspected_body(participants[0], second.id(), Mode::Survival, site, subjects[0]).is_ok());
    assert_eq!(session.save(), saved);
    session.replace_attention(participants[0], second, "eponym".into(), WindowAttention::default()).unwrap();
    let saved = session.save();
    assert!(session.sim.inspected_body(participants[0], second.id(), Mode::Creative, site, subjects[0]).is_err(),
        "an examiner loses detail as soon as its own close-up is withdrawn");
    assert_eq!(session.save(), saved);
}
