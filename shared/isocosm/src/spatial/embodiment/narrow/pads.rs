// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! A turning sole against voxel columns, using integer separating axes.
use isometer_core::legs::{Bounds, Point, Rotation, UNIT};

pub(in crate::spatial::embodiment) struct Pad {
    corners: [[i64; 2]; 4],
    axes: [[i64; 2]; 2],
    pub bounds: Bounds,
}
impl Pad {
    pub fn new(local: Bounds, rotation: Rotation, origin: Point) -> Option<Self> {
        let mut corners = [[0; 2]; 4];
        for (i, (x, z)) in [
            (local[0][0], local[0][2]),
            (local[0][0], local[1][2]),
            (local[1][0], local[0][2]),
            (local[1][0], local[1][2]),
        ]
        .into_iter()
        .enumerate()
        {
            let p = rotation.rotate([x, 0, z])?;
            corners[i] = [p[0] + origin[0], p[2] + origin[2]];
        }
        Some(Self {
            corners,
            axes: [
                [rotation.cosine, -rotation.sine],
                [rotation.sine, rotation.cosine],
            ],
            bounds: rotation.bounds(local, origin)?,
        })
    }
    pub fn columns(&self, x: i64, z: i64) -> bool {
        let corners = [
            [x * UNIT, z * UNIT],
            [(x + 1) * UNIT, z * UNIT],
            [x * UNIT, (z + 1) * UNIT],
            [(x + 1) * UNIT, (z + 1) * UNIT],
        ];
        [[1, 0], [0, 1], self.axes[0], self.axes[1]]
            .into_iter()
            .all(|axis| {
                let project = |p: [i64; 2]| {
                    i128::from(p[0]) * i128::from(axis[0]) + i128::from(p[1]) * i128::from(axis[1])
                };
                let a = self.corners.map(project);
                let b = corners.map(project);
                a.iter().min().unwrap() < b.iter().max().unwrap()
                    && b.iter().min().unwrap() < a.iter().max().unwrap()
            })
    }
    pub fn hits(&self, at: Point) -> bool {
        self.bounds[0][1] < (at[1] + 1) * UNIT
            && at[1] * UNIT < self.bounds[1][1]
            && self.columns(at[0], at[2])
    }
    pub fn cells(&self) -> Bounds {
        [
            self.bounds[0].map(|v| v.div_euclid(UNIT)),
            self.bounds[1].map(|v| -(-v).div_euclid(UNIT)),
        ]
    }
}
