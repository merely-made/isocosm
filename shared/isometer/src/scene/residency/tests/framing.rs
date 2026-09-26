// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipts for which bricks a camera frames, and the box a framing fits in.

use super::*;

#[test]
fn every_framed_brick_is_one_the_frame_can_show() {
    let hills = Hills::new();
    for margin in [0.0, 8.0] {
        for centre in [[0.0, 0.0], [3.0, 9.0], [31.7, -12.3], [-70.2, 55.9]] {
            let view = camera(centre);
            let framed = keys(&framed_bricks(view, margin, &hills, usize::MAX));
            let inside = brute(view, margin, -1e-3, &hills);
            let outside = brute(view, margin, 1e-3, &hills);
            assert!(!inside.is_empty(), "{centre:?}: the frame shows terrain");
            assert!(
                inside.is_subset(&framed),
                "{centre:?} margin {margin}: a brick the frame shows was not named"
            );
            assert!(
                framed.is_subset(&outside),
                "{centre:?} margin {margin}: a brick the frame cannot show was named"
            );
        }
    }
}

#[test]
fn a_framing_is_a_function_of_the_view_and_grows_with_its_margin() {
    let hills = Hills::new();
    let centre = [12.5, -4.25];
    assert_eq!(frame(&hills, centre), frame(&hills, centre));
    let at = |margin| keys(&framed_bricks(camera(centre), margin, &hills, usize::MAX));
    let (none, one, two) = (at(0.0), at(8.0), at(16.0));
    assert!(none.is_subset(&one) && one.is_subset(&two));
    assert!(none.len() < one.len() && one.len() < two.len());
}

#[test]
fn overflow_drops_the_margin_before_the_frame() {
    let hills = Hills::new();
    let view = camera([3.0, 9.0]);
    let exact = framed_bricks(view, 0.0, &hills, usize::MAX);
    let wide = framed_bricks(view, 8.0, &hills, usize::MAX);
    let squeezed = framed_bricks(view, 8.0, &hills, exact.keys.len());
    assert_eq!(wide.overflow, 0);
    assert_eq!(
        squeezed.keys, exact.keys,
        "held to the frame's own count, the frame is what stays"
    );
    assert_eq!(squeezed.overflow, wide.keys.len() - exact.keys.len());
    let tighter = framed_bricks(view, 8.0, &hills, exact.keys.len() / 2);
    assert_eq!(tighter.keys.len(), exact.keys.len() / 2);
    assert_eq!(
        tighter,
        framed_bricks(view, 8.0, &hills, exact.keys.len() / 2),
        "and what it drops is decided the same way every time"
    );
}

#[test]
fn the_extent_holds_every_framing_as_the_camera_pans() {
    let hills = Hills::new();
    let extent_of = |framed: &FramedBricks| framed.extent(framed.layers().expect("terrain"));
    let extent = extent_of(&frame(&hills, [0.0, 0.0]));
    for step in 0..40 {
        let centre = [step as f32 * 3.7 - 70.0, 40.0 - step as f32 * 2.9];
        let framed = frame(&hills, centre);
        assert_eq!(
            extent_of(&framed),
            extent,
            "{centre:?}: a pan does not move it"
        );
        let span = span_of(&framed.keys);
        assert!(
            (0..3).all(|axis| span[axis] <= extent[axis]),
            "{centre:?}: {span:?} past {extent:?}"
        );
    }
}
