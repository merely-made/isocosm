// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Fixed substrate positions for a rooted colony and a finite litter deposit.
//! The same living part boxes drive contact and the soil cutaway.
use crate::{
    Result, Simulation, growth,
    rules::{Development, Rules},
    schema::{Entity, Id},
    spatial::{Pose, embodiment},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Bounds = [[i64; 3]; 2];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bed {
    pub litter: Bounds,
    pub colonies: BTreeMap<Id, Pose>,
    /// Local wind carries the provisioned seed across this bed in base units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_rain: Option<SeedRain>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedRain {
    pub lineage: String,
    pub drift: [i32; 2],
    pub jitter: u16,
}

impl Bed {
    pub fn rooted(&self, e: &Entity) -> bool {
        self.seed_rain
            .as_ref()
            .is_some_and(|rain| rain.lineage == e.lineage)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Contact {
    Unanchored,
    SeedLanded,
    InSoil,
    Litter,
    Root,
}

/// Shared faces conduct. Sharing just a corner or an edge does not.
pub fn touching(a: Bounds, b: Bounds) -> bool {
    let overlap = [0, 1, 2].map(|i| a[1][i].min(b[1][i]).cmp(&a[0][i].max(b[0][i])));
    overlap.iter().all(|v| !v.is_lt()) && overlap.iter().filter(|v| v.is_gt()).count() >= 2
}

pub fn pose<'a>(bed: &'a Bed, site: Id, id: Id, e: &Entity) -> Option<&'a Pose> {
    (e.alive && e.resident_site() == Some(site) && e.arrived == 0)
        .then(|| bed.colonies.get(&id))
        .flatten()
}

pub fn collar(e: &Entity, p: &Pose) -> Option<Bounds> {
    let d = e.body.as_ref()?;
    if !e.lives(d.root) {
        return None;
    }
    embodiment::part_boxes(d, p).ok()?.get(&d.root).copied()
}

/// Whether any living part occupies the deposit or contacts its face.
pub fn reaches(e: &Entity, anchor: &Pose, bounds: Bounds) -> bool {
    e.body
        .as_ref()
        .and_then(|doc| embodiment::part_boxes(doc, anchor).ok())
        .is_some_and(|parts| {
            e.living()
                .any(|(id, _)| parts.get(&id).is_some_and(|b| touching(*b, bounds)))
        })
}

pub(crate) fn test(
    sim: &Simulation,
    actor: Id,
    target: Option<Id>,
    site: Id,
    kind: Contact,
) -> Result<String> {
    if kind == Contact::SeedLanded {
        let requires_landing = sim
            .genesis()
            .rules
            .soil_beds
            .get(&site)
            .is_some_and(|bed| sim.body_at_start(actor).is_some_and(|e| bed.rooted(e)));
        return if !requires_landing || sim.spatial_pose_at_start(actor).is_some() {
            Ok("seed on ground".into())
        } else {
            Err("seed lacks a landing".into())
        };
    }
    if kind == Contact::Unanchored {
        let anchored = sim.genesis().rules.soil_beds.get(&site).is_some_and(|bed| {
            bed.colonies.contains_key(&actor)
                || sim.body_at_start(actor).is_some_and(|e| bed.rooted(e))
        });
        return if anchored {
            Err("rooted in substrate".into())
        } else {
            Ok("unanchored".into())
        };
    }
    let yes = (|| {
        let bed = sim.genesis().rules.soil_beds.get(&site)?;
        // A root requests phosphate from its connected fungus. Growth and
        // litter recovery are acts of the colony itself.
        let colony = if kind == Contact::Root {
            target?
        } else {
            actor
        };
        let fungus = sim.body_at_start(colony)?;
        let anchor = pose(bed, site, colony, fungus)?;
        let bounds = match kind {
            Contact::Unanchored | Contact::SeedLanded => unreachable!(),
            Contact::InSoil => return Some(true),
            Contact::Litter => bed.litter,
            Contact::Root => {
                let plant = sim.body_at_start(actor)?;
                let r = sim.state().spatial_poses.get(&actor)?;
                if !plant.alive
                    || plant.resident_site() != Some(site)
                    || r.site != site
                    || r.arrived != plant.arrived
                {
                    return None;
                }
                collar(plant, &r.pose)?
            },
        };
        Some(reaches(fungus, anchor, bounds))
    })()
    .unwrap_or(false);
    if yes {
        Ok(format!("contact: {kind:?}"))
    } else {
        Err(format!("missing requirement: {kind:?} contact"))
    }
}

/// Extend one connected part of the inherited map, moving a unit of basal
/// tissue into it. Leave living stock in the collar for the next extension.
pub(crate) fn extend(
    body: &mut Entity,
    rules: &Rules,
    d: &Development,
    tissue: &str,
) -> Result<()> {
    let root = body.body.as_ref().ok_or("extension needs a body")?.root;
    if body
        .parts
        .get(&root)
        .and_then(|p| p.matter.get(tissue))
        .copied()
        .unwrap_or(0)
        < 2
    {
        return Ok(());
    }
    if let Some((frame, mut part)) = growth::lacking(rules, d, body)? {
        *part.matter.entry(tissue.into()).or_default() += 1;
        body.add_part(&frame, part)?;
        body.body_revision = body
            .body_revision
            .checked_add(1)
            .ok_or("body revision overflow")?;
        *body
            .part_mut(&root)
            .and_then(|p| p.matter.get_mut(tissue))
            .ok_or("extension lost its source")? -= 1;
    }
    Ok(())
}
