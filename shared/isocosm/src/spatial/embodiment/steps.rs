// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_core::legs::Rotation;

pub(crate) fn articulated_neighbours(
    places: &Places,
    profile: &StepProfile,
    document: &BodyDocument,
    pose: &Pose,
    grounding: &mut Grounding<'_>,
) -> Vec<Pose> {
    grounding.articulated = true;
    let Some(from) = places.place_at(pose.at) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let turning = grounding.walking.is_some_and(|r| r.turns);
    // The axial graph's forward direction is -Z. Try facing the actual step,
    // then retain an admitted sidestep if a whole-body turn cannot fit.
    let mut directions = vec![
        (0, -1, Yaw::Zero),
        (-1, 0, Yaw::Quarter),
        (0, 1, Yaw::Half),
        (1, 0, Yaw::ThreeQuarter),
    ];
    if turning {
        use Yaw::*;
        let (left, right) = match pose.yaw {
            Zero => (Quarter, ThreeQuarter),
            Quarter => (Half, Zero),
            Half => (ThreeQuarter, Quarter),
            ThreeQuarter => (Zero, Half),
        };
        directions.extend([(0, 0, left), (0, 0, right)]);
    }
    for (dx, dz, facing) in directions {
        for yaw in [facing, pose.yaw] {
            if (dx == 0 && dz == 0 && yaw == pose.yaw)
                || (turning && yaw != pose.yaw && Rotation::quarter_turn(pose.yaw, yaw).is_none())
            {
                continue;
            }
            if out.iter().any(|p: &Pose| {
                p.at[0] == pose.at[0] + dx && p.at[2] == pose.at[2] + dz && p.yaw == yaw
            }) {
                continue;
            }
            let Ok(next) = grounding.at_near(
                document,
                pose.at[0] + dx,
                pose.at[2] + dz,
                yaw,
                Some(pose.at[1]),
            ) else {
                continue;
            };
            if next.at[1].abs_diff(pose.at[1]) > u64::from(profile.body.climb) {
                continue;
            }
            let Some(to) = places.place_at(next.at) else {
                continue;
            };
            if from != to {
                let key = if from < to { [from, to] } else { [to, from] };
                if places
                    .passages
                    .get(&key)
                    .and_then(|p| profile.body.fits(p))
                    .is_none()
                {
                    continue;
                }
            }
            // Turning geometry is checked through its conservative body sweep.
            if yaw != pose.yaw
                && !turning
                && swing::turn_clear(grounding.volume, document, pose, &next).is_err()
            {
                continue;
            }
            if let Some(rules) = grounding.walking.filter(|r| r.stances > 0) {
                out.extend(footholds::arrivals(
                    grounding.volume,
                    document,
                    pose,
                    &next,
                    rules,
                ));
            } else {
                out.push(next);
            }
        }
    }
    out
}
