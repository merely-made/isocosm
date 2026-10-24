// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Support-guided transfers. The accepted explicit plan, not a second display
//! planner, determines body position, foot order and clearance.
use super::*;
use isometer_core::{
    PartId,
    legs::{self, Stage, Stride, StrideFrame, UNIT, support as polygon},
};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Walking {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub clearance: bool,
    /// Maximum root deviation around the two admitted endpoint roots, in cells.
    pub shift: u8,
    /// Radius searched around an anatomical sole, in cells.
    pub foot_search: u8,
    /// Maximum sole lift above the higher endpoint, in cells.
    pub lift: u8,
    /// Bounded candidate postures per grounding or planned step.
    pub postures: u16,
}
impl Default for Walking {
    fn default() -> Self {
        Self {
            clearance: true,
            shift: 4,
            foot_search: 2,
            lift: 3,
            postures: 256,
        }
    }
}
impl Walking {
    pub fn valid(self) -> bool {
        self.shift <= 4
            && self.foot_search <= 3
            && (1..=4).contains(&self.lift)
            && (1..=1024).contains(&self.postures)
    }
}
pub fn walking_rules(sim: &Simulation) -> Option<Walking> {
    sim.genesis()
        .rules
        .spatial_templates
        .as_ref()
        .and_then(|t| t.walking)
}
pub fn local_feet(frame: &StrideFrame, yaw: Yaw) -> Result<BTreeMap<PartId, [i64; 3]>> {
    frame
        .feet
        .iter()
        .map(|(&id, &p)| {
            Ok((
                id,
                super::super::birth::rotate(
                    swing::inverse(yaw),
                    [0, 1, 2].map(|k| p[k] - frame.root[k]),
                )?,
            ))
        })
        .collect()
}
pub fn stride_sample(from: &Pose, to: &Pose, stride: &Stride, phase: i64) -> Result<StrideFrame> {
    if from.yaw != to.yaw {
        return Err("weight transfer needs matching body frames".into());
    }
    stride
        .sample(
            from.at.map(|v| v * UNIT),
            to.at.map(|v| v * UNIT),
            &foot_points(from)?,
            &foot_points(to)?,
            phase,
        )
        .ok_or("invalid explicit foot trajectory".into())
}
pub fn stride_boxes(
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    stride: &Stride,
    phase: i64,
) -> Result<Vec<Box3>> {
    let frame = stride_sample(from, to, stride, phase)?;
    boxes_at(
        document,
        frame.root,
        from.yaw,
        &local_feet(&frame, from.yaw)?,
    )
}
fn boxes_at(
    document: &BodyDocument,
    origin: [i64; 3],
    yaw: Yaw,
    feet: &BTreeMap<PartId, [i64; 3]>,
) -> Result<Vec<Box3>> {
    let mut replacements = BTreeMap::new();
    for leg in legs::chains(document) {
        for (id, bounds) in leg
            .occupied(*feet.get(&leg.foot).ok_or("planned foot absent")?)
            .ok_or("weight transfer exceeds paid reach")?
        {
            replacements.insert(id, swing::placed(bounds, yaw, origin)?);
        }
    }
    let rest = boxes(
        document,
        &Pose {
            yaw,
            ..Default::default()
        },
    )?;
    document
        .living()
        .filter(|p| p.half_extent.iter().all(|h| *h > 0))
        .zip(rest)
        .map(|(part, bounds)| {
            if let Some(b) = replacements.get(&part.id) {
                return Ok(*b);
            }
            let b = bounds.map(|p| [0, 1, 2].map(|k| p[k] * UNIT + origin[k]));
            Ok([
                b[0].map(|v| v.div_euclid(UNIT)),
                b[1].map(|v| -(-v).div_euclid(UNIT)),
            ])
        })
        .collect()
}
fn samples(
    volume: &Volume,
    document: &BodyDocument,
    yaw: Yaw,
    stride: &Stride,
    a: [i64; 3],
    b: [i64; 3],
    from: &BTreeMap<PartId, [i64; 3]>,
    to: &BTreeMap<PartId, [i64; 3]>,
    blocked: &[Box3],
    count: i64,
) -> Result<()> {
    let mut work = 0u64;
    for sample in 0..=count {
        let frame = stride
            .sample(a, b, from, to, sample * UNIT / count)
            .ok_or("invalid planned stage")?;
        let feet = local_feet(&frame, yaw)?;
        let occupied = boxes_at(document, frame.root, yaw, &feet)?;
        for [lo, hi] in &occupied {
            let n = (0..3)
                .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
                .ok_or("stride work overflow")?;
            work = work
                .checked_add(n)
                .filter(|n| *n <= 4_194_304)
                .ok_or("whole swing work budget exceeded")?;
        }
        narrow::clear(volume, document, frame.root, yaw, &feet, &occupied, blocked)
            .and_then(|_| support::check(volume, document, &feet, frame.root, yaw, &frame.bearing))
            .map_err(|why| format!("weight-transfer sample {sample}: {why}"))?;
    }
    Ok(())
}
pub fn check_stride(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Walking,
    stride: &Stride,
    blocked: &[Box3],
) -> Result<()> {
    let _span = crate::profile::Span::new("walking.check");
    if stride.clearance != rules.clearance
        || !rules.valid()
        || from.yaw != to.yaw
        || from.feet.is_empty()
        || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1)
    {
        return Err("weight transfer differs from its bounded walking policy".into());
    }
    check(volume, document, from)?;
    check(volume, document, to)?;
    let a = from.at.map(|v| v * UNIT);
    let b = to.at.map(|v| v * UNIT);
    let feet = foot_points(from)?;
    let end = foot_points(to)?;
    let moved: BTreeSet<_> = feet
        .iter()
        .filter(|(id, p)| end.get(id) != Some(*p))
        .map(|(id, _)| *id)
        .collect();
    let ids: BTreeSet<_> = stride.stages.iter().map(|s| s.foot).collect();
    if ids != moved || ids.len() != stride.stages.len() || stride.stages.len() > 64 {
        return Err("planned foot order differs from actual displaced soles".into());
    }
    let reach = i64::from(rules.shift) * UNIT;
    for s in &stride.stages {
        if s.lift <= 0
            || s.lift > i64::from(rules.lift) * UNIT
            || (0..3).any(|k| {
                s.root[k] < a[k].min(b[k]) - if k == 1 { 0 } else { reach }
                    || s.root[k] > a[k].max(b[k]) + reach
            })
        {
            return Err("planned body shift or foot lift exceeds its declared bound".into());
        }
    }
    samples(
        volume,
        document,
        from.yaw,
        stride,
        a,
        b,
        &feet,
        &end,
        blocked,
        stride.samples(),
    )
}

/// Find transfers using the real support polygon, then validate the complete
/// explicit trajectory at every phase the renderer can display.
pub fn plan_stride(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Walking,
    blocked: &[Box3],
) -> Result<Stride> {
    let _span = crate::profile::Span::new("walking.plan");
    if !rules.valid()
        || from.yaw != to.yaw
        || from.feet.is_empty()
        || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1)
    {
        return Err("weight transfer needs a bounded adjacent articulated step".into());
    }
    check(volume, document, from)?;
    check(volume, document, to)?;
    let start = from.at.map(|v| v * UNIT);
    let finish = to.at.map(|v| v * UNIT);
    let first = foot_points(from)?;
    let end = foot_points(to)?;
    let mut feet = first.clone();
    let mut root = start;
    let mut stages = Vec::new();
    let mut remaining: BTreeSet<_> = feet
        .iter()
        .filter(|(id, p)| end.get(id) != Some(*p))
        .map(|(id, _)| *id)
        .collect();
    let total = remaining.len();
    let centre = support::centre(document, from.yaw)?;
    let mut tried = 0usize;
    while !remaining.is_empty() {
        let nominal = [0, 1, 2].map(|k| {
            if k == 1 {
                start[k].max(finish[k])
            } else {
                start[k] + (finish[k] - start[k]) * (stages.len() as i64 + 1) / (total as i64 + 1)
            }
        });
        let current = StrideFrame {
            root,
            feet: feet.clone(),
            bearing: feet.keys().copied().collect(),
        };
        let local = local_feet(&current, from.yaw)?;
        let mut candidates = Vec::new();
        for &foot in &remaining {
            let bearing = feet.keys().copied().filter(|id| *id != foot).collect();
            let Ok(poly) = support::bearing(volume, document, &local, root, from.yaw, &bearing)
            else {
                continue;
            };
            let Some(at) = polygon::balance(
                &poly,
                [nominal[0] + centre[0], nominal[2] + centre[2]],
                UNIT / 4,
            ) else {
                continue;
            };
            let at = [at[0] - centre[0], nominal[1], at[1] - centre[2]];
            let reach = i64::from(rules.shift) * UNIT;
            if [0, 2].iter().any(|&k| {
                at[k] < start[k].min(finish[k]) - reach || at[k] > start[k].max(finish[k]) + reach
            }) {
                continue;
            }
            let cost = at[0].abs_diff(nominal[0]) + at[2].abs_diff(nominal[2]);
            candidates.push((cost, foot, at));
        }
        candidates.sort();
        let mut found = None;
        let mut why = "no bearing-foot polygon within declared body shift".to_string();
        let rise_limit = if rules.clearance {
            rules.shift
        } else {
            rules.shift.min(1)
        };
        'candidate: for (_, foot, at) in candidates {
            for rise in 0..=i64::from(rise_limit) {
                for lift in 1..=i64::from(rules.lift) {
                    tried += 1;
                    if tried > usize::from(rules.postures) {
                        return Err("walking stage candidate budget exhausted".into());
                    }
                    let mut at = at;
                    at[1] += rise * UNIT;
                    let stage = Stage {
                        foot,
                        root: at,
                        lift: lift * UNIT,
                    };
                    let single = Stride {
                        clearance: rules.clearance,
                        stages: vec![stage.clone()],
                    };
                    let mut after = feet.clone();
                    after.insert(foot, end[&foot]);
                    match samples(
                        volume, document, from.yaw, &single, root, at, &feet, &after, blocked, 16,
                    ) {
                        Ok(()) => {
                            found = Some((stage, after));
                            break 'candidate;
                        },
                        Err(error) => why = error,
                    }
                }
            }
        }
        let (stage, after) = found.ok_or(why)?;
        remaining.remove(&stage.foot);
        root = stage.root;
        feet = after;
        stages.push(stage);
    }
    let stride = Stride {
        stages,
        clearance: rules.clearance,
    };
    check_stride(volume, document, from, to, rules, &stride, blocked)?;
    Ok(stride)
}

pub fn check_walk(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Option<Walking>,
    stride: Option<&Stride>,
    blocked: &[Box3],
) -> Result<()> {
    if let Some(rules) = rules.filter(|_| !from.feet.is_empty() && from.yaw == to.yaw) {
        return check_stride(
            volume,
            document,
            from,
            to,
            rules,
            stride.ok_or("walking step lacks its admitted weight-transfer plan")?,
            blocked,
        );
    }
    if stride.is_some() {
        return Err("weight-transfer plan has no matching walking policy".into());
    }
    sweep_clear(volume, document, from, to, blocked)
}

pub fn step_stride(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Option<Walking>,
    blocked: &[Box3],
) -> Result<Option<Stride>> {
    if let Some(rules) = rules.filter(|_| !from.feet.is_empty() && from.yaw == to.yaw) {
        return plan_stride(volume, document, from, to, rules, blocked).map(Some);
    }
    sweep_clear(volume, document, from, to, blocked)?;
    Ok(None)
}
