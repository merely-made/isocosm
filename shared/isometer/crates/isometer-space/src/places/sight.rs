// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Sight (ruling 741): a ray through the volume's cells, with places
//! choosing which targets are worth casting at. Candidates are chosen by
//! where places lie, not by passages, because sight crosses a chasm a walk
//! has to go round.

use super::{PlaceId, Places};
use crate::volume::Volume;
use std::collections::BTreeSet;

impl Places {
    /// The places with a cell within `range` of `eye` on every axis.
    pub fn candidates(&self, eye: [i64; 3], range: u64) -> BTreeSet<PlaceId> {
        let r = range as i64;
        self.places
            .values()
            .filter(|p| (0..3).all(|k| p.bounds[0][k] - r <= eye[k] && eye[k] <= p.bounds[1][k] + r))
            .map(|p| p.id)
            .collect()
    }

    /// Whether `eye` sees `target` within `range`: the target's place is a
    /// candidate, and the ray between them is clear.
    pub fn sees(&self, volume: &Volume, eye: [i64; 3], target: [i64; 3], range: u64) -> bool {
        let near = (0..3).all(|k| eye[k].abs_diff(target[k]) <= range);
        let candidate = self
            .place_at(target)
            .is_some_and(|p| self.candidates(eye, range).contains(&p));
        near && candidate && ray(volume, eye, target)
    }
}

/// Whether no solid cell lies strictly between two cells' centres. Air and
/// water are clear; outside the window is not. Sampled at half-cell
/// strides in integers, so the walk is the same everywhere and never skips
/// a corner.
pub fn ray(volume: &Volume, from: [i64; 3], to: [i64; 3]) -> bool {
    let m = volume.materials;
    let clear = |at: [i64; 3]| volume.material(at).is_some_and(|x| x == m.air || x == m.water);
    let delta = [0, 1, 2].map(|k| to[k] - from[k]);
    let strides = 2 * delta.iter().map(|d| d.abs()).max().unwrap_or(0);
    let mut last = from;
    for stride in 1..strides {
        let at = [0, 1, 2].map(|k| from[k] + (delta[k] * stride).div_euclid(strides));
        if at == last || at == to {
            continue;
        }
        last = at;
        if !clear(at) {
            return false;
        }
    }
    true
}
