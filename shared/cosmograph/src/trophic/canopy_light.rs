// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A plant's direct overhead light sampled at sixteen positions on each fixing plate.
//! Units are 1/64 of one effective cell. The understory leaf uses one quarter
//! efficiency under cover and three quarters in the open; ordinary leaves use
//! direct light at full efficiency. These are game rate parameters.

use super::*;

struct Surface {
    id: PartId,
    // Box limits in eighths of a voxel. Quarter turns keep axes aligned.
    low: [i64; 3],
    high: [i64; 3],
    upright_plate: bool,
}

fn surfaces(doc: &BodyDocument) -> Vec<Surface> {
    doc.living()
        .filter_map(|p| {
            let origin = doc.world_offset(p.id)?.map(i64::from);
            let yaw = doc.world_yaw(p.id)?;
            let span = p.half_extent.map(|n| 2 * i64::from(n).abs());
            // The far corner is opposite the local origin after any quarter turn.
            let x = yaw.rotate([1, 0, 0]);
            let z = yaw.rotate([0, 0, 1]);
            let far = std::array::from_fn::<_, 3, _>(|i| {
                origin[i]
                    + i64::from(x[i]) * span[0]
                    + if i == 1 { span[1] } else { 0 }
                    + i64::from(z[i]) * span[2]
            });
            Some(Surface {
                id: p.id,
                low: std::array::from_fn(|i| origin[i].min(far[i]) * 8),
                high: std::array::from_fn(|i| origin[i].max(far[i]) * 8),
                upright_plate: classify(p.half_extent) == Role::Plate
                    && p.attachment.is_some_and(|a| {
                        a.offset[1] > 0
                            && a.offset[1] >= a.offset[0].saturating_abs()
                            && a.offset[1] >= a.offset[2].saturating_abs()
                    }),
            })
        })
        .collect()
}

/// Total fixing cells in canopy positions and capture by tissue backed cells.
pub fn capture(e: &Entity, tissue: &str) -> (u64, u64) {
    let Some(doc) = &e.body else { return (0, 0) };
    let boxes = surfaces(doc);
    let mut cells = 0;
    let mut units = 0;
    for leaf in boxes.iter().filter(|b| b.upright_plate) {
        let Some(p) = e.parts.get(&leaf.id) else {
            continue;
        };
        let fixed = u64::from(p.cells.get(FIX).copied().unwrap_or(0));
        cells += fixed;
        let backed = fixed.min(p.matter.get(tissue).copied().unwrap_or(0));
        if backed == 0 {
            continue;
        }
        let mut shade = 0u16;
        for b in boxes.iter().filter(|b| b.high[1] > leaf.high[1]) {
            let sample = |axis: usize, n: i64| {
                leaf.low[axis] + (2 * n + 1) * (leaf.high[axis] - leaf.low[axis]) / 8
            };
            for row in 0..4 {
                let z = sample(2, row);
                if z < b.low[2] || z >= b.high[2] {
                    continue;
                }
                for col in 0..4 {
                    let x = sample(0, col);
                    if x >= b.low[0] && x < b.high[0] {
                        shade |= 1 << (row * 4 + col);
                    }
                }
            }
        }
        let shaded = u64::from(shade.count_ones());
        let direct = 16 - shaded;
        let response = p
            .port
            .and_then(|p| p.light_response())
            .unwrap_or(crate::process::LightResponse::ORDINARY);
        units +=
            backed * (u64::from(response.open) * direct + u64::from(response.covered) * shaded);
    }
    (cells, units)
}

/// Same pulse-density rule as ordinary canopy growth at 64 times its precision.
pub(crate) fn due(units: u64, budget: u32, turn: u64) -> bool {
    if budget == 0 {
        return false;
    }
    let full = u128::from(budget) * 64;
    let phase = u128::from(turn) % full;
    let rate = u128::from(units).min(full);
    (phase + 1) * rate / full != phase * rate / full
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Body, Sketch};

    fn plant(offset: i32, shade_leaf: bool) -> Entity {
        let mut e = crate::probe::BodyFounding::default()
            .generate()
            .unwrap()
            .genesis
            .population
            .groups
            .into_values()
            .next()
            .unwrap()
            .entity
            .as_ref()
            .clone();
        let mut plate = Part::default();
        plate.cells.insert(FIX.into(), 8);
        plate.matter.insert("green".into(), 4);
        if shade_leaf {
            let leaf = crate::catalog::catalog()
                .part_by_key("part:understory-leaf")
                .unwrap();
            plate.port = crate::repertoire::lowering(leaf).unwrap().template.port;
        }
        let sketch = |parent, half_extent, offset, part| Sketch {
            parent,
            half_extent,
            offset,
            part,
            ..Default::default()
        };
        e.embody(Body::sketch([
            (0, sketch(None, [1, 1, 1], [0; 3], Part::default())),
            (1, sketch(Some(0), [4, 1, 4], [0, 3, 0], plate)),
            (
                2,
                sketch(Some(0), [4, 1, 4], [offset, 7, 0], Part::default()),
            ),
        ]));
        e
    }

    #[test]
    fn full_partial_and_clear_shade_depend_on_the_actual_footprints() {
        for (offset, ordinary, understory) in [(0, 0, 1), (4, 2, 2), (12, 4, 3)] {
            for (shade_leaf, wanted) in [(false, ordinary), (true, understory)] {
                let e = plant(offset, shade_leaf);
                assert_eq!(
                    fixing_tissue(&e, "green").1,
                    0,
                    "the old rule shades the whole plant"
                );
                let (cells, units) = capture(&e, "green");
                assert_eq!((cells, units), (8, 64 * wanted));
                assert_eq!(
                    (0..4).filter(|&n| due(units, 4, n)).count(),
                    wanted as usize
                );
                assert_eq!(capture(&e, "empty").1, 0);
            }
        }
    }
    #[test]
    fn a_prairie_blade_trades_its_shade_yield_for_open_light() {
        let row = crate::catalog::catalog()
            .part_by_key("part:prairie-blade")
            .unwrap();
        let blade = crate::repertoire::lowering(row)
            .unwrap()
            .template
            .port
            .unwrap();
        for (offset, expected) in [(0, 0), (4, 160), (12, 320)] {
            let mut e = plant(offset, false);
            e.part_mut(&PartId(1)).unwrap().port = Some(blade);
            assert_eq!(capture(&e, "green"), (8, expected));
        }
    }

    #[test]
    fn a_turned_shoot_casts_its_rotated_shadow_and_a_severed_one_casts_none() {
        let mut e = plant(4, false);
        let blocker = &mut e.body.as_mut().unwrap().parts[2];
        blocker.half_extent = [1, 1, 4];
        blocker.pivot = blocker.half_extent;
        assert_eq!(capture(&e, "green").1, 4 * 4 * 12);
        let blocker = &mut e.body.as_mut().unwrap().parts[2];
        blocker.attachment.as_mut().unwrap().yaw = isometer_core::Yaw::Quarter;
        assert_eq!(capture(&e, "green").1, 4 * 4 * 14);
        e.body.as_mut().unwrap().parts[2].severed = true;
        assert_eq!(capture(&e, "green").1, 4 * 64);
    }
}
