// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::{Command, Execution, Founding, attention::{Region, WindowAttention}, directing::Played, schema::Method};
use isometer::{FrameRequest, SceneProducer};

pub(super) fn fixture() -> (Session, Id, Id, u64, Id) {
    // A played world is mapped (783): the close-up lifts its site's terrain.
    let grid = isocosm::map::Grid { width: 2, height: 2, ..isocosm::map::Grid::drawn(11) };
    let genesis = Founding { seed: 11, sites: 4, map: Some(isocosm::map::Layout::Grid(grid)),
        population: 24, cohort_size: 1, lineages: 3,
        ecology: true, bodies: Some(Default::default()), played: Some(Played { lineage: 1, region_sites: 1 }),
        ..Default::default() }.generate().unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let subject = *session.sim.state().population.groups.iter()
        .find(|(_, group)| group.entity.method == Method::Deliberative).unwrap().0;
    let participant = session.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    session.command(Command::Take { participant, critter: subject }).unwrap();
    let site = session.sim.state().population.get(subject).unwrap().place;
    let window = session.attach_window(participant, "mesocosm".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    (session, participant, subject, window.id(), site)
}

#[test]
fn native_snapshot_is_exactly_the_owned_visible_region_and_has_no_first_site_fallback() {
    let (mut session, participant, subject, window, site) = fixture();
    let handle = SceneHandle::default();
    let before = session.save();
    handle.update(&session, participant, window, Mode::Survival, site, 0).unwrap();
    let snapshot = handle.snapshot().unwrap();
    let visible = session.sim.region_reading(participant, window, Mode::Survival, site).unwrap();
    assert_eq!(snapshot.body_ids(), visible.bodies.iter().map(|b| b.id).collect::<Vec<_>>(),
        "Section contains exactly the native visible bodies");
    assert_eq!(snapshot.played(), Some(subject));
    assert_eq!(snapshot.site(), site);
    assert_eq!(session.save(), before, "picture preparation cannot emit native commands");
    let foreign: Id = session.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    let before = session.save();
    assert!(handle.update(&session, foreign, window, Mode::Creative, site, 0).is_err());
    assert!(handle.snapshot().is_none(), "foreign or stale native source withdraws the old picture input");
    let other = *session.sim.state().sites.keys().find(|id| **id != site).unwrap();
    assert!(handle.update(&session, participant, window, Mode::Creative, other, 0).is_err());
    assert_eq!(session.save(), before);
}

#[test]
fn two_native_windows_keep_independent_cameras_and_close_withdraws_only_its_input() {
    let (mut session, participant, _, window, site) = fixture();
    let second = session.attach_window(participant, "mesocosm".into(), WindowAttention {
        examined: Some(Region::Site(site)), ..Default::default() }).unwrap();
    let handles = [SceneHandle::default(), SceneHandle::default()];
    let before = session.save();
    handles[0].update(&session, participant, window, Mode::Survival, site, 1).unwrap();
    handles[1].update(&session, participant, second.id(), Mode::Survival, site, 2).unwrap();
    assert_ne!(handles[0].snapshot().unwrap().camera(1.0), handles[1].snapshot().unwrap().camera(1.0));
    assert_eq!(session.save(), before, "independent cameras never change native history");
    let native_window = session.sim.attention_window(participant, window).unwrap().0;
    session.close_attention(participant, native_window).unwrap();
    assert!(handles[0].update(&session, participant, window, Mode::Survival, site, 1).is_err());
    assert!(handles[0].snapshot().is_none());
    assert!(handles[1].snapshot().is_some());
    assert!(session.sim.player_attention(participant).unwrap().examined.contains(&Region::Site(site)));
}

#[test]
fn unchanged_native_snapshot_skips_lowering_and_invalid_camera_refuses_atomically() {
    let (session, participant, _, window, site) = fixture();
    let handle = SceneHandle::default();
    assert!(handle.update(&session, participant, window, Mode::Survival, site, 0).unwrap());
    let first = handle.snapshot().unwrap();
    assert!(!handle.update(&session, participant, window, Mode::Survival, site, 0).unwrap());
    assert!(Rc::ptr_eq(&first, &handle.snapshot().unwrap()));
    assert!(handle.update(&session, participant, window, Mode::Survival, site, 1).unwrap());
    assert_ne!(first.camera(1.0).forward, handle.snapshot().unwrap().camera(1.0).forward);
    assert!(handle.update(&session, participant, window, Mode::Survival, site, 4).is_err());
    assert!(handle.snapshot().is_none());
}

#[test]
fn survival_section_does_not_gain_awareness_from_a_watch() {
    let (mut session, participant, subject, _, _) = fixture();
    let visible = isocosm::directing::readings::view(&session.sim, subject, Mode::Survival);
    let hidden = *session.sim.state().sites.keys().find(|id| !visible.sites.contains(id)).unwrap();
    let window = session.attach_window(participant, "mesocosm".into(), WindowAttention {
        examined: Some(Region::Site(hidden)), ..Default::default() }).unwrap();
    let before = session.save();
    let handle = SceneHandle::default();
    assert!(handle.update(&session, participant, window.id(), Mode::Survival, hidden, 0).is_err(),
        "a watch cannot widen Survival awareness for the Section");
    assert!(handle.snapshot().is_none());
    assert!(handle.update(&session, participant, window.id(), Mode::Creative, hidden, 0).is_ok());
    assert_eq!(session.save(), before);
}

#[test]
#[ignore = "requires the independently owned actual GPU gate"]
fn actual_section_uses_present_device_composes_lit_bodies_and_obeys_scene_skip() {
    let (session, participant, _, window, site) = fixture();
    let handle = SceneHandle::default();
    handle.update(&session, participant, window, Mode::Survival, site, 0).unwrap();
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions::default())
        .expect("GPU refusal is not a pass for this gate");
    let host = section::host_device::from_renderer(core.renderer());
    assert_eq!(&host.device, core.device()); assert_eq!(&host.queue, core.queue());
    let mut producer = SceneProducer::new(producer::Source::new(host, handle.clone()));
    let request = FrameRequest { device: core.device(), queue: core.queue(), size: [128,96],
        aspect: 128.0 / 96.0, color: None, needs_frame: true, render_scale: 1 };
    assert!(producer.render_scene(&request).unwrap().is_some());
    let (width,height,pixels) = producer.source().capture().expect("actual composed output readback");
    assert_eq!((width,height), (128,96));
    assert!(pixels.chunks_exact(4).all(|p| p[3] == 255), "opaque composite meets straight-alpha source contract");
    assert!(pixels.chunks_exact(4).any(|p| p[..3] != [0,0,0]),
        "actual section viewport paints terrain and lit body pixels");
    let stats = producer.source().stats();
    assert!(stats.controlled_drawn, "actual played body reaches the lit tenant");
    assert_eq!(stats.body_tenant_internal_submissions, 0, "tenant uses the host's encoder");
    assert!(producer.render_scene(&FrameRequest { needs_frame: false, ..request }).unwrap().is_none());
    assert_eq!(producer.renders(), 1, "unchanged actual GPU inputs do not advance generation");
    handle.update(&session, participant, window, Mode::Survival, site, 1).unwrap();
    assert!(producer.render_scene(&FrameRequest { needs_frame: false, ..request }).unwrap().is_some());
    assert_eq!(producer.renders(), 2);
    assert_eq!(producer.presented_camera(), Some(handle.snapshot().unwrap().camera(request.aspect)));
}
