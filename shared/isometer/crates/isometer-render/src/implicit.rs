// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation surfaces over declared anatomy. Drawing and picking share the
//! same bounded field; none of these floating point values enters a body save.
use crate::live_body::{BodyHit, BodyQueryError, ClipSlab};
use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};
use isometer_core::PartId;

mod anatomy;
mod chains;
mod contacts;
mod gait;
mod gpu;
#[cfg(test)]
mod tests;
pub use anatomy::{AnatomyStats, from_anatomy, two_bone, walking_anatomy};
pub use gait::Gait;
pub use gpu::{Frame, Renderer};

/// GPU layout, also evaluated by the CPU picker. Kind 0 is a rounded box;
/// kind 1 is a capsule. Quaternion maps local coordinates into world space.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Primitive {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub half: [f32; 4],
    pub rotation: [f32; 4],
    pub colour: [f32; 4],
    pub meta: [u32; 4],
}
impl Primitive {
    pub fn rounded_box(
        part: PartId,
        centre: Vec3,
        half: Vec3,
        rotation: Quat,
        radius: f32,
        colour: [f32; 3],
    ) -> Self {
        Self {
            a: centre.extend(0.0).to_array(),
            b: [0.0; 4],
            half: half.extend(radius).to_array(),
            rotation: rotation.to_array(),
            colour: [colour[0], colour[1], colour[2], 1.0],
            meta: [0, part.0, 0, 0],
        }
    }
    pub fn capsule(part: PartId, a: Vec3, b: Vec3, radius: f32, colour: [f32; 3]) -> Self {
        Self {
            a: a.extend(radius).to_array(),
            b: b.extend(0.0).to_array(),
            half: [0.0; 4],
            rotation: Quat::IDENTITY.to_array(),
            colour: [colour[0], colour[1], colour[2], 1.0],
            meta: [1, part.0, 0, 0],
        }
    }
    fn distance(&self, point: Vec3) -> f32 {
        let a = Vec3::from_slice(&self.a);
        if self.meta[0] == 1 {
            let ab = Vec3::from_slice(&self.b) - a;
            let t = ((point - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
            return (point - a - ab * t).length() - self.a[3];
        }
        let p = Quat::from_array(self.rotation).conjugate() * (point - a);
        let q = p.abs() - Vec3::from_slice(&self.half) + self.half[3];
        q.max(Vec3::ZERO).length() + q.max_element().min(0.0) - self.half[3]
    }
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let a = Vec3::from_slice(&self.a);
        if self.meta[0] == 1 {
            let b = Vec3::from_slice(&self.b);
            return (a.min(b) - self.a[3], a.max(b) + self.a[3]);
        }
        let q = Quat::from_array(self.rotation);
        let h = Vec3::from_slice(&self.half);
        let extent =
            (q * Vec3::X).abs() * h.x + (q * Vec3::Y).abs() * h.y + (q * Vec3::Z).abs() * h.z;
        (a - extent, a + extent)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Surface {
    pub parts: Vec<Primitive>,
    pub blend: f32,
}
impl Surface {
    pub fn valid(&self) -> bool {
        self.blend.is_finite()
            && self.blend >= 0.0
            && self.parts.iter().all(|p| {
                p.a.iter()
                    .chain(&p.b)
                    .chain(&p.half)
                    .chain(&p.rotation)
                    .chain(&p.colour)
                    .all(|v| v.is_finite())
                    && p.meta[0] <= 1
                    && p.a[3] >= 0.0
                    && p.half.iter().all(|v| *v >= 0.0)
                    && (Quat::from_array(p.rotation).length_squared() - 1.0).abs() < 0.01
            })
    }
    pub fn bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = -min;
        for p in &self.parts {
            let (a, b) = p.bounds();
            min = min.min(a);
            max = max.max(b);
        }
        // Repeated polynomial unions approach, but never exceed, k below
        // the nearest constituent distance. Padding by k bounds every part count.
        let padding = self.blend;
        (!self.parts.is_empty()).then_some(((min - padding).to_array(), (max + padding).to_array()))
    }
    pub fn part_bounds(&self, part: PartId) -> Option<([f32; 3], [f32; 3])> {
        let subset = Self {
            parts: self
                .parts
                .iter()
                .filter(|p| p.meta[1] == part.0)
                .copied()
                .collect(),
            blend: self.blend,
        };
        subset.bounds()
    }
    /// Returns distance and the largest local contributor, using the same
    /// stable order and tie policy as the shader.
    pub fn sample(&self, point: [f32; 3]) -> (f32, PartId) {
        let mut d = f32::INFINITY;
        let mut owner = PartId(0);
        for part in &self.parts {
            let next = part.distance(Vec3::from_array(point));
            if next < d {
                owner = PartId(part.meta[1]);
            }
            if self.blend > 0.0 && d.is_finite() {
                let h = (0.5 + 0.5 * (next - d) / self.blend).clamp(0.0, 1.0);
                d = next * (1.0 - h) + d * h - self.blend * h * (1.0 - h);
            } else {
                d = d.min(next);
            }
        }
        (d, owner)
    }
    pub fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: ClipSlab,
    ) -> Result<Option<BodyHit>, BodyQueryError> {
        let origin = Vec3::from_array(origin);
        let direction = Vec3::from_array(direction);
        if !self.valid() {
            return Err(crate::LiveBodyError::InvalidBody.into());
        }
        if !origin.is_finite()
            || !direction.is_finite()
            || direction.length_squared() == 0.0
            || !far.is_finite()
            || far < 0.0
        {
            return Err(BodyQueryError::InvalidRay);
        }
        let Some((mut min, mut max)) = self.bounds() else {
            return Ok(None);
        };
        if let Some((a, b)) = clip.bounds {
            min = Vec3::from_array(min).max(Vec3::from_array(a)).to_array();
            max = Vec3::from_array(max).min(Vec3::from_array(b)).to_array();
        }
        let direction = direction.normalize();
        let Some((mut t, mut end)) = interval(
            origin,
            direction,
            Vec3::from_array(min),
            Vec3::from_array(max),
            far,
        ) else {
            return Ok(None);
        };
        let normal = Vec3::from_array(clip.normal);
        let at = normal.dot(origin);
        let slope = normal.dot(direction);
        if slope.abs() < 1e-8 {
            if at < clip.min || at > clip.max {
                return Ok(None);
            }
        } else {
            let a = (clip.min - at) / slope;
            let b = (clip.max - at) / slope;
            t = t.max(a.min(b));
            end = end.min(a.max(b));
        }
        if end < t {
            return Ok(None);
        }
        for _ in 0..128 {
            let p = origin + direction * t;
            let (d, part) = self.sample(p.to_array());
            if d.abs() < 0.015 && contains(clip, p) {
                return Ok(Some(BodyHit {
                    body_index: 0,
                    part,
                    distance: t,
                    point: p.to_array(),
                    tied: false,
                }));
            }
            t += d.abs().max(0.008) * 0.8;
            if t > end {
                break;
            }
        }
        Ok(None)
    }
}
fn contains(clip: ClipSlab, p: Vec3) -> bool {
    let s = Vec3::from_array(clip.normal).dot(p);
    s >= clip.min
        && s <= clip.max
        && clip.bounds.is_none_or(|(a, b)| {
            p.cmpge(Vec3::from_array(a)).all() && p.cmple(Vec3::from_array(b)).all()
        })
}
fn interval(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3, far: f32) -> Option<(f32, f32)> {
    let mut start = 0.0_f32;
    let mut end = far;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-8 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - origin[axis]) / direction[axis];
            let b = (max[axis] - origin[axis]) / direction[axis];
            start = start.max(a.min(b));
            end = end.min(a.max(b));
        }
    }
    (end >= start).then_some((start, end))
}
