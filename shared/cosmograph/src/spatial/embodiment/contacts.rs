// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Contact witnesses and conservative sweeps over admitted native part boxes.
use super::*;
use isometer_core::PartId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    pub part: PartId,
    /// Bottom face of the part, above this column's solid supporting cell.
    pub at: [i64; 3],
}

/// One deterministic contact per supported part. This reads the same terrain,
/// half-open boxes and water policy as placement, including edited columns.
pub fn contacts(volume: &Volume, document: &BodyDocument, pose: &Pose) -> Result<Vec<Contact>> {
    let occupied = boxes(document, pose)?;
    let pads = if pose.feet.is_empty() {
        BTreeMap::new()
    } else {
        super::narrow::pads(
            document,
            pose.at.map(|v| v * isometer_core::legs::UNIT),
            pose.yaw,
            &pose.feet,
        )?
    };
    let mut out = Vec::new();
    for (part, [lo, hi]) in document
        .living()
        .filter(|p| p.half_extent.iter().all(|h| *h > 0))
        .zip(occupied)
    {
        let [lo, hi] = if pose.feet.is_empty() {
            [lo, hi]
        } else {
            let Some(bounds) = pads.get(&part.id) else {
                continue;
            };
            *bounds
        };
        let mut best = None;
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                if !supported(
                    volume,
                    [x, lo[1].checked_sub(1).ok_or("support overflow")?, z],
                ) {
                    continue;
                }
                // Doubled cell-centre distance, entirely integer and stable.
                let dx = i128::from(x) * 2 + 1 - i128::from(lo[0]) - i128::from(hi[0]);
                let dz = i128::from(z) * 2 + 1 - i128::from(lo[2]) - i128::from(hi[2]);
                let candidate = (dx * dx + dz * dz, [x, lo[1], z]);
                if best.as_ref().is_none_or(|old| candidate < *old) {
                    best = Some(candidate);
                }
            }
        }
        if let Some((_, at)) = best {
            out.push(Contact { part: part.id, at });
        }
    }
    Ok(out)
}

pub(super) fn supported(volume: &Volume, at: [i64; 3]) -> bool {
    volume
        .material(at)
        .is_some_and(|m| m != volume.materials.air && m != volume.materials.water)
}

/// A deliberately conservative translation envelope. A visual interpolation
/// may use it only after a native step was accepted; this admits no new step.
/// The union box includes every fractional pose, including a one-cell climb.
pub fn sweep_clear(
    volume: &Volume,
    document: &BodyDocument,
    from: &Pose,
    to: &Pose,
    blocked: &[Box3],
) -> Result<()> {
    if !from.feet.is_empty() && !to.feet.is_empty() {
        check(volume, document, from)?;
        check(volume, document, to)?;
        if (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1) {
            return Err("swing needs adjacent roots".into());
        }
        if from.yaw != to.yaw {
            return super::swing::check_clear(
                volume,
                &super::swing::turn_clear(volume, document, from, to)?,
                blocked,
            );
        }
        let mut work = 0u64;
        for sample in 0..=super::swing::SAMPLES {
            let phase = sample * isometer_core::legs::UNIT / super::swing::SAMPLES;
            let occupied = super::swing::swing_boxes(document, from, to, phase)?;
            for [lo, hi] in &occupied {
                let count = (0..3)
                    .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
                    .ok_or("swing work overflow")?;
                work = work
                    .checked_add(count)
                    .filter(|n| *n <= 4_194_304)
                    .ok_or("whole swing work budget exceeded")?;
            }
            let origin = [0, 1, 2]
                .map(|k| from.at[k] * isometer_core::legs::UNIT + (to.at[k] - from.at[k]) * phase);
            let feet = super::swing::swing_feet(document, from, to, phase)?;
            super::narrow::clear(
                volume, document, origin, from.yaw, &feet, &occupied, blocked,
            )
            .map_err(|why| format!("swing sample {sample}: {why}"))?;
            super::swing::support_sample(volume, document, from, to, phase, &occupied)?;
        }
        return Ok(());
    }
    if from.yaw != to.yaw || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1) {
        return Err("sweep needs adjacent untranslated frames".into());
    }
    let a = boxes(document, from)?;
    let b = boxes(document, to)?;
    let mut work = 0u64;
    for ([alo, ahi], [blo, bhi]) in a.into_iter().zip(b) {
        let lo = [0, 1, 2].map(|k| alo[k].min(blo[k]));
        let hi = [0, 1, 2].map(|k| ahi[k].max(bhi[k]));
        let n = (0..3)
            .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
            .ok_or("sweep work overflow")?;
        work = work
            .checked_add(n)
            .filter(|n| *n <= 1_048_576)
            .ok_or("sweep work budget exceeded")?;
        if intersects(&[[lo, hi]], blocked) {
            return Err("sweep intersects another body".into());
        }
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                for y in lo[1]..hi[1] {
                    if !air(volume, [x, y, z]) {
                        return Err("sweep intersects terrain or water".into());
                    }
                }
            }
        }
    }
    Ok(())
}
