// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Walking and sight in the played site (wing rulings 670, 696, 702 and
//! 741): the site lifted by isometer, the walker stepping between the
//! stances isometer's places read under the world's climb, and seeing along
//! isometer's ray. Positions are in the lifted `Ground`'s frame, which the
//! renderer and the contact solver read; isometer reads the site's.

use isometer_space::{places, volume::Volume};

/// How tall Eponym's walker is, in cells, for occupancy.
pub const WALKER_HEIGHT: i32 = 2;

/// Half the window's widest side: where `Ground`'s origin sits in it.
fn half(v: &Volume) -> i64 {
    (v.max[0] - v.min[0]).max(v.max[1] - v.min[1]) / 2
}

/// A `Ground` cell in the site's frame.
pub fn to_site(v: &Volume, [x, y, z]: [i32; 3]) -> [i64; 3] {
    let e = half(v);
    [i64::from(x) + v.min[0] + e, i64::from(y) + v.datum, i64::from(z) + v.min[1] + e]
}

/// A site cell in the lifted `Ground`'s frame.
pub fn to_ground(v: &Volume, [x, y, z]: [i64; 3]) -> [i32; 3] {
    let e = half(v);
    [(x - v.min[0] - e) as i32, (y - v.datum) as i32, (z - v.min[1] - e) as i32]
}

/// Whether the walker fits standing at `at`.
pub fn stands(v: &Volume, at: [i32; 3]) -> bool {
    v.ground().stands(at, WALKER_HEIGHT)
}

/// One step toward `toward`: to the neighbouring stance that closes the
/// most distance across, then the most height, or nowhere.
pub fn step(v: &Volume, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
    let gap = |at: [i32; 3]| {
        let across = (toward[0] - at[0]).abs() + (toward[2] - at[2]).abs();
        (across, (toward[1] - at[1]).abs())
    };
    let rules = places::Rules::default();
    let best = places::steps(v, rules, to_site(v, from))
        .into_iter()
        .map(|(at, _)| to_ground(v, at))
        .filter(|at| stands(v, *at))
        .min_by_key(|at| gap(*at));
    match best {
        Some(at) if gap(at) < gap(from) => at,
        _ => from,
    }
}

/// Whether `eye` sees `target` within `range`, head to head.
pub fn spot(v: &Volume, eye: [i32; 3], target: [i32; 3], range: i32) -> bool {
    let near = (0..3).all(|k| (eye[k] - target[k]).abs() <= range);
    let head = |at: [i32; 3]| to_site(v, [at[0], at[1] + WALKER_HEIGHT - 1, at[2]]);
    near && places::ray(v, head(eye), head(target))
}
