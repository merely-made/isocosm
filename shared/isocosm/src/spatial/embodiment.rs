// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Whole native part boxes over the existing terrain and spatial clock.
//! Version-two generated templates opt in; recorded version-one worlds retain
//! their root/eye contract. No display volumes participate in physical checks.

use super::*;
use isometer_core::BodyDocument;
use isometer_space::{places::Places, volume::Volume};

mod contacts;
mod founding;
pub use contacts::{Contact, contacts, sweep_clear};
#[cfg(test)]
mod tests;
pub use founding::declare;

pub type Box3 = [[i64; 3]; 2];
pub const SEARCH_REACH: i64 = 24;
pub const SEARCH_STRIDE: usize = 2;
pub const SEARCH_CANDIDATES: usize = 625;

pub fn enabled(sim: &Simulation) -> bool {
    sim.genesis()
        .rules
        .spatial_templates
        .as_ref()
        .is_some_and(|t| t.version >= 2)
}

/// Exact half-open boxes, including each attachment's yaw and the root yaw.
pub fn boxes(document: &BodyDocument, pose: &Pose) -> Result<Vec<Box3>> {
    let mut out = Vec::new();
    let mut cells = 0u64;
    for part in document
        .living()
        .filter(|part| part.half_extent.iter().all(|h| *h > 0))
    {
        let size = part
            .half_extent
            .map(|h| h.checked_mul(2).ok_or("body extent overflow"));
        let size = [size[0].clone()?, size[1].clone()?, size[2].clone()?];
        let mut lo = [i64::MAX; 3];
        let mut hi = [i64::MIN; 3];
        for x in [0, size[0]] {
            for y in [0, size[1]] {
                for z in [0, size[2]] {
                    let local = document
                        .place(part.id, [x, y, z])
                        .ok_or("unplaceable native body part")?
                        .map(i64::from);
                    let offset = super::birth::rotate(pose.yaw, local)?;
                    for k in 0..3 {
                        let value = pose.at[k]
                            .checked_add(offset[k])
                            .ok_or("body placement overflow")?;
                        lo[k] = lo[k].min(value);
                        hi[k] = hi[k].max(value);
                    }
                }
            }
        }
        let count = (0..3)
            .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
            .ok_or("body clearance overflow")?;
        cells = cells
            .checked_add(count)
            .filter(|n| *n <= 1_048_576)
            .ok_or("whole body exceeds bounded clearance work")?;
        out.push([lo, hi]);
    }
    if out.is_empty() {
        return Err("native body has no occupied part boxes".into());
    }
    Ok(out)
}

pub fn intersects(a: &[Box3], b: &[Box3]) -> bool {
    a.iter().any(|[lo, hi]| {
        b.iter()
            .any(|[other_lo, other_hi]| (0..3).all(|k| lo[k] < other_hi[k] && other_lo[k] < hi[k]))
    })
}

pub fn air(volume: &Volume, at: [i64; 3]) -> bool {
    at[1] >= volume.datum
        && at[1]
            .checked_sub(volume.datum)
            .is_some_and(|y| y <= i64::from(i32::MAX))
        && volume.material(at) == Some(volume.materials.air)
}

/// Every occupied part cell must fit; at least one actual part floor touches
/// non-water ground. Empty attachment points have no collision volume.
pub fn check(volume: &Volume, document: &BodyDocument, pose: &Pose) -> Result<()> {
    check_occupied(volume, boxes(document, pose)?)
}

fn check_occupied(volume: &Volume, occupied: Vec<Box3>) -> Result<()> {
    let mut supported = false;
    for [lo, hi] in occupied {
        if !volume.holds(lo[0], lo[2]) || !volume.holds(hi[0] - 1, hi[2] - 1) {
            return Err("whole body leaves its declared terrain window".into());
        }
        let count = (0..3)
            .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
            .ok_or("body clearance overflow")?;
        if count > 1_048_576 {
            return Err("native part exceeds bounded clearance work".into());
        }
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                let below = [x, lo[1].checked_sub(1).ok_or("body support overflow")?, z];
                supported |= contacts::supported(volume, below);
                for y in lo[1]..hi[1] {
                    if !air(volume, [x, y, z]) {
                        return Err("whole body intersects terrain or water".into());
                    }
                }
            }
        }
    }
    if !supported {
        return Err("whole body has no actual ground contact".into());
    }
    Ok(())
}

pub(crate) fn translated(local: &[Box3], at: [i64; 3]) -> Result<Vec<Box3>> {
    local
        .iter()
        .map(|bounds| {
            let mut placed = *bounds;
            for corner in &mut placed {
                for k in 0..3 {
                    corner[k] = corner[k]
                        .checked_add(at[k])
                        .ok_or("body translation overflow")?;
                }
            }
            Ok(placed)
        })
        .collect()
}

/// Highest terrain cells are read from Ground, including edits and water.
/// Reused across a bounded stand/path search, never persisted as authority.
pub struct Grounding<'a> {
    volume: &'a Volume,
    tops: BTreeMap<[i64; 2], i64>,
    // Exact terrain-only answers. Occupancy is always tested again by find.
    stands: BTreeMap<Vec<Box3>, BTreeMap<[i64; 2], Option<[i64; 3]>>>,
    stand_count: usize,
}

impl<'a> Grounding<'a> {
    pub fn new(volume: &'a Volume) -> Self {
        Self {
            volume,
            tops: BTreeMap::new(),
            stands: BTreeMap::new(),
            stand_count: 0,
        }
    }

    pub fn at(&mut self, document: &BodyDocument, x: i64, z: i64, yaw: Yaw) -> Result<Pose> {
        let local = boxes(document, &Pose { at: [0; 3], yaw })?;
        self.at_boxes(&local, x, z, yaw)
    }

    pub(crate) fn at_boxes(&mut self, relative: &[Box3], x: i64, z: i64, yaw: Yaw) -> Result<Pose> {
        let local = translated(relative, [x, 0, z])?;
        let mut y = i64::MIN;
        for [lo, hi] in &local {
            if !self.volume.holds(lo[0], lo[2]) || !self.volume.holds(hi[0] - 1, hi[2] - 1) {
                return Err("whole body leaves its declared terrain window".into());
            }
            for x in lo[0]..hi[0] {
                for z in lo[2]..hi[2] {
                    let top = *self.tops.entry([x, z]).or_insert_with(|| {
                        let cell = self.volume.to_ground([x, self.volume.datum, z]);
                        self.volume
                            .ground()
                            .surface(cell[0], cell[2])
                            .map_or(self.volume.datum - 1, |height| {
                                self.volume.datum + i64::from(height)
                            })
                    });
                    y = y.max(
                        top.checked_add(1)
                            .and_then(|top| top.checked_sub(lo[1]))
                            .ok_or("grounding overflow")?,
                    );
                }
            }
        }
        let pose = Pose { at: [x, y, z], yaw };
        check_occupied(self.volume, translated(relative, pose.at)?)?;
        Ok(pose)
    }

    /// Nearest candidates first, a declared lattice and finite attempt budget.
    pub fn find(
        &mut self,
        document: &BodyDocument,
        centre: [i64; 3],
        yaw: Yaw,
        reach: i64,
        stride: usize,
        cap: usize,
        blocked: &[Box3],
    ) -> Result<Pose> {
        let local = boxes(document, &Pose { at: [0; 3], yaw })?;
        let mut tried = 0;
        for ring in (0..=reach).step_by(stride.max(1)) {
            for x in (centre[0] - ring..=centre[0] + ring).step_by(stride.max(1)) {
                for z in (centre[2] - ring..=centre[2] + ring).step_by(stride.max(1)) {
                    if (x - centre[0]).abs().max((z - centre[2]).abs()) != ring {
                        continue;
                    }
                    tried += 1;
                    if tried > cap {
                        return Err("body placement candidate budget exhausted".into());
                    }
                    if let Some(pose) = self.stand(&local, x, z, yaw)
                        && !intersects(&translated(&local, pose.at)?, blocked)
                    {
                        return Ok(pose);
                    }
                }
            }
        }
        Err("no supported whole-body placement in the declared search".into())
    }

    fn stand(&mut self, local: &[Box3], x: i64, z: i64, yaw: Yaw) -> Option<Pose> {
        if let Some(at) = self
            .stands
            .get(local)
            .and_then(|stands| stands.get(&[x, z]))
        {
            return at.map(|at| Pose { at, yaw });
        }
        let pose = self.at_boxes(local, x, z, yaw).ok();
        // Bounded for the whole immutable terrain window, including refusals.
        if local.len() <= 64 && self.stand_count < 4096 {
            self.stands
                .entry(local.to_vec())
                .or_default()
                .insert([x, z], pose.as_ref().map(|p| p.at));
            self.stand_count += 1;
        }
        pose
    }
}

/// The same four-column movement and climb bound as existing place walking,
/// with the actual whole body grounded and checked at each destination.
pub fn neighbours(
    volume: &Volume,
    places: &Places,
    profile: &StepProfile,
    document: &BodyDocument,
    pose: &Pose,
) -> Vec<Pose> {
    let mut grounding = Grounding::new(volume);
    let Ok(local) = boxes(
        document,
        &Pose {
            at: [0; 3],
            yaw: pose.yaw,
        },
    ) else {
        return Vec::new();
    };
    neighbours_from_boxes(places, profile, &local, pose, &mut grounding)
}

pub(crate) fn neighbours_from_boxes(
    places: &Places,
    profile: &StepProfile,
    local: &[Box3],
    pose: &Pose,
    grounding: &mut Grounding<'_>,
) -> Vec<Pose> {
    let Some(from) = places.place_at(pose.at) else {
        return Vec::new();
    };
    [[1, 0], [-1, 0], [0, 1], [0, -1]]
        .into_iter()
        .filter_map(|[dx, dz]| {
            let next = grounding
                .at_boxes(
                    local,
                    pose.at[0].checked_add(dx)?,
                    pose.at[2].checked_add(dz)?,
                    pose.yaw,
                )
                .ok()?;
            if next.at[1].abs_diff(pose.at[1]) > u64::from(profile.body.climb) {
                return None;
            }
            let to = places.place_at(next.at)?;
            if from != to {
                let key = if from < to { [from, to] } else { [to, from] };
                profile.body.fits(places.passages.get(&key)?)?;
            }
            Some(next)
        })
        .collect()
}
