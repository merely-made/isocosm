// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded geometric walking sweep. Soles lift before crossing a riser;
//! endpoints and links come from the same paid-chain solver as admission.
use super::*;
use isometer_core::{
    PartId,
    legs::{self, UNIT},
};

pub const SAMPLES: i64 = 128;

pub fn swing_boxes(
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    phase: i64,
) -> Result<Vec<Box3>> {
    if from.yaw != to.yaw || from.feet.is_empty() || phase < 0 || phase > UNIT {
        return Err("articulated sweep needs matching live chain frames".into());
    }
    let chains = legs::chains(document);
    let origin = [0, 1, 2].map(|k| from.at[k] * UNIT + (to.at[k] - from.at[k]) * phase);
    let feet = swing_feet(document, from, to, phase)?;
    let mut replacements = BTreeMap::new();
    for leg in chains {
        for (id, bounds) in leg
            .occupied(feet[&leg.foot])
            .ok_or("swing leaves paid chain reach")?
        {
            replacements.insert(id, placed(bounds, from.yaw, origin)?);
        }
    }
    let rest = boxes(
        document,
        &Pose {
            at: [0; 3],
            yaw: from.yaw,
            ..Default::default()
        },
    )?;
    let mut out = Vec::new();
    for (part, bounds) in document
        .living()
        .filter(|p| p.half_extent.iter().all(|h| *h > 0))
        .zip(rest)
    {
        out.push(if let Some(b) = replacements.get(&part.id) {
            *b
        } else {
            let fixed = bounds.map(|p| [0, 1, 2].map(|k| p[k] * UNIT + origin[k]));
            rounded(fixed)
        });
    }
    Ok(out)
}

/// One foot moves at a time; all other soles remain fixed in world space.
/// Native clearance and the displayed gait consume these exact targets.
pub fn swing_feet(
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    phase: i64,
) -> Result<BTreeMap<PartId, [i64; 3]>> {
    if from.yaw != to.yaw || from.feet.is_empty() || !(0..=UNIT).contains(&phase) {
        return Err("foot swing needs matching articulated frames".into());
    }
    if [from, to].iter().any(|p| {
        p.at.iter()
            .any(|v| v.unsigned_abs() > (i64::MAX / UNIT / 2) as u64)
            || p.feet
                .values()
                .flatten()
                .any(|v| v.unsigned_abs() > (1u64 << 42))
    }) || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1)
    {
        return Err("foot swing exceeds bounded coordinates or adjacent roots".into());
    }
    let chains = legs::chains(document);
    let mut out = BTreeMap::new();

    for (index, leg) in chains.iter().enumerate() {
        let foot_phase = (phase * chains.len() as i64 - index as i64 * UNIT).clamp(0, UNIT);
        let a = *from.feet.get(&leg.foot).ok_or("source foot absent")?;
        let b = *to.feet.get(&leg.foot).ok_or("destination foot absent")?;
        let delta = super::super::birth::rotate(
            inverse(from.yaw),
            [0, 1, 2].map(|k| (to.at[k] - from.at[k]) * UNIT),
        )?;
        let end = [0, 1, 2].map(|k| b[k] + delta[k]);
        if (0..3).any(|k| a[k].abs_diff(end[k]) > (4 * UNIT) as u64) {
            return Err("foot swing exceeds bounded local step".into());
        }
        // Three phases give vertical clearance at both ends of a voxel riser.
        let travel = ((foot_phase - UNIT / 4) * 2).clamp(0, UNIT);
        let height = a[1].max(end[1]) + UNIT;
        let y = if foot_phase < UNIT / 4 {
            a[1] + (height - a[1]) * foot_phase * 4 / UNIT
        } else if foot_phase > UNIT * 3 / 4 {
            end[1] + (height - end[1]) * (UNIT - foot_phase) * 4 / UNIT
        } else {
            height
        };
        let absolute = [
            a[0] + (end[0] - a[0]) * travel / UNIT,
            y,
            a[2] + (end[2] - a[2]) * travel / UNIT,
        ];
        let root_delta = super::super::birth::rotate(
            inverse(from.yaw),
            [0, 1, 2].map(|k| (to.at[k] - from.at[k]) * phase),
        )?;
        let sole = [0, 1, 2].map(|k| absolute[k] - root_delta[k]);
        out.insert(leg.foot, sole);
    }
    Ok(out)
}

pub(super) fn support_sample(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    phase: i64,
    _occupied: &[Box3],
) -> Result<()> {
    let chains = legs::chains(document);
    let bearing = chains
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let t = (phase * chains.len() as i64 - i as i64 * UNIT).clamp(0, UNIT);
            (t == 0 || t == UNIT).then_some(l.foot)
        })
        .collect();
    let origin = [0, 1, 2].map(|k| from.at[k] * UNIT + (to.at[k] - from.at[k]) * phase);
    super::support::check(
        volume,
        document,
        &swing_feet(document, from, to, phase)?,
        origin,
        from.yaw,
        &bearing,
    )
}

pub(super) fn inverse(yaw: Yaw) -> Yaw {
    match yaw {
        Yaw::Zero => Yaw::Zero,
        Yaw::Quarter => Yaw::ThreeQuarter,
        Yaw::Half => Yaw::Half,
        Yaw::ThreeQuarter => Yaw::Quarter,
    }
}
pub(super) fn placed(bounds: [[i64; 3]; 2], yaw: Yaw, origin: [i64; 3]) -> Result<Box3> {
    let a = super::super::birth::rotate(yaw, bounds[0])?;
    let b = super::super::birth::rotate(yaw, bounds[1])?;
    Ok(rounded([
        [0, 1, 2].map(|k| a[k].min(b[k]) + origin[k]),
        [0, 1, 2].map(|k| a[k].max(b[k]) + origin[k]),
    ]))
}
fn rounded(bounds: [[i64; 3]; 2]) -> Box3 {
    [
        bounds[0].map(|v| v.div_euclid(UNIT)),
        bounds[1].map(|v| -(-v).div_euclid(UNIT)),
    ]
}

pub(super) fn turn_clear(
    volume: &Volume,
    doc: &BodyDocument,
    from: &Pose,
    to: &Pose,
) -> Result<Vec<Box3>> {
    let occupied = boxes(doc, from)?
        .into_iter()
        .chain(boxes(doc, to)?)
        .collect::<Vec<_>>();
    let mut radius = 0i64;
    let mut lo_y = i64::MAX;
    let mut hi_y = i64::MIN;
    for [lo, hi] in occupied {
        for x in [lo[0], hi[0]] {
            for z in [lo[2], hi[2]] {
                let dx = i128::from(x - from.at[0]);
                let dz = i128::from(z - from.at[2]);
                radius = radius.max(((dx * dx + dz * dz) as u128).isqrt() as i64 + 1);
            }
        }
        lo_y = lo_y.min(lo[1]);
        hi_y = hi_y.max(hi[1]);
    }
    let bounds = [
        [from.at[0] - radius, lo_y, from.at[2] - radius],
        [from.at[0] + radius, hi_y, from.at[2] + radius],
    ];
    check_clear(volume, &[bounds], &[])?;
    Ok(vec![bounds])
}

pub(super) fn check_clear(volume: &Volume, boxes: &[Box3], blocked: &[Box3]) -> Result<()> {
    let mut work = 0u64;
    if intersects(boxes, blocked) {
        return Err("swing intersects another native body".into());
    }
    for [lo, hi] in boxes {
        let count = (0..3)
            .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
            .ok_or("sweep work overflow")?;
        work = work
            .checked_add(count)
            .filter(|n| *n <= 1_048_576)
            .ok_or("sweep work budget exceeded")?;
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                for y in lo[1]..hi[1] {
                    if !volume.holds(x, z) || !air(volume, [x, y, z]) {
                        return Err("swing intersects terrain or water".into());
                    }
                }
            }
        }
    }
    Ok(())
}
