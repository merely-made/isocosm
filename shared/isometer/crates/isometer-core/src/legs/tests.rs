// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn lifting_through_a_horizontal_ankle_keeps_the_knee_above_ground() {
    let leg = Leg {
        upper: PartId(1),
        lower: PartId(2),
        foot: PartId(3),
        shoulder: [0; 3],
        knee: [4 * UNIT, 0, 0],
        sole: [5 * UNIT, -UNIT, 0],
        half: [UNIT; 3],
        radius: UNIT / 3,
        lengths: [4 * UNIT, 7 * UNIT],
        ankle_offset: [0, UNIT, 0],
    };
    let knees: Vec<_> = [-16, 0, 16]
        .map(|dy| leg.solve([5 * UNIT, -UNIT + dy, 0]).unwrap()[0])
        .into_iter()
        .collect();
    assert!(knees.iter().all(|p| p[1] > 0));
    for pair in knees.windows(2) {
        assert!(
            length(sub(pair[0], pair[1])) < UNIT / 4,
            "a tiny ankle lift must not flip the knee through the floor"
        );
    }
}
