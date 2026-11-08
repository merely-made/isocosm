// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{Command, Execution, Session, directing::readings::Mode, harm::Wounded};

#[test]
#[ignore = "actual-device hybrid section receipt"]
fn hybrid_section_replaces_the_native_wounded_anatomy() {
    let (mut session, participant, played, window, site) = fixture();
    let handle = SceneHandle::default();
    handle
        .update(&session, participant, window, Mode::Creative, site, 0)
        .unwrap();
    let native = handle.snapshot().unwrap();
    let (target, limb) = native
        .scene
        .bodies
        .iter()
        .find_map(|body| {
            (body.id != played && body.alive)
                .then(|| {
                    body.document
                        .living()
                        .find(|part| part.id != body.document.root)
                        .map(|part| (body.id, part.id))
                })
                .flatten()
        })
        .expect("generated fixture has an admitted peer with a limb");
    // This truth view places bodies without native poses: cohort stand-ins
    // that hold only a sprite (1171). Posing them lets examination promote.
    let isolate = |native: &crate::borrowed_section::Snapshot, posed: bool| {
        let mut scene = (*native.scene).clone();
        scene
            .bodies
            .retain(|body| body.id == target || body.id == played);
        for body in scene.bodies.iter_mut().filter(|_| posed) {
            body.physical
                .get_or_insert_with(|| cosmograph::spatial::Pose {
                    at: body.at.map(i64::from),
                    ..Default::default()
                });
        }
        Rc::new(scene)
    };
    let stand_ins = isolate(&native, false);
    let original = isolate(&native, true);
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions::default())
        .expect("a GPU is required for the hybrid receipt");
    let host = section::host_device::from_renderer(core.renderer());
    let source = Source::new(host, handle.clone());
    let mut producer = SceneProducer::new(source);
    let size = [192, 144];
    let camera = native.camera(size[0] as f32 / size[1] as f32);
    let target_body = stand_ins.body(target).unwrap();
    let bounds = target_body.document.aabb();
    let longest = bounds.extent().into_iter().max().unwrap() as f32;
    let centre = std::array::from_fn(|axis| {
        target_body.at[axis] as f32 + (bounds.min[axis] + bounds.max[axis]) as f32 * 0.5
    });
    let mut view = Presentation {
        mode: section::CameraMode::default(),
        depth: Some(512.0),
        forward: camera.forward,
        centre,
        // Fit the entire native bound well below the lowest refresh band.
        half_height: (longest * 16.0).max(100.0),
        bodies: BodyMode::Hybrid,
        budget: section::DEFAULT_BODY_BUDGET,
        light: 1.0,
        sky: section::SkyBlock::fixed(),
        terrain: section::TerrainStyle::default(),
        terrain_y: camera.centre[1],
        looking: section::perspective::Looking::overview(),
        focus: None,
        followed: None,
        light_step: section::DEFAULT_LIGHT_STEP,
        selection: None,
    };
    let request = FrameRequest {
        device: core.device(),
        queue: core.queue(),
        size,
        aspect: camera.aspect,
        color: None,
        needs_frame: false,
        render_scale: 1,
    };
    let before = session.save();
    // Stand-ins: the played body and a followed one stay sprites until M4.8.
    view.followed = Some(target);
    handle.present(&stand_ins, 0, view.clone()).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    let held = producer.source().section.as_ref().unwrap().hybrid_stats();
    assert_eq!((held.sprite_bodies, held.refreshes), (2, 2));
    view.followed = None;
    handle.present(&original, 0, view.clone()).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    let section = producer.source().section.as_ref().unwrap();
    eprintln!(
        "Mesocosm far viewport {:?}; prepared={:?}; target={target}; bounds={bounds:?}; camera={:?}",
        section.hybrid_stats(),
        section.body_stats(),
        producer.presented_camera()
    );
    // Posed: the played body is examined and live; the actor's image holds.
    assert_eq!(section.hybrid_stats().sprite_bodies, 1);
    assert_eq!(section.hybrid_stats().refreshes, 0);
    let (_, _, pixels) = producer.source().capture().unwrap();
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] != [0, 0, 0]));
    let selection = section
        .select_part(target, None, false)
        .expect("the native actor stays selectable in the far rung");
    view.selection = Some(selection);
    handle.present(&original, 0, view.clone()).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    assert_eq!(
        producer
            .source()
            .section
            .as_ref()
            .unwrap()
            .hybrid_stats()
            .refreshes,
        0
    );
    view.selection = None;
    handle.present(&original, 0, view.clone()).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    // Following the actor, without dev focus, examines it: Mesocosm's hook.
    view.followed = Some(target);
    handle.present(&original, 0, view.clone()).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    let followed = producer.source().section.as_ref().unwrap().hybrid_stats();
    assert_eq!(followed.sprite_bodies, 0, "the followed body stays live");
    view.followed = None;
    handle.present(&original, 0, view.clone()).unwrap();
    let animate = FrameRequest {
        needs_frame: true,
        ..request
    };
    for _ in 0..16 {
        producer.render_scene(&animate).unwrap();
    }
    let released = producer.source().section.as_ref().unwrap().hybrid_stats();
    assert_eq!((released.sprite_bodies, released.transition_bodies), (1, 0));
    assert_eq!(session.save(), before);

    let mut control = Session::load(before, Execution::Individuals).unwrap();
    let wound = Command::Wound {
        entity: target,
        part: Some(limb),
        cells: u32::MAX,
    };
    let result = session.command(wound.clone()).unwrap();
    assert_eq!(result, control.command(wound).unwrap());
    let wounded: Wounded = serde_json::from_str(&result).unwrap();
    assert!(wounded.severed.contains(&limb));
    let after = session.save();
    handle
        .update(&session, participant, window, Mode::Creative, site, 0)
        .unwrap();
    let changed = isolate(&handle.snapshot().unwrap(), true);
    handle.present(&changed, 1, view).unwrap();
    assert!(producer.render_scene(&request).unwrap().is_some());
    let section = producer.source().section.as_ref().unwrap();
    eprintln!("Mesocosm wounded viewport {:?}", section.hybrid_stats());
    assert_eq!(
        (
            section.hybrid_stats().sprite_bodies,
            section.hybrid_stats().refreshes
        ),
        (1, 1)
    );
    assert!(
        !changed
            .body(target)
            .unwrap()
            .document
            .living()
            .any(|part| part.id == limb)
    );
    assert_eq!(session.save(), after);
    assert_eq!(session.sim.state_hash(), control.sim.state_hash());
}
