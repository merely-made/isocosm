// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Place near native-edible bodies at their actual terrain contacts. This
//! supplies geometry to the existing choice machinery, never a feeding act.
use super::*;

pub struct FoodStand {
    occupied: Vec<Box3>,
    contacts: Vec<([i64; 3], Box3)>,
    search: u64,
}
impl FoodStand {
    pub fn of(volume: &Volume, doc: &BodyDocument, pose: &Pose, search: u64) -> Result<Self> {
        check(volume, doc, pose)?;
        let occupied = boxes(doc, pose)?;
        let parts: BTreeMap<_, _> = doc
            .living()
            .filter(|p| p.half_extent.iter().all(|h| *h > 0))
            .zip(occupied.iter())
            .map(|(p, b)| (p.id, *b))
            .collect();
        let contacts = contacts(volume, doc, pose)?
            .into_iter()
            .filter_map(|c| Some((c.at, *parts.get(&c.part)?)))
            .collect();
        Ok(Self {
            search,
            occupied,
            contacts,
        })
    }
}
impl Grounding<'_> {
    pub fn near_food(
        &mut self,
        doc: &BodyDocument,
        centre: [i64; 3],
        yaw: Yaw,
        blocked: &[Box3],
        foods: &[FoodStand],
    ) -> Result<Pose> {
        let mut witnesses: Vec<_> = foods
            .iter()
            .flat_map(|food| food.contacts.iter().map(move |contact| (food, *contact)))
            .collect();
        witnesses.sort_by_key(|(_, (at, _))| {
            at[0]
                .abs_diff(centre[0])
                .saturating_add(at[2].abs_diff(centre[2]))
        });
        let mut starts = Vec::new();
        for (_food, (contact, [low, high])) in witnesses.into_iter().take(STANDS_PER_SITE) {
            let mut facing = yaw;
            for _ in 0..4 {
                let local = boxes(
                    doc,
                    &Pose {
                        yaw: facing,
                        ..Default::default()
                    },
                )?;
                let floor = local.iter().map(|b| b[0][1]).min().unwrap();
                for &[lo, hi] in local
                    .iter()
                    .filter(|b| b[0][1] == floor)
                    .take(STANDS_PER_SITE)
                {
                    let x = contact[0] - (lo[0] + hi[0]) / 2;
                    let z = contact[2] - (lo[2] + hi[2]) / 2;
                    let anchors = [
                        [x, 0, low[2] - self.spacing - hi[2]],
                        [x, 0, high[2] + self.spacing - lo[2]],
                        [low[0] - self.spacing - hi[0], 0, z],
                        [high[0] + self.spacing - lo[0], 0, z],
                    ];
                    starts.extend(anchors.into_iter().map(|anchor| (anchor, facing)));
                }
                facing = facing.compose(Yaw::Quarter);
            }
        }
        // One shared budget across contacts, orientations and nearby offsets.
        // Try exact anatomically aligned anchors first, then widen their rings.
        let mut seen = BTreeSet::new();
        for ring in (0..=SEARCH_REACH).step_by(SEARCH_STRIDE) {
            for &(anchor, facing) in &starts {
                for dx in (-ring..=ring).step_by(SEARCH_STRIDE) {
                    for dz in (-ring..=ring).step_by(SEARCH_STRIDE) {
                        if dx.abs().max(dz.abs()) != ring {
                            continue;
                        }
                        let at = [anchor[0] + dx, 0, anchor[2] + dz];
                        if !seen.insert((at, facing)) {
                            continue;
                        }
                        if seen.len() > SEARCH_CANDIDATES {
                            return Err("food placement candidate budget exhausted".into());
                        }
                        if let Ok(pose) =
                            self.find_where(doc, at, facing, 0, 1, 1, blocked, |_, occupied| {
                                foods.iter().any(|food| {
                                    super::super::foraging::gap(occupied, &food.occupied)
                                        <= food.search
                                })
                            })
                        {
                            return Ok(pose);
                        }
                    }
                }
            }
        }
        Err("no supported stance near native-edible placed food".into())
    }
}
