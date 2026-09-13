// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{BrickChange, BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Flight, Grade};
use mesocosm_core::places::{Ground, Places};

#[test]
fn cold_retained_and_carved_terrain_have_distinct_upload_receipts() {
    let mut ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
    let mut map = BrickMap::from_ground(&ground).expect("bounded map");
    let mut tracer =
        BrickTracer::headless(64, 64).expect("GPU adapter required for counter receipt");
    let top = ground.surface(4, 4).expect("ground column");
    let camera = Flight {
        eye: [4.5, top as f32 + 14., 4.5],
        yaw: 0.,
        pitch: -1.52,
        fov: 0.15,
        far: 48.,
    };
    let grade = Grade::clay();
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade);
    let cold = tracer.capture(input).expect("cold capture").diagnostics;
    assert!(cold.map_recreated && cold.full_map_upload);
    assert!(!cold.map_revision_unchanged);
    assert_eq!((cold.pointer_write_calls, cold.atlas_write_calls), (1, 1));
    assert_eq!(
        cold.brick_upload_bytes,
        (std::mem::size_of_val(map.pointers()) + map.atlas().len()) as u64
    );
    assert!(cold.uniform_write_calls > 0);
    let retained = tracer.capture(input).expect("retained capture").diagnostics;
    assert!(retained.map_revision_unchanged);
    assert!(!retained.map_recreated && !retained.full_map_upload);
    assert_eq!(
        (
            retained.pointer_write_calls,
            retained.atlas_write_calls,
            retained.uniform_write_calls
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (retained.brick_upload_bytes, retained.uniform_upload_bytes),
        (0, 0)
    );
    assert_eq!(retained.trace_passes, 1, "retained terrain is still traced");

    assert!(ground.carve([4, top, 4], 2) > 0);
    let changed = ground.drain_dirty();
    let slots = map.refresh(&ground, changed).expect("dirty slots");
    assert!(!slots.is_empty());
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
        .changed(BrickChange::Slots(&slots));
    let dirty = tracer.capture(input).expect("carved capture").diagnostics;
    assert!(!dirty.map_revision_unchanged && !dirty.full_map_upload && !dirty.map_recreated);
    assert_eq!(dirty.changed_slots_declared, slots.len());
    assert_eq!(dirty.pointer_write_calls as usize, slots.len());
    assert!(dirty.atlas_write_calls > 0 && dirty.atlas_write_calls as usize <= slots.len());
    assert_eq!(dirty.brick_upload_bytes, slots.len() as u64 * (512 + 4));
    let again = tracer
        .capture(input)
        .expect("same dirty declaration retained")
        .diagnostics;
    assert!(again.map_revision_unchanged);
    assert_eq!(
        again.changed_slots_declared, 0,
        "unchanged revision does not reprocess declarations"
    );
    assert_eq!((again.pointer_write_calls, again.atlas_write_calls), (0, 0));
}
