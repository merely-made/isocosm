//! Changing local settings must reach a parked scene exactly once.

use super::harness::{PANE, device};
use super::{BoardProducer, BoardSource, BoardView};
use crate::{demo::demo_map, state::UiState};
use isometer::{FrameRequest, Rebuild, ResidencySettings};

#[test]
#[ignore = "manual receipt requires a real GPU"]
fn changing_terrain_settings_rebuilds_a_parked_scene_once() {
    let (device, queue) = device().expect("this receipt requires a GPU");
    let mut ui = UiState::new(demo_map());
    // A dimmed increment at the device cap must not silently replace a
    // larger persisted request with that machine's cap.
    ui.terrain_settings.budget_mib = 64;
    ui.terrain_settings.step(1);
    assert_eq!(ui.terrain_settings.budget_mib, 64, "limits not known yet");
    ui.terrain_settings.report(32 * 1024 * 1024, 0);
    ui.terrain_settings.step(1);
    assert_eq!(
        ui.terrain_settings.budget_mib, 64,
        "dimmed increment is inert"
    );
    ui.terrain_settings.step(-1);
    assert_eq!(
        ui.terrain_settings.budget_mib, 31,
        "explicit decrease changes the request"
    );
    ui.terrain_settings = Default::default();
    ui.viewport = PANE;
    let mut view = BoardView::new(ui.map.clone());
    view.sync(&ui);
    let handle = view.into_handle();
    let mut producer = BoardProducer::new(BoardSource::new(handle.clone()));
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size: [640, 480],
        aspect: 640.0 / 480.0,
        color: None,
        needs_frame: false,
        render_scale: 1,
    };
    assert!(producer.render_scene(&request).unwrap().is_some());
    assert_upfront(&producer, 8 * 1024 * 1024);
    assert!(producer.render_scene(&request).unwrap().is_none());
    let mut renders = producer.renders();
    for budget in [4u64, 12, 64, 2] {
        producer.source_mut().set_atlas_budget(budget * 1024 * 1024);
        assert!(
            producer.render_scene(&request).unwrap().is_some(),
            "changed budget must draw"
        );
        renders += 1;
        assert_eq!(producer.renders(), renders);
        let cost = producer.source().ground_cost().unwrap();
        assert_eq!(cost.residency.rebuilt, Some(Rebuild::Requested));
        let diagnostics = producer.source().terrain_diagnostics().unwrap();
        assert!(diagnostics.full_map_upload);
        assert_upfront(
            &producer,
            (budget * 1024 * 1024).min(producer.source().atlas_device_budget().unwrap()),
        );
        assert_eq!(
            producer.source().ground().unwrap().limits().max_atlas_bytes,
            (budget * 1024 * 1024).min(producer.source().atlas_device_budget().unwrap())
        );
        assert!(producer.render_scene(&request).unwrap().is_none());
        assert_eq!(producer.renders(), renders);
    }
    producer
        .source_mut()
        .set_residency(ResidencySettings { headroom: 2 });
    assert!(producer.render_scene(&request).unwrap().is_some());
    assert_eq!(
        producer.source().ground_cost().unwrap().residency.rebuilt,
        Some(Rebuild::Requested)
    );
    assert!(producer.render_scene(&request).unwrap().is_none());
    // Chosen capacity stands through ordinary view and content changes.
    for edit in [false, true] {
        if edit {
            ui.map.elevation.set(0, 0, 1);
        } else {
            ui.camera.0 += 8.0;
        }
        handle.borrow_mut().sync(&ui);
        assert!(producer.render_scene(&request).unwrap().is_some());
        let diagnostics = producer.source().terrain_diagnostics().unwrap();
        assert!(!diagnostics.full_map_upload && !diagnostics.map_recreated);
        assert_eq!(diagnostics.resource_creations, 0);
        assert_eq!(producer.source().ground().unwrap().capacity(), 4095);
        if edit {
            assert!(diagnostics.changed_slots_declared > 0);
        }
        eprintln!(
            "[terrain-settings] edit={edit} resource_creations=0 capacity=4095 upload_bytes={}",
            diagnostics.brick_upload_bytes
        );
        assert!(producer.render_scene(&request).unwrap().is_none());
    }
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    eprintln!(
        "[terrain-settings] budgets 8 -> 4 -> 12 -> 64 (device-clamped) -> 2 MiB; each one rebuild then skip; headroom one rebuild then skip; renders {}",
        producer.renders()
    );
}

fn assert_upfront(producer: &BoardProducer, expected_bytes: u64) {
    let stats = producer.source().ground_cost().unwrap().residency;
    let diagnostics = producer.source().terrain_diagnostics().unwrap();
    let pointers = stats.extent.into_iter().map(u64::from).product::<u64>() * 4;
    assert!(stats.resident > 0 && stats.resident < stats.capacity);
    assert!(diagnostics.full_map_upload);
    assert_eq!(
        diagnostics.brick_upload_bytes - pointers,
        expected_bytes,
        "actual complete atlas upload includes empty chosen capacity"
    );
    assert_eq!((stats.capacity as u64 + 1) * 512, expected_bytes);
    eprintln!(
        "[terrain-settings] upfront atlas_bytes={expected_bytes} resident={} capacity={}",
        stats.resident, stats.capacity
    );
}
