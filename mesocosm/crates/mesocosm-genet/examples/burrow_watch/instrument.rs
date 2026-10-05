// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The burrow watch's chrome bar, drawn as its instrument panel.

use super::SIZE;
use netrender::Scene;

/// The instrument panel, in rects: sight lamp, closing distance, and a
/// threshold lamp. A judgment about tension needs the simulation's own
/// state legible, or it becomes a judgment about the picture.
pub(super) fn instrument(
    seen: bool,
    distance: Option<f32>,
    carved: bool,
    at_doorway: bool,
) -> Scene {
    let width = SIZE[0] as f32;
    let mut scene = Scene::new(SIZE[0], SIZE[1]);
    scene.push_rect(0.0, 0.0, width, 44.0, [0.02, 0.03, 0.05, 0.86]);

    // Sight: green while it cannot see you, red when it can, and a
    // dead grey when there is no hunter left to see anything. The third
    // state exists because the scenario reaches it.
    let sight = match (distance, seen) {
        (None, _) => [0.35, 0.35, 0.38, 0.9],
        (Some(_), true) => [0.94, 0.28, 0.24, 0.95],
        (Some(_), false) => [0.22, 0.78, 0.48, 0.92],
    };
    scene.push_rect(18.0, 12.0, 150.0, 32.0, sight);

    // Distance, closing left to right: the bar grows as it nears.
    let near = distance.map_or(0.0, |d| (1.0 - (d / 12.0).clamp(0.0, 1.0)).clamp(0.0, 1.0));
    let bar_left = 170.0;
    let bar_right = width - 200.0;
    scene.push_rect(bar_left, 18.0, bar_right, 26.0, [0.10, 0.12, 0.16, 0.9]);
    scene.push_rect(
        bar_left,
        18.0,
        bar_left + (bar_right - bar_left) * near,
        26.0,
        if seen {
            [0.95, 0.55, 0.25, 0.95]
        } else {
            [0.35, 0.45, 0.62, 0.95]
        },
    );

    // Carved lamp, then the threshold lamp when it actually steps in.
    if carved {
        scene.push_rect(
            width - 180.0,
            12.0,
            width - 110.0,
            32.0,
            [0.85, 0.75, 0.30, 0.9],
        );
    }
    if at_doorway {
        scene.push_rect(
            width - 96.0,
            12.0,
            width - 18.0,
            32.0,
            [0.95, 0.25, 0.30, 0.98],
        );
    }
    scene.push_rect(
        0.0,
        0.0,
        width,
        2.0,
        if seen {
            [0.94, 0.28, 0.24, 1.0]
        } else {
            [0.22, 0.78, 0.48, 1.0]
        },
    );
    scene
}
