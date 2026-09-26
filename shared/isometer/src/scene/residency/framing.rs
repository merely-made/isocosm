// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which bricks a camera frames, and the pointer box a framing fits in.

use super::*;

/// What one camera frames of a terrain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FramedBricks {
    /// The framed bricks, sorted, without repeats, never past the capacity.
    pub keys: Vec<[i16; 3]>,
    /// Framed bricks dropped because the atlas could not hold them.
    pub overflow: usize,
    /// A pointer extent holding any framing by a camera of this size,
    /// wherever it stands, so a map built at it is retargeted rather than
    /// rebuilt until the camera's size or the terrain's height moves.
    pub extent: [u32; 3],
}

/// The bricks `camera` can show of `source`: those whose screen box overlaps
/// the frame grown by `margin` world units along the screen's right and up,
/// and whose box along the camera's forward is within the slab's reach. At
/// most `capacity` are kept.
pub fn framed_bricks(
    camera: SlabCamera,
    margin: f32,
    source: &dyn BrickSource,
    capacity: usize,
) -> FramedBricks {
    let Some([low, high]) = source.bounds() else {
        return FramedBricks {
            keys: Vec::new(),
            overflow: 0,
            extent: [1; 3],
        };
    };
    let edge = BRICK as f32;
    let slab = [f32::from(low[1]) * edge, (f32::from(high[1]) + 1.0) * edge];
    let window = grown(camera.window(), margin);
    let frame = Frame::of(window, margin);
    // A brick's screen box can reach the frame where no point of the brick is
    // inside it, so the columns are searched over the window grown by one more
    // box width: every point of such a brick lies within that.
    let search = SlabWindow {
        half: [0, 1, 2].map(|axis| window.half[axis] + 2.0 * frame.spread[axis]),
        ..window
    };
    let extent = extent_of(search, slab, low, high);
    let Some([near, far]) = region(search, slab) else {
        return FramedBricks {
            keys: Vec::new(),
            overflow: 0,
            extent,
        };
    };
    let key = |value: f32| (value / edge).floor() as i32;
    let columns = |axis: usize, horizontal: usize| {
        key(near[horizontal]).max(low[axis].into())..=key(far[horizontal]).min(high[axis].into())
    };
    let mut framed = Vec::new();
    for z in columns(2, 1) {
        for x in columns(0, 0) {
            let column = [x as i16, z as i16];
            let layers = source.layers(column);
            for y in layers.start.max(low[1])..layers.end.min(high[1] + 1) {
                let brick = [column[0], y, column[1]];
                if let Some(distance) = frame.distance(brick) {
                    framed.push((distance, brick));
                }
            }
        }
    }
    let overflow = framed.len().saturating_sub(capacity);
    if overflow > 0 {
        framed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        framed.truncate(capacity);
    }
    let mut keys: Vec<_> = framed.into_iter().map(|(_, brick)| brick).collect();
    keys.sort_unstable();
    FramedBricks {
        keys,
        overflow,
        extent,
    }
}

/// The camera's window with the margin added to its screen half extents.
fn grown(window: SlabWindow, margin: f32) -> SlabWindow {
    SlabWindow {
        half: [
            window.half[0] + margin,
            window.half[1] + margin,
            window.half[2],
        ],
        ..window
    }
}

/// The frame a brick is tested against, in the camera's own axes.
struct Frame {
    centre: [f32; 3],
    axes: [[f32; 3]; 3],
    /// The grown half extents: where a brick stops overlapping.
    reach: [f32; 3],
    /// The frame's own half extents, before the margin: the unit a brick's
    /// distance from the centre is counted in.
    unit: [f32; 2],
    /// Half a brick's box, measured along each axis.
    spread: [f32; 3],
}

impl Frame {
    fn of(window: SlabWindow, margin: f32) -> Self {
        let half = BRICK as f32 / 2.0;
        Self {
            centre: window.centre,
            axes: window.axes,
            reach: window.half,
            unit: [
                (window.half[0] - margin).max(f32::EPSILON),
                (window.half[1] - margin).max(f32::EPSILON),
            ],
            spread: window
                .axes
                .map(|axis| half * axis.iter().map(|v| v.abs()).sum::<f32>()),
        }
    }

    /// How far from the frame's centre the brick's nearest point lands, in
    /// frame half-widths (1 is the frame's own edge), or `None` when the brick
    /// misses the grown frame or the slab.
    fn distance(&self, key: [i16; 3]) -> Option<f32> {
        let half = BRICK as f32 / 2.0;
        let offset = [0, 1, 2].map(|i| f32::from(key[i]) * BRICK as f32 + half - self.centre[i]);
        let mut nearest = 0.0f32;
        for axis in 0..3 {
            let along: f32 = (0..3).map(|i| offset[i] * self.axes[axis][i]).sum();
            let gap = along.abs() - self.spread[axis];
            if gap > self.reach[axis] {
                return None;
            }
            if axis < 2 {
                nearest = nearest.max(gap.max(0.0) / self.unit[axis]);
            }
        }
        Some(nearest)
    }
}

/// The x and z bounds of the part of `window` between the heights in `slab`:
/// the box of the corners inside the slab and the edges crossing its faces.
fn region(window: SlabWindow, slab: [f32; 2]) -> Option<[[f32; 2]; 2]> {
    // Corner `bits` takes the positive half extent on each axis whose bit is
    // set; an edge joins two corners one bit apart.
    let corners: [[f32; 3]; 8] = std::array::from_fn(|bits| {
        let mut point = window.centre;
        for axis in 0..3 {
            let sign = if bits >> axis & 1 == 1 { 1.0 } else { -1.0 };
            for (i, value) in point.iter_mut().enumerate() {
                *value += sign * window.half[axis] * window.axes[axis][i];
            }
        }
        point
    });
    let mut low = [f32::INFINITY; 2];
    let mut high = [f32::NEG_INFINITY; 2];
    let mut take = |point: [f32; 3]| {
        for (slot, value) in [point[0], point[2]].into_iter().enumerate() {
            low[slot] = low[slot].min(value);
            high[slot] = high[slot].max(value);
        }
    };
    for point in corners {
        if (slab[0]..=slab[1]).contains(&point[1]) {
            take(point);
        }
    }
    for bits in 0..8 {
        for axis in 0..3 {
            if bits >> axis & 1 == 1 {
                continue;
            }
            let (p, q) = (corners[bits], corners[bits | 1 << axis]);
            for level in slab {
                if (p[1] - level) * (q[1] - level) < 0.0 {
                    let t = (level - p[1]) / (q[1] - p[1]);
                    take([0, 1, 2].map(|i| p[i] + t * (q[i] - p[i])));
                }
            }
        }
    }
    (low[0] <= high[0] && low[1] <= high[1]).then_some([low, high])
}

/// The pointer extent any framing by this window's size fits in: its region
/// measured with the window stood at the slab's mid-height, so the answer
/// does not move as the camera pans, one brick of slack for where a region's
/// edges fall on the grid, and never more than the terrain's own key box.
fn extent_of(window: SlabWindow, slab: [f32; 2], low: [i16; 3], high: [i16; 3]) -> [u32; 3] {
    let span = |axis: usize| (i32::from(high[axis]) - i32::from(low[axis]) + 1).max(1) as u32;
    let standing = SlabWindow {
        centre: [0.0, (slab[0] + slab[1]) / 2.0, 0.0],
        ..window
    };
    let region = region(standing, slab);
    let across = |horizontal: usize, axis: usize| {
        region
            .map(|[near, far]| far[horizontal] - near[horizontal])
            .filter(|width| width.is_finite())
            .map_or(span(axis), |width| {
                ((width / BRICK as f32).floor() as u32 + 3).clamp(1, span(axis))
            })
    };
    [across(0, 0), span(1), across(1, 2)]
}
