// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Smaller planted pivots joined by the existing paid foot-step planner.
use super::*;

pub(super) fn validate(
    volume: &Volume,
    doc: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Walking,
    stride: &Stride,
) -> Result<()> {
    if !rules.valid()
        || !rules.turns
        || stride.clearance != rules.clearance
        || Rotation::quarter_turn(from.yaw, to.yaw) != Some(stride.turn)
        || stride.stages.len() > 64
        || from.feet.is_empty()
        || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1)
    {
        return Err("turn differs from its bounded walking policy".into());
    }
    check(volume, doc, from)?;
    check(volume, doc, to)?;
    let mut feet = foot_points(from)?;
    let mut angle = 0;
    let direction = i64::from(stride.turn);
    let reach = i64::from(rules.shift) * UNIT;
    for s in &stride.stages {
        let old = feet.get(&s.foot).ok_or("turn foot absent")?;
        let target = s.target.ok_or("turn foot target absent")?;
        if s.turn.unsigned_abs() > UNIT as u64
            || target.iter().any(|p| p.unsigned_abs() > (1u64 << 52))
            || s.lift < 0
            || s.lift > i64::from(rules.lift) * UNIT
            || s.turn * direction < angle * direction
            || s.turn * direction > UNIT
            || (s.turn != angle && s.lift != 0)
            || (s.lift == 0 && target != *old)
            || (0..3).any(|k| {
                s.root[k] < from.at[k].min(to.at[k]) * UNIT - reach
                    || s.root[k] > from.at[k].max(to.at[k]) * UNIT + reach
            })
        {
            return Err("turn stage exceeds its paid lift, shift or planted-pivot bounds".into());
        }
        feet.insert(s.foot, target);
        angle = s.turn;
    }
    if angle != direction * UNIT || feet != foot_points(to)? {
        return Err("turn does not finish at its admitted facing and soles".into());
    }
    Ok(())
}

fn planted(root: [i64; 3], feet: BTreeMap<PartId, [i64; 3]>, turn: i64) -> StrideFrame {
    StrideFrame {
        root,
        bearing: feet.keys().copied().collect(),
        feet,
        turn,
    }
}
fn balanced(
    volume: &Volume,
    doc: &BodyDocument,
    mut frame: StrideFrame,
    yaw: Yaw,
    from: &Pose,
    to: &Pose,
    rules: Walking,
) -> Result<StrideFrame> {
    let rotation = Rotation::at(yaw, frame.turn);
    let local = local_feet(&frame, yaw)?;
    let hull = support::bearing_rotated(volume, doc, &local, frame.root, rotation, &frame.bearing)?;
    let centre = support::centre_rotated(doc, rotation)?;
    let at = polygon::balance(
        &hull,
        [frame.root[0] + centre[0], frame.root[2] + centre[2]],
        UNIT / 4,
    )
    .ok_or("pivot has no balanced body position")?;
    frame.root[0] = at[0] - centre[0];
    frame.root[2] = at[1] - centre[2];
    let reach = i64::from(rules.shift) * UNIT;
    if [0, 2].iter().any(|&k| {
        frame.root[k] < from.at[k].min(to.at[k]) * UNIT - reach
            || frame.root[k] > from.at[k].max(to.at[k]) * UNIT + reach
    }) {
        return Err("pivot balance exceeds the declared body shift".into());
    }
    Ok(frame)
}
fn body_stage(frame: &StrideFrame) -> Stage {
    let (&foot, &target) = frame.feet.iter().next().unwrap();
    Stage {
        foot,
        root: frame.root,
        lift: 0,
        target: Some(target),
        turn: frame.turn,
    }
}
fn append(stages: &mut Vec<Stage>, entry: Stride, goal: &StrideFrame, angle: i64) {
    stages.extend(entry.stages.into_iter().map(|mut s| {
        s.target = Some(goal.feet[&s.foot]);
        s.turn = angle;
        s
    }));
}

fn common(
    g: &mut Grounding<'_>,
    doc: &BodyDocument,
    root: [i64; 3],
    yaw: Yaw,
    a: i64,
    b: i64,
    rules: Walking,
    current: &StrideFrame,
) -> Vec<BTreeMap<PartId, [i64; 3]>> {
    let chains = legs::chains(doc);
    let middle = Rotation::at(yaw, (a + b) / 2);
    let mut choices = Vec::new();
    for leg in &chains {
        let options = footholds::targets_rotated(
            g,
            leg,
            root[0] / UNIT,
            root[2] / UNIT,
            middle,
            i64::from(rules.foot_search),
        )
        .unwrap_or_default();
        let mut points: Vec<_> = options
            .into_iter()
            .filter_map(|p| {
                let offset = middle.rotate([p[0], p[1] - root[1], p[2]])?;
                let world = [0, 1, 2].map(|k| offset[k] + root[k]);
                (0..=8)
                    .all(|i| {
                        Rotation::at(yaw, a + (b - a) * i / 8)
                            .inverse(offset)
                            .and_then(|local| leg.skin(local))
                            .is_some()
                    })
                    .then_some(world)
            })
            .take(8)
            .collect();
        if let Some(&old) = current.feet.get(&leg.foot) {
            let offset = [0, 1, 2].map(|k| old[k] - root[k]);
            if (0..=8).all(|i| {
                Rotation::at(yaw, a + (b - a) * i / 8)
                    .inverse(offset)
                    .and_then(|p| leg.skin(p))
                    .is_some()
            }) {
                points.retain(|p| *p != old);
                points.insert(0, old);
            }
        }
        if points.is_empty() {
            return Vec::new();
        }
        choices.push((leg.foot, points));
    }
    if choices.is_empty() {
        return Vec::new();
    }
    let base: BTreeMap<_, _> = choices.iter().map(|(id, ps)| (*id, ps[0])).collect();
    let mut variants = vec![base.clone()];
    let nominal: BTreeMap<_, _> = choices
        .iter()
        .map(|(id, ps)| {
            (
                *id,
                *ps.iter()
                    .find(|p| current.feet.get(id) != Some(*p))
                    .unwrap_or(&ps[0]),
            )
        })
        .collect();
    if nominal != base {
        variants.push(nominal);
    }
    'pairs: for (first, (id, points)) in choices.iter().enumerate() {
        for &point in points.iter().skip(1).take(2) {
            if variants.len() >= usize::from(rules.postures) {
                break 'pairs;
            }
            let mut one = base.clone();
            one.insert(*id, point);
            variants.push(one.clone());
            for (second, targets) in choices.iter().skip(first + 1) {
                if variants.len() >= usize::from(rules.postures) {
                    break 'pairs;
                }
                let mut two = one.clone();
                two.insert(*second, targets[0]);
                if let Some(p) = targets
                    .iter()
                    .find(|p| current.feet.get(second) != Some(*p))
                {
                    two.insert(*second, *p);
                }
                variants.push(two);
            }
        }
    }
    'variants: for (id, points) in choices {
        for p in points.into_iter().skip(1) {
            if variants.len() >= usize::from(rules.postures) {
                break 'variants;
            }
            let mut next = base.clone();
            next.insert(id, p);
            variants.push(next);
        }
    }
    variants
}

fn attempt(
    volume: &Volume,
    doc: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Walking,
    blocked: &[Box3],
    root: [i64; 3],
    pieces: i64,
    direction: i8,
    g: &mut Grounding<'_>,
    tried: &mut usize,
) -> Result<Stride> {
    let mut current = frame(from)?;
    let mut stages = Vec::new();
    if current.root[1] != root[1] {
        let mut low = current.clone();
        low.root[1] = root[1];
        let entry = plan_at(
            volume, doc, &current, &low, from.yaw, 0, rules, blocked, tried,
        )?;
        append(&mut stages, entry, &low, 0);
        stages.push(body_stage(&low));
        current = low;
    }
    for piece in 0..pieces {
        let angle = i64::from(direction) * UNIT * piece / pieces;
        let next_angle = i64::from(direction) * UNIT * (piece + 1) / pieces;
        let mut accepted = None;
        let mut why = "no common paid stance for the next small pivot".to_string();
        for world in common(g, doc, root, from.yaw, angle, next_angle, rules, &current) {
            *tried += 1;
            if *tried > usize::from(rules.postures) {
                return Err(format!("turn posture budget exhausted: {why}"));
            }
            let result: Result<_> = (|| {
                let before = balanced(
                    volume,
                    doc,
                    planted(root, world.clone(), angle),
                    from.yaw,
                    from,
                    to,
                    rules,
                )?;
                let after = balanced(
                    volume,
                    doc,
                    planted(root, world, next_angle),
                    from.yaw,
                    from,
                    to,
                    rules,
                )?;
                for i in 0..=16 {
                    let mut f = before.clone();
                    f.root = [0, 1, 2]
                        .map(|k| before.root[k] + (after.root[k] - before.root[k]) * i / 16);
                    f.turn = angle + (next_angle - angle) * i / 16;
                    check_frame(volume, doc, &f, from.yaw, blocked)?;
                }
                let entry = plan_at(
                    volume, doc, &current, &before, from.yaw, angle, rules, blocked, tried,
                )?;
                Ok((entry, before, after))
            })();
            match result {
                Ok(value) => {
                    accepted = Some(value);
                    break;
                },
                Err(error) => why = format!("pivot {}/{pieces}: {error}", piece + 1),
            }
        }
        let (entry, before, after) = accepted.ok_or(why)?;
        append(&mut stages, entry, &before, angle);
        // Finish the admitted fixed-orientation root transfer before rotating.
        if stages.last().map_or(from.at.map(|v| v * UNIT), |s| s.root) != before.root {
            stages.push(body_stage(&before));
        }
        stages.push(body_stage(&after));
        if stages.len() > 64 {
            return Err("turn exceeds the existing stage budget".into());
        }
        current = after;
    }
    let mut end = frame(to)?;
    end.turn = i64::from(direction) * UNIT;
    let exit = plan_at(
        volume, doc, &current, &end, from.yaw, end.turn, rules, blocked, tried,
    )?;
    append(&mut stages, exit, &end, end.turn);
    let stride = Stride {
        stages,
        clearance: rules.clearance,
        turn: direction,
    };
    check_stride(volume, doc, from, to, rules, &stride, blocked)?;
    Ok(stride)
}

pub(super) fn plan(
    volume: &Volume,
    doc: &BodyDocument,
    from: &Pose,
    to: &Pose,
    rules: Walking,
    blocked: &[Box3],
) -> Result<Stride> {
    let direction = Rotation::quarter_turn(from.yaw, to.yaw)
        .ok_or("a supported turn is one quarter at a time")?;
    if !rules.valid() || from.feet.is_empty() || (0..3).any(|k| from.at[k].abs_diff(to.at[k]) > 1) {
        return Err("turn needs a bounded adjacent articulated step".into());
    }
    check(volume, doc, from)?;
    check(volume, doc, to)?;
    let mut grounding = Grounding::new(volume).with_walking(Some(rules));
    let mut tried = 0usize;
    let mut why = "no paid sequence of smaller pivots".to_string();
    let mut roots = Vec::new();
    for base in [from.at, to.at] {
        for rise in (1..=i64::from(rules.shift)).map(|n| -n).chain([0, 1]) {
            let root = [base[0], from.at[1].max(to.at[1]) + rise, base[2]].map(|v| v * UNIT);
            if !roots.contains(&root)
                && (0..3).all(|k| {
                    let limit = if k == 1 {
                        i64::from(rules.shift) * UNIT
                    } else {
                        UNIT
                    };
                    root[k].abs_diff(from.at[k] * UNIT) <= limit as u64
                        && root[k].abs_diff(to.at[k] * UNIT) <= limit as u64
                })
            {
                roots.push(root);
            }
        }
    }
    for pieces in [8, 4] {
        for &root in &roots {
            if tried >= usize::from(rules.postures) {
                return Err(format!("turn posture budget exhausted: {why}"));
            }
            match attempt(
                volume,
                doc,
                from,
                to,
                rules,
                blocked,
                root,
                pieces,
                direction,
                &mut grounding,
                &mut tried,
            ) {
                Ok(stride) => return Ok(stride),
                Err(error) => why = format!("root {root:?}, {pieces} pivots: {error}"),
            }
        }
    }
    Err(why)
}
