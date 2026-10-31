// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Derived posture work lives only as long as its immutable physical inputs.
use super::*;
use crate::profile::Clock;

const CAPACITY: usize = 8192;
type Key = (Yaw, [i64; 3], i64, Vec<(PartId, [i64; 3])>, Vec<PartId>);
type Evaluation = Result<(Result<u64>, Result<()>)>;
type Stand = ([i64; 3], Yaw, Vec<(PartId, [i64; 3])>);

pub(crate) struct Checks<'a> {
    pub(super) volume: &'a Volume,
    pub(super) document: &'a BodyDocument,
    pub(super) blocked: &'a [Box3],
    body: Body,
    clearance: narrow::Clearance<'a>,
    bearing: support::Bearing<'a>,
    entries: Memo<Key, Evaluation>,
    residents: Memo<Box3, bool>,
    admissions: Memo<Stand, Result<()>>,
    pub(in crate::spatial::embodiment) grounding: Grounding<'a>,
    resident_reads: usize,
    resident_misses: usize,
    admission_reads: usize,
    admission_misses: usize,
    pub(super) reads: usize,
    pub(super) misses: usize,
    times: [usize; 7],
}
struct Body {
    balance: Option<Balance>,
    chains: Vec<legs::Leg>,
    rest: Result<Vec<(PartId, Box3)>>,
    centre: [i64; 3],
}
impl Body {
    fn new(document: &BodyDocument) -> Self {
        Self {
            balance: None,
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
    pub(crate) fn with_balance(mut self, balance: Option<Balance>) -> Self {
        self.entries.clear();
        self.admissions.clear();
        self.body.balance = balance.clone();
        self.grounding.set_balance(balance);
        self
    }
    pub(super) fn centre(
        &self,
        feet: &BTreeMap<PartId, [i64; 3]>,
        rotation: Rotation,
    ) -> Result<[i64; 3]> {
        match &self.body.balance {
            Some(balance) => balance.at(self.document, feet, rotation),
            None => rotation
                .rotate(self.body.centre)
                .ok_or("turn centre overflow".into()),
        }
    }
    pub(crate) fn check_walk(
        &mut self,
        from: &Pose,
        to: &Pose,
        rules: Option<Walking>,
        stride: Option<&Stride>,
    ) -> Result<()> {
        if let Some(rules) =
            rules.filter(|r| !from.feet.is_empty() && (from.yaw == to.yaw || r.turns))
        {
            return check_stride_with(
                self,
                from,
                to,
                rules,
                stride.ok_or("walking step lacks its admitted weight-transfer plan")?,
            );
        }
        if stride.is_some() {
            return Err("weight-transfer plan has no matching walking policy".into());
        }
        sweep_clear(self.volume, self.document, from, to, self.blocked)
    }
    #[cfg(test)]
    pub(crate) fn without_reuse(mut self) -> Self {
        self.entries.set_capacity(0);
        self.residents.set_capacity(0);
        self.admissions.set_capacity(0);
        self.clearance = narrow::Clearance::new(self.volume, 0);
        self.bearing = support::Bearing::new(self.volume, 0);
        self
    }
    #[cfg(test)]
    pub(super) fn set_capacity(&mut self, capacity: usize) {
        self.entries.set_capacity(capacity);
        self.residents.set_capacity(capacity);
        self.admissions.set_capacity(capacity);
    }
    #[cfg(test)]
    pub(super) fn reuse_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.resident_reads,
            self.resident_misses,
            self.admission_reads,
            self.admission_misses,
        )
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
            entries: Memo::new(CAPACITY),
            residents: Memo::new(16384),
            admissions: Memo::new(8192),
            grounding: Grounding::new(volume),
            resident_reads: 0,
            resident_misses: 0,
            admission_reads: 0,
            admission_misses: 0,
            reads: 0,
            misses: 0,
            times: [0; 7],
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
    pub(crate) fn neighbours(
        &mut self,
        places: &Places,
        profile: &StepProfile,
        local: &[Box3],
        pose: &Pose,
        rules: Option<Walking>,
    ) -> Vec<Pose> {
        self.grounding.set_walking(rules);
        if !pose.feet.is_empty() {
            articulated_neighbours(places, profile, self.document, pose, self)
        } else {
            neighbours_from_boxes(places, profile, local, pose, &mut self.grounding)
        }
    }
    /// Arrival selection reads terrain alone. Residents remain a separate
    /// route-node/stride check after the original stance truncation.
    pub(in crate::spatial::embodiment) fn admit(&mut self, pose: &Pose) -> Result<()> {
        self.admission_reads += 1;
        if !self.admissions.enabled() {
            self.admission_misses += 1;
            return check_balanced(self.volume, self.document, pose, self.body.balance.as_ref());
        }
        let key = (
            pose.at,
            pose.yaw,
            pose.feet.iter().map(|(&id, &p)| (id, p)).collect(),
        );
        if let Some(found) = self.admissions.get(&key) {
            return found.clone();
        }
        self.admission_misses += 1;
        let result = self.admit_parts(pose);
        self.admissions.insert(key, result.clone());
        result
    }
    fn admit_parts(&mut self, pose: &Pose) -> Result<()> {
        // Retain native box/sole validation and its work bounds. Only the
        // repeated narrow and support readings use the prepared components.
        let occupied = boxes(self.document, pose)?;
        if pose.feet.is_empty() {
            return check_occupied(self.volume, occupied);
        }
        let origin = pose.at.map(|v| v * UNIT);
        let rotation = Rotation::at(pose.yaw, 0);
        let skins = self
            .body
            .chains
            .iter()
            .map(|leg| {
                leg.skin(*pose.feet.get(&leg.foot).ok_or("admitted foot absent")?)
                    .ok_or("unreachable paid links".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let shapes = narrow::shapes(self.body.chains.iter().zip(&skins), origin, rotation)?;
        self.clearance.check(
            self.document
                .living()
                .filter(|p| p.half_extent.iter().all(|h| *h > 0))
                .map(|p| p.id)
                .zip(occupied),
            &shapes,
        )?;
        self.supported(
            origin,
            rotation,
            &skins,
            &shapes.pads,
            &pose.feet.keys().copied().collect(),
        )
    }
    fn supported(
        &mut self,
        origin: [i64; 3],
        rotation: Rotation,
        skins: &[legs::Skin],
        pads: &BTreeMap<PartId, narrow::Pad>,
        bearing: &BTreeSet<PartId>,
    ) -> Result<()> {
        let centre = match &self.body.balance {
            Some(balance) => rotation
                .rotate(balance.centre(self.body.chains.iter().zip(skins))?)
                .ok_or("posed centre rotation overflow")?,
            None => rotation
                .rotate(self.body.centre)
                .ok_or("turn centre overflow")?,
        };
        let hull = self.bearing.hull(pads, bearing)?;
        if !polygon::contains(&hull, [centre[0] + origin[0], centre[2] + origin[2]]) {
            return Err(if self.body.balance.is_some() {
                "posed matter centre leaves its bearing-foot polygon"
            } else {
                "body volume centre leaves its bearing-foot polygon"
            }
            .into());
        }
        Ok(())
    }
    pub(super) fn intersects_residents(&mut self, occupied: &[Box3]) -> bool {
        for &bounds in occupied {
            self.resident_reads += 1;
            if let Some(&hit) = self.residents.get(&bounds) {
                if hit {
                    return true;
                }
                continue;
            }
            self.resident_misses += 1;
            let hit = intersects(&[bounds], self.blocked);
            self.residents.insert(bounds, hit);
            if hit {
                return true;
            }
        }
        false
    }
    pub(super) fn evaluate(&mut self, frame: &StrideFrame, yaw: Yaw) -> Evaluation {
        let key_time = Clock::new();
        self.reads += 1;
        if !self.entries.enabled() {
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
            self.times[0] += key_time.ns();
            return found.clone();
        }
        self.times[0] += key_time.ns();
        self.misses += 1;
        let result = self.read(frame, yaw);
        let write_time = Clock::new();
        self.entries.insert(key, result.clone());
        self.times[1] += write_time.ns();
        result
    }
    fn read(&mut self, frame: &StrideFrame, yaw: Yaw) -> Evaluation {
        let pose_time = Clock::new();
        let posed: Result<_> = (|| {
            let feet = local_feet(frame, yaw)?;
            let rotation = Rotation::at(yaw, frame.turn);
            let (occupied, skins) = self.body.posed(frame.root, rotation, &feet)?;
            Ok((rotation, occupied, skins))
        })();
        self.times[2] += pose_time.ns();
        let (rotation, occupied, skins) = posed?;
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
            let time = Clock::new();
            let blocked = self.intersects_residents(&occupied);
            self.times[3] += time.ns();
            if blocked {
                return Err("swing intersects another native body".into());
            }
            let time = Clock::new();
            let shapes = narrow::shapes(self.body.chains.iter().zip(&skins), frame.root, rotation);
            self.times[4] += time.ns();
            let shapes = shapes?;
            let time = Clock::new();
            let cleared = self.clearance.check(
                self.body
                    .rest
                    .as_ref()
                    .map_err(Clone::clone)?
                    .iter()
                    .map(|(id, _)| *id)
                    .zip(occupied.iter().copied()),
                &shapes,
            );
            self.times[5] += time.ns();
            cleared?;
            let time = Clock::new();
            let supported =
                self.supported(frame.root, rotation, &skins, &shapes.pads, &frame.bearing);
            self.times[6] += time.ns();
            supported
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
                    ("resident_reads", self.resident_reads),
                    ("resident_evaluated", self.resident_misses),
                ],
            );
            crate::profile::counts(
                "walking.cost",
                &[
                    ("key_ns", self.times[0]),
                    ("memo_ns", self.times[1]),
                    ("pose_ns", self.times[2]),
                    ("blockers_ns", self.times[3]),
                    ("shapes_ns", self.times[4]),
                    ("terrain_ns", self.times[5]),
                    ("support_ns", self.times[6]),
                ],
            );
        }
        if self.admission_reads > 0 {
            crate::profile::counts(
                "walking.arrivals",
                &[
                    ("reads", self.admission_reads),
                    ("evaluated", self.admission_misses),
                    ("retained", self.admissions.len()),
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
