// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Derived posture work lives only as long as its immutable physical inputs.
use super::*;
use std::collections::VecDeque;

const CAPACITY: usize = 8192;
type Key = (Yaw, [i64; 3], i64, Vec<(PartId, [i64; 3])>, Vec<PartId>);
type Evaluation = Result<(Result<u64>, Result<()>)>;

pub(crate) struct Checks<'a> {
    pub(super) volume: &'a Volume,
    pub(super) document: &'a BodyDocument,
    pub(super) blocked: &'a [Box3],
    body: Body,
    clearance: narrow::Clearance<'a>,
    bearing: support::Bearing<'a>,
    entries: BTreeMap<Key, Evaluation>,
    order: VecDeque<Key>,
    pub(super) reads: usize,
    pub(super) misses: usize,
    #[cfg(test)]
    pub(super) capacity: usize,
}
struct Body {
    chains: Vec<legs::Leg>,
    rest: Result<Vec<(PartId, Box3)>>,
    centre: [i64; 3],
}
impl Body {
    fn new(document: &BodyDocument) -> Self {
        Self {
            chains: legs::chains(document),
            rest: boxes(document, &Pose::default()).map(|rest| {
                document
                    .living()
                    .filter(|p| p.half_extent.iter().all(|h| *h > 0))
                    .map(|p| p.id)
                    .zip(rest)
                    .collect()
            }),
            centre: support::centre_local(document),
        }
    }
    fn posed(
        &self,
        origin: [i64; 3],
        rotation: Rotation,
        feet: &BTreeMap<PartId, [i64; 3]>,
    ) -> Result<(Vec<Box3>, Vec<legs::Skin>)> {
        let mut replacements = BTreeMap::new();
        let mut skins = Vec::new();
        for leg in &self.chains {
            let skin = leg
                .skin(*feet.get(&leg.foot).ok_or("planned foot absent")?)
                .ok_or("weight transfer exceeds paid reach")?;
            for (id, bounds) in [leg.upper, leg.lower, leg.foot]
                .into_iter()
                .zip(skin.occupied())
            {
                replacements.insert(id, placed(bounds, rotation, origin)?);
            }
            skins.push(skin);
        }
        let occupied = self
            .rest
            .as_ref()
            .map_err(Clone::clone)?
            .iter()
            .map(|(id, bounds)| {
                replacements.get(id).copied().map(Ok).unwrap_or_else(|| {
                    placed(bounds.map(|p| p.map(|v| v * UNIT)), rotation, origin)
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok((occupied, skins))
    }
}
impl<'a> Checks<'a> {
    #[cfg(test)]
    pub(crate) fn without_reuse(mut self) -> Self {
        self.capacity = 0;
        self.clearance = narrow::Clearance::new(self.volume, 0);
        self.bearing = support::Bearing::new(self.volume, 0);
        self
    }
    #[cfg(test)]
    pub(crate) fn counts(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.reads,
            self.misses,
            self.clearance.reads,
            self.clearance.misses,
            self.bearing.reads,
            self.bearing.misses,
        )
    }
    #[cfg(test)]
    pub(super) fn retained(&self) -> usize {
        self.entries.len()
    }
    pub(crate) fn new(volume: &'a Volume, document: &'a BodyDocument, blocked: &'a [Box3]) -> Self {
        Self {
            volume,
            document,
            blocked,
            body: Body::new(document),
            clearance: narrow::Clearance::new(volume, 16384),
            bearing: support::Bearing::new(volume, 4096),
            entries: BTreeMap::new(),
            order: VecDeque::new(),
            reads: 0,
            misses: 0,
            #[cfg(test)]
            capacity: CAPACITY,
        }
    }
    pub(crate) fn step(
        &mut self,
        from: &Pose,
        to: &Pose,
        rules: Option<Walking>,
    ) -> Result<Option<Stride>> {
        if let Some(rules) =
            rules.filter(|r| !from.feet.is_empty() && (from.yaw == to.yaw || r.turns))
        {
            return plan_with(self, from, to, rules).map(Some);
        }
        sweep_clear(self.volume, self.document, from, to, self.blocked)?;
        Ok(None)
    }
    pub(super) fn evaluate(&mut self, frame: &StrideFrame, yaw: Yaw) -> Evaluation {
        self.reads += 1;
        #[cfg(test)]
        if self.capacity == 0 {
            self.misses += 1;
            return self.read(frame, yaw);
        }
        let key = (
            yaw,
            frame.root,
            frame.turn,
            frame.feet.iter().map(|(&id, &p)| (id, p)).collect(),
            frame.bearing.iter().copied().collect(),
        );
        if let Some(found) = self.entries.get(&key) {
            return found.clone();
        }
        self.misses += 1;
        let result = self.read(frame, yaw);
        #[cfg(test)]
        let capacity = self.capacity;
        #[cfg(not(test))]
        let capacity = CAPACITY;
        if capacity > 0 {
            if self.entries.len() >= capacity {
                self.entries.remove(&self.order.pop_front().unwrap());
            }
            self.order.push_back(key.clone());
            self.entries.insert(key, result.clone());
        }
        result
    }
    fn read(&mut self, frame: &StrideFrame, yaw: Yaw) -> Evaluation {
        let feet = local_feet(frame, yaw)?;
        let rotation = Rotation::at(yaw, frame.turn);
        let (occupied, skins) = self.body.posed(frame.root, rotation, &feet)?;
        // Count every sampled box even on a cache hit. Reuse must never enlarge
        // the cumulative swing work budget or alter its refusal precedence.
        let cells = occupied.iter().try_fold(0u64, |sum, [lo, hi]| {
            let n = (0..3)
                .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
                .ok_or("stride work overflow")?;
            sum.checked_add(n)
                .ok_or("whole swing work budget exceeded".into())
        });
        let clear = (|| {
            if intersects(&occupied, self.blocked) {
                return Err("swing intersects another native body".into());
            }
            let shapes = narrow::shapes(self.body.chains.iter().zip(&skins), frame.root, rotation)?;
            self.clearance.check(
                self.body
                    .rest
                    .as_ref()
                    .map_err(Clone::clone)?
                    .iter()
                    .map(|(id, _)| *id)
                    .zip(occupied.iter().copied()),
                &shapes,
            )?;
            let centre = rotation
                .rotate(self.body.centre)
                .ok_or("turn centre overflow")?;
            let hull = self.bearing.hull(&shapes.pads, &frame.bearing)?;
            if !polygon::contains(
                &hull,
                [centre[0] + frame.root[0], centre[2] + frame.root[2]],
            ) {
                return Err("body volume centre leaves its bearing-foot polygon".into());
            }
            Ok(())
        })();
        Ok((cells, clear))
    }
}
impl Drop for Checks<'_> {
    fn drop(&mut self) {
        if self.reads > 0 {
            crate::profile::counts(
                "walking.postures",
                &[
                    ("reads", self.reads),
                    ("evaluated", self.misses),
                    ("retained", self.entries.len()),
                    ("parts", self.clearance.reads),
                    ("parts_evaluated", self.clearance.misses),
                    ("supports", self.bearing.reads),
                    ("supports_evaluated", self.bearing.misses),
                ],
            );
        }
    }
}

pub(super) fn boxes_at(
    document: &BodyDocument,
    origin: [i64; 3],
    rotation: Rotation,
    feet: &BTreeMap<PartId, [i64; 3]>,
) -> Result<Vec<Box3>> {
    Body::new(document)
        .posed(origin, rotation, feet)
        .map(|(boxes, _)| boxes)
}
