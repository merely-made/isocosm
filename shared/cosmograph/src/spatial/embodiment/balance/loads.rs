// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Quasi-static measurements. No reaction split, gravity or strength constant.
use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct FootLoad {
    pub foot: PartId,
    pub contact_cells: u64,
    pub upper_section: u128,
    pub lower_section: u128,
}
#[derive(Clone, Debug, Serialize)]
pub struct Load {
    pub mass_mg: u128,
    pub tissue_mg: u128,
    pub current_parts_capacity_mg: u128,
    pub age: Tick,
    /// World coordinates in 1/1024 base cells, the gait's existing units.
    pub centre: [i64; 3],
    pub rest_volume_centre: [i64; 3],
    pub contact_cells: u64,
    pub feet: Vec<FootLoad>,
    pub supported: bool,
}
pub fn stride_load(
    sim: &Simulation,
    actor: Id,
    volume: &Volume,
    from: &Pose,
    to: &Pose,
    stride: &isometer_core::legs::Stride,
    phase: i64,
) -> Result<Load> {
    let member = motion_member(sim, actor)?;
    let doc = member.body.as_ref().ok_or("load geometry absent")?;
    let balance = Balance::of(member, &sim.genesis().rules)?;
    let rules = &sim.genesis().rules;
    let own = crate::anatomy::own(rules, &member.lineage);
    let tissue_mg = member
        .living()
        .map(|(_, p)| {
            own.tissue
                .as_ref()
                .and_then(|key| p.matter.get(key))
                .copied()
                .unwrap_or(0) as u128
        })
        .sum();
    let current_parts_capacity_mg = member
        .living()
        .map(|(id, _)| u128::from(crate::anatomy::ceiling(member.extent(id), rules.body())))
        .sum();
    let frame = stride_sample(from, to, stride, phase)?;
    let local = stride_feet(&frame, from.yaw)?;
    let rotation = Rotation::at(from.yaw, frame.turn);
    let c = balance.at(doc, &local, rotation)?;
    let centre = [0, 1, 2].map(|k| frame.root[k] + c[k]);
    let c = support::centre_rotated(doc, rotation)?;
    let rest_volume_centre = [0, 1, 2].map(|k| frame.root[k] + c[k]);
    let pads = narrow::pad_shapes(doc, frame.root, rotation, &local)?;
    let hull = support::bearing_pads(volume, &pads, &frame.bearing)?;
    let mut feet = Vec::new();
    for leg in isometer_core::legs::chains(doc)
        .iter()
        .filter(|leg| frame.bearing.contains(&leg.foot))
    {
        let pad = pads.get(&leg.foot).ok_or("load pad absent")?;
        let [lo, hi] = pad.cells();
        let mut contact_cells = 0;
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                if pad.columns(x, z) && contacts::supported(volume, [x, lo[1] - 1, z]) {
                    contact_cells += 1;
                }
            }
        }
        let section =
            |id| crate::anatomy::measure(member.extent(id), crate::rules::Measure::CrossSection);
        feet.push(FootLoad {
            foot: leg.foot,
            contact_cells,
            upper_section: section(leg.upper),
            lower_section: section(leg.lower),
        });
    }
    Ok(Load {
        mass_mg: balance.mass,
        tissue_mg,
        current_parts_capacity_mg,
        age: sim.state().tick.saturating_sub(member.born),
        centre,
        rest_volume_centre,
        contact_cells: feet.iter().map(|f| f.contact_cells).sum(),
        feet,
        supported: isometer_core::legs::support::contains(&hull, [centre[0], centre[2]]),
    })
}
