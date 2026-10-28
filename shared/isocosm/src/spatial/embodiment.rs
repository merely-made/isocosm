// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Whole native part boxes over the existing terrain and spatial clock.
//! Version-two generated templates opt in; recorded version-one worlds retain
//! their root/eye contract. No display volumes participate in physical checks.

use super::*;
use isometer_core::BodyDocument;
use isometer_space::{places::Places, volume::Volume};

mod articulation;
mod balance;
use balance::check_balanced;
pub(crate) use balance::motion_member;
pub use balance::{
    Balance, FootLoad, Load, balance_for, check_member, check_motion, neighbours_for, step_for,
    stride_load,
};
mod contacts;
mod food;
mod footholds;
mod memo;
mod narrow;
use memo::Memo;
mod steps;
mod support;
mod walking;
pub use food::FoodStand;
pub(crate) use steps::articulated_neighbours;
pub(crate) use walking::Checks as StrideChecks;
pub use walking::local_feet as stride_feet;
pub use walking::{
    Walking, check_stride, check_walk, plan_stride, step_stride, stride_boxes, stride_sample,
    walking_rules,
};
mod swing;
pub use articulation::{articulated, foot_points};
pub use swing::{SAMPLES as SWING_SAMPLES, swing_boxes, swing_feet};
mod founding;
mod habitat;
pub use contacts::{Contact, contacts, sweep_clear};
pub use habitat::{declare_site, place_in_site, place_within, site_window, whole_site};
#[cfg(test)]
pub(super) mod articulation_tests;
#[cfg(test)]
pub(super) mod tests;
pub use founding::{anchor, declare};

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
    visit_boxes(document, pose, |_, bounds| out.push(bounds))?;
    Ok(out)
}

/// The same placed bounds keyed by actual part identity, for contact and senses.
pub fn part_boxes(
    document: &BodyDocument,
    pose: &Pose,
) -> Result<BTreeMap<isometer_core::PartId, Box3>> {
    let mut out = BTreeMap::new();
    visit_boxes(document, pose, |id, bounds| {
        out.insert(id, bounds);
    })?;
    Ok(out)
}

fn visit_boxes(
    document: &BodyDocument,
    pose: &Pose,
    mut put: impl FnMut(isometer_core::PartId, Box3),
) -> Result<()> {
    let articulated = articulation::part_boxes(document, pose)?;
    let mut occupied = false;
    let mut cells = 0u64;
    for part in document
        .living()
        .filter(|part| part.half_extent.iter().all(|h| *h > 0))
    {
        occupied = true;
        if let Some(bounds) = articulated.get(&part.id) {
            put(part.id, *bounds);
            continue;
        }
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
        put(part.id, [lo, hi]);
    }
    if !occupied {
        return Err("native body has no occupied part boxes".into());
    }
    Ok(())
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
    let occupied = boxes(document, pose)?;
    if pose.feet.is_empty() {
        check_occupied(volume, occupied)?;
    } else {
        narrow::clear(
            volume,
            document,
            pose.at.map(|v| v * isometer_core::legs::UNIT),
            pose.yaw,
            &pose.feet,
            &occupied,
            &[],
        )?;
        support::standing(volume, document, pose)?;
    }
    Ok(())
}

/// A new articulated world cannot submit an old rigid pose to bypass support.
pub fn check_for(
    sim: &Simulation,
    volume: &Volume,
    document: &BodyDocument,
    pose: &Pose,
) -> Result<()> {
    if articulated(sim) && pose.feet.is_empty() && !isometer_core::legs::chains(document).is_empty()
    {
        return Err("articulated world requires admitted paid-foot placement".into());
    }
    check(volume, document, pose)
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
    balance: Option<Balance>,
    costs: steps::Costs,
    volume: &'a Volume,
    tops: BTreeMap<[i64; 2], i64>,
    // Exact terrain-only answers. Occupancy is always tested again by find.
    stands: BTreeMap<Vec<Box3>, BTreeMap<[i64; 2], Option<[i64; 3]>>>,
    stand_count: usize,
    articulated: bool,
    spacing: i64,
    walking: Option<Walking>,
    articulated_stands:
        BTreeMap<BodyDocument, BTreeMap<([i64; 2], Yaw, Option<i64>), Option<Pose>>>,
}

impl<'a> Grounding<'a> {
    pub fn with_balance(mut self, balance: Option<Balance>) -> Self {
        self.set_balance(balance);
        self
    }
    fn set_balance(&mut self, balance: Option<Balance>) {
        if self.balance != balance {
            self.articulated_stands.clear();
        }
        self.balance = balance;
    }
    pub fn with_walking(mut self, walking: Option<Walking>) -> Self {
        self.set_walking(walking);
        self
    }
    fn set_walking(&mut self, walking: Option<Walking>) {
        if self.walking != walking {
            self.articulated_stands.clear();
        }
        self.walking = walking;
    }
    pub fn new(volume: &'a Volume) -> Self {
        Self {
            balance: None,
            costs: Default::default(),
            volume,
            tops: BTreeMap::new(),
            stands: BTreeMap::new(),
            stand_count: 0,
            articulated: false,
            spacing: 0,
            walking: None,
            articulated_stands: BTreeMap::new(),
        }
    }

    pub fn with_articulation(mut self, enabled: bool) -> Self {
        self.articulated = enabled;
        self
    }
    pub fn with_spacing(mut self, cells: u32) -> Self {
        self.spacing = i64::from(cells);
        self
    }

    pub fn at(&mut self, document: &BodyDocument, x: i64, z: i64, yaw: Yaw) -> Result<Pose> {
        self.at_near(document, x, z, yaw, None)
    }
    fn at_near(
        &mut self,
        document: &BodyDocument,
        x: i64,
        z: i64,
        yaw: Yaw,
        height: Option<i64>,
    ) -> Result<Pose> {
        if self.articulated && !isometer_core::legs::chains(document).is_empty() {
            if let Some(cached) = self
                .articulated_stands
                .get(document)
                .and_then(|s| s.get(&([x, z], yaw, height)))
            {
                return cached
                    .clone()
                    .ok_or_else(|| "cached unsupported chain posture".into());
            }
            let pose = articulation::ground(self, document, x, z, yaw, height);
            if self.stand_count < 4096 && self.articulated_stands.len() < 64 {
                self.articulated_stands
                    .entry(document.clone())
                    .or_default()
                    .insert(([x, z], yaw, height), pose.clone().ok());
                self.stand_count += 1;
            }
            return pose;
        }
        let local = boxes(
            document,
            &Pose {
                at: [0; 3],
                yaw,
                ..Default::default()
            },
        )?;
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
                        self.volume.surface(x, z).unwrap_or(self.volume.datum - 1)
                    });
                    y = y.max(
                        top.checked_add(1)
                            .and_then(|top| top.checked_sub(lo[1]))
                            .ok_or("grounding overflow")?,
                    );
                }
            }
        }
        let pose = Pose {
            at: [x, y, z],
            yaw,
            feet: Default::default(),
        };
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
        self.find_where(
            document,
            centre,
            yaw,
            reach,
            stride,
            cap,
            blocked,
            |_, _| true,
        )
    }
    pub fn find_where(
        &mut self,
        document: &BodyDocument,
        centre: [i64; 3],
        yaw: Yaw,
        reach: i64,
        stride: usize,
        cap: usize,
        blocked: &[Box3],
        mut accept: impl FnMut(&Pose, &[Box3]) -> bool,
    ) -> Result<Pose> {
        let local = boxes(
            document,
            &Pose {
                at: [0; 3],
                yaw,
                feet: Default::default(),
            },
        )?;
        let separated: Vec<_> = blocked
            .iter()
            .map(|[lo, hi]| {
                [
                    [0, 1, 2].map(|k| lo[k].saturating_sub(if k == 1 { 0 } else { self.spacing })),
                    [0, 1, 2].map(|k| hi[k].saturating_add(if k == 1 { 0 } else { self.spacing })),
                ]
            })
            .collect();
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
                    let candidate = if self.articulated {
                        self.at(document, x, z, yaw).ok()
                    } else {
                        self.stand(&local, x, z, yaw)
                    };
                    if let Some(pose) = candidate {
                        let occupied = boxes(document, &pose)?;
                        if !intersects(&occupied, &separated) && accept(&pose, &occupied) {
                            return Ok(pose);
                        }
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
            return at.map(|at| Pose {
                at,
                yaw,
                feet: Default::default(),
            });
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
    neighbours_with_walking(volume, places, profile, document, pose, None)
}

pub fn neighbours_with_walking(
    volume: &Volume,
    places: &Places,
    profile: &StepProfile,
    document: &BodyDocument,
    pose: &Pose,
    walking: Option<Walking>,
) -> Vec<Pose> {
    if !pose.feet.is_empty() {
        return StrideChecks::new(volume, document, &[]).neighbours(
            places,
            profile,
            &[],
            pose,
            walking,
        );
    }
    let mut grounding = Grounding::new(volume).with_walking(walking);
    let Ok(local) = boxes(
        document,
        &Pose {
            at: [0; 3],
            yaw: pose.yaw,
            feet: Default::default(),
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
