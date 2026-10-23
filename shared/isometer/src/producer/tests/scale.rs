// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The render-scale tests (I3), split from the producer tests at the
//! 600-line ceiling.

use super::*;

/// I3: scale 1 is the path that shipped. No presentation pass is built, the
/// view the leaf receives is the scene's own, and the bytes are the bytes an
/// unscaled producer draws.
#[test]
fn scale_one_takes_the_unscaled_path_byte_for_byte() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scale-1 receipt");
        return;
    };
    let mut plain = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    let view = plain
        .render_scene(&request(&device, &queue, false))
        .unwrap()
        .expect("the unscaled frame");
    assert_eq!(
        &view,
        plain.scene.display_view(),
        "an unscaled frame presents the scene's own view"
    );
    assert!(
        plain.display.is_none() && plain.upscale.is_none(),
        "and builds no presentation pass"
    );
    let unscaled = read_back(&device, &queue, plain.scene.display_texture(), SIZE);

    let mut declared = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    declared.set_render_scale(1);
    let view = declared
        .render_scene(&scaled(&device, &queue, SIZE, 1))
        .unwrap()
        .expect("the scale-1 frame");
    assert_eq!(&view, declared.scene.display_view());
    assert!(declared.display.is_none() && declared.upscale.is_none());
    assert_eq!(
        read_back(&device, &queue, declared.scene.display_texture(), SIZE),
        unscaled,
        "scale 1 is byte-identical to no scale at all"
    );
    assert_eq!(declared.signature().render_scale, 1);
}

/// I3's done-condition: a scene requested at scale 4 draws a quarter-sized
/// image, and the leaf receives a texture of its own size in which every 4 by
/// 4 block is one colour — the scene pixel it came from, not a blend of its
/// neighbours.
#[test]
fn a_scale_four_frame_presents_each_scene_pixel_as_one_block() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer render-scale receipt");
        return;
    };
    const LEAF: [u32; 2] = [64, 64];
    const SCALE: u32 = 4;
    let small = [LEAF[0] / SCALE, LEAF[1] / SCALE];

    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer.set_render_scale(SCALE);
    producer
        .render_scene(&scaled(&device, &queue, LEAF, SCALE))
        .unwrap()
        .expect("the scaled frame");

    assert_eq!(
        [producer.scene.width, producer.scene.height],
        small,
        "the scene drew at the scaled size"
    );
    assert_eq!(producer.signature().size, small, "and declared it");
    assert_eq!(producer.signature().render_scale, SCALE);
    let (texture, _, size) = producer.display.as_ref().expect("a presentation target");
    assert_eq!(*size, LEAF, "the leaf is handed its own physical size");

    let scene = read_back(&device, &queue, producer.scene.display_texture(), small);
    let presented = read_back(&device, &queue, texture, LEAF);
    for y in 0..small[1] {
        for x in 0..small[0] {
            let want = pixel(&scene, small[0], x, y);
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let got = pixel(&presented, LEAF[0], x * SCALE + dx, y * SCALE + dy);
                    assert_eq!(
                        got, want,
                        "block ({x}, {y}) offset ({dx}, {dy}) is not the scene pixel"
                    );
                }
            }
        }
    }
}

/// A scale change with nothing else moved is a real frame, even where the
/// scaled size floors to the same pixel and the signature's size cannot tell
/// the two apart.
#[test]
fn a_changed_render_scale_alone_is_a_new_frame() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scale-change receipt");
        return;
    };
    const LEAF: [u32; 2] = [4, 4];
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer
        .render_scene(&scaled(&device, &queue, LEAF, 4))
        .unwrap()
        .expect("the first scaled frame");
    assert_eq!(producer.renders(), 1);
    let banked = producer.signature().clone();
    assert_eq!(banked.size, [1, 1]);

    producer
        .render_scene(&scaled(&device, &queue, LEAF, 5))
        .unwrap()
        .expect("a changed scale must produce a frame");
    assert_eq!(producer.renders(), 2);
    assert_eq!(
        producer.signature().size,
        banked.size,
        "the scene size floored to the same pixel"
    );
    assert_eq!(producer.signature().render_scale, 5);
}

/// The skip is the skip at every scale: an unchanged scaled frame re-encodes
/// nothing and advances no generation.
#[test]
fn an_unchanged_scaled_frame_still_skips() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scaled-skip receipt");
        return;
    };
    const LEAF: [u32; 2] = [64, 64];
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer
        .render_scene(&scaled(&device, &queue, LEAF, 4))
        .unwrap()
        .expect("the first scaled frame");
    assert_eq!(producer.draws, 1);

    assert!(
        producer
            .render_scene(&scaled(&device, &queue, LEAF, 4))
            .unwrap()
            .is_none(),
        "an unchanged scaled frame must produce nothing"
    );
    assert_eq!(producer.renders(), 1, "a skip advances no generation");
    assert_eq!(producer.draws, 1, "and reaches no encode");
}
