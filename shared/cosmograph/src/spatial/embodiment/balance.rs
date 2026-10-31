// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native matter at resolved part centres, never mass inferred from a picture.
use super::*;
use isometer_core::{
    PartId,
    legs::{Leg, Rotation, Skin, UNIT},
};
mod loads;
pub use loads::{FootLoad, Load, stride_load};

/// Biology chooses from pass-start reads; physical balance uses the actual
/// matter present when geometry is requested. Old worlds keep their source.
pub(crate) fn motion_member(sim: &Simulation, actor: Id) -> Result<&Entity> {
    if walking_rules(sim).is_some_and(|r| r.balance) {
        sim.state()
            .population
            .get(actor)
            .ok_or("motion body absent".into())
    } else {
        sim.body_at_start(actor).ok_or("motion body absent".into())
    }
}

pub fn neighbours_for(
    sim: &Simulation,
    actor: Id,
    volume: &Volume,
    places: &Places,
    profile: &StepProfile,
    pose: &Pose,
) -> Result<Vec<Pose>> {
    let member = motion_member(sim, actor)?;
    let doc = member.body.as_ref().ok_or("motion geometry absent")?;
    let local = boxes(
        doc,
        &Pose {
            yaw: pose.yaw,
            ..Pose::default()
        },
    )?;
    Ok(StrideChecks::new(volume, doc, &[])
        .with_balance(balance_for(sim, member)?)
        .neighbours(places, profile, &local, pose, walking_rules(sim)))
}
pub fn step_for(
    sim: &Simulation,
    actor: Id,
    volume: &Volume,
    from: &Pose,
    to: &Pose,
    blocked: &[Box3],
) -> Result<Option<isometer_core::legs::Stride>> {
    let member = motion_member(sim, actor)?;
    let doc = member.body.as_ref().ok_or("motion geometry absent")?;
    StrideChecks::new(volume, doc, blocked)
        .with_balance(balance_for(sim, member)?)
        .step(from, to, walking_rules(sim))
}

pub fn check_motion(
    sim: &Simulation,
    actor: Id,
    volume: &Volume,
    from: &Pose,
    to: &Pose,
    stride: Option<&isometer_core::legs::Stride>,
    blocked: &[Box3],
) -> Result<()> {
    let member = motion_member(sim, actor)?;
    let doc = member.body.as_ref().ok_or("motion geometry absent")?;
    StrideChecks::new(volume, doc, blocked)
        .with_balance(balance_for(sim, member)?)
        .check_walk(from, to, walking_rules(sim), stride)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    parts: BTreeMap<PartId, (u128, [i64; 3])>,
    mass: u128,
    moment: [i128; 3],
}
impl Balance {
    pub fn of(entity: &Entity, rules: &crate::rules::Rules) -> Result<Self> {
        let doc = entity.body.as_ref().ok_or("balance body absent")?;
        Self::from_parts(
            doc,
            entity
                .living()
                .map(|(id, p)| (id, crate::meaning::mass(&p.matter, rules))),
        )
    }
    /// Masses in native milligrams, keyed by the existing part identities.
    pub fn from_parts(
        doc: &BodyDocument,
        masses: impl IntoIterator<Item = (PartId, u128)>,
    ) -> Result<Self> {
        let masses: BTreeMap<_, _> = masses.into_iter().collect();
        let mut parts = BTreeMap::new();
        let mut moment = [0i128; 3];
        let mut mass = 0u128;
        for part in doc.living() {
            let m = masses.get(&part.id).copied().unwrap_or(0);
            let p = doc
                .world_pivot(part.id)
                .ok_or("mass part has no graph placement")?
                .map(|v| i64::from(v) * UNIT);
            mass = mass.checked_add(m).ok_or("body mass overflow")?;
            for k in 0..3 {
                moment[k] = moment[k]
                    .checked_add(product(m, p[k])?)
                    .ok_or("body moment overflow")?;
            }
            parts.insert(part.id, (m, p));
        }
        if mass == 0 {
            return Err("body has no living matter mass".into());
        }
        Ok(Self {
            parts,
            mass,
            moment,
        })
    }
    pub fn mass_mg(&self) -> u128 {
        self.mass
    }
    /// Link capsules are symmetric about their midpoint; the foot uses its
    /// rigid pad's centre. Other parts retain their graph-resolved pivots.
    pub(super) fn centre<'a>(
        &self,
        skins: impl Iterator<Item = (&'a Leg, &'a Skin)>,
    ) -> Result<[i64; 3]> {
        let mut moment = self.moment;
        for (leg, skin) in skins {
            for (id, a, b) in [
                (leg.upper, skin.upper.a, skin.upper.b),
                (leg.lower, skin.lower.a, skin.lower.b),
                (leg.foot, skin.pad[0], skin.pad[1]),
            ] {
                let (mass, rest) = self.parts.get(&id).ok_or("posed mass part absent")?;
                for k in 0..3 {
                    let p = a[k] + (b[k] - a[k]) / 2;
                    moment[k] = moment[k]
                        .checked_add(product(*mass, p - rest[k])?)
                        .ok_or("posed moment overflow")?;
                }
            }
        }
        let mass = i128::try_from(self.mass).map_err(|_| "body mass exceeds moment arithmetic")?;
        let mut out = [0; 3];
        for k in 0..3 {
            out[k] = i64::try_from(moment[k] / mass).map_err(|_| "mass centre overflow")?;
        }
        Ok(out)
    }
    pub(super) fn at(
        &self,
        doc: &BodyDocument,
        feet: &BTreeMap<PartId, [i64; 3]>,
        rotation: Rotation,
    ) -> Result<[i64; 3]> {
        let chains = isometer_core::legs::chains(doc);
        let skins = chains
            .iter()
            .map(|leg| {
                leg.skin(*feet.get(&leg.foot).ok_or("mass foot absent")?)
                    .ok_or("mass sole exceeds paid reach".into())
            })
            .collect::<Result<Vec<_>>>()?;
        rotation
            .rotate(self.centre(chains.iter().zip(&skins))?)
            .ok_or("posed centre rotation overflow".into())
    }
}
fn product(mass: u128, distance: i64) -> Result<i128> {
    i128::try_from(mass)
        .ok()
        .and_then(|m| m.checked_mul(i128::from(distance)))
        .ok_or("body moment overflow".into())
}
pub fn balance_for(sim: &Simulation, member: &Entity) -> Result<Option<Balance>> {
    walking_rules(sim)
        .filter(|r| r.balance)
        .map(|_| Balance::of(member, &sim.genesis().rules))
        .transpose()
}
pub fn check_member(sim: &Simulation, volume: &Volume, member: &Entity, pose: &Pose) -> Result<()> {
    let doc = member.body.as_ref().ok_or("balance body absent")?;
    if articulated(sim) && pose.feet.is_empty() && !isometer_core::legs::chains(doc).is_empty() {
        return Err("articulated world requires admitted paid-foot placement".into());
    }
    check_balanced(volume, doc, pose, balance_for(sim, member)?.as_ref())
}
pub(super) fn check_balanced(
    volume: &Volume,
    doc: &BodyDocument,
    pose: &Pose,
    balance: Option<&Balance>,
) -> Result<()> {
    let Some(balance) = balance.filter(|_| !pose.feet.is_empty()) else {
        return check(volume, doc, pose);
    };
    let occupied = boxes(doc, pose)?;
    let root = pose.at.map(|p| p * UNIT);
    narrow::clear(volume, doc, root, pose.yaw, &pose.feet, &occupied, &[])?;
    let centre = balance.at(doc, &pose.feet, Rotation::at(pose.yaw, 0))?;
    let bearing = pose.feet.keys().copied().collect();
    let hull = support::bearing(volume, doc, &pose.feet, root, pose.yaw, &bearing)?;
    if !isometer_core::legs::support::contains(&hull, [root[0] + centre[0], root[2] + centre[2]]) {
        return Err("posed matter centre leaves its bearing-foot polygon".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
