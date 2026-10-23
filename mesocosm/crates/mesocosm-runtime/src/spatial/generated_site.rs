// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Whole-site arrivals use the same native placement policy as founders.
use super::*;
use isocosm::spatial::{self as native, embodiment as body};

impl Game {
    pub(super) fn site_arrivals(&self, session: &Session) -> Result<Vec<native::ArrivalResolution>,String> {
        let sim = &session.sim;
        let t = sim.genesis().rules.spatial_templates.as_ref().ok_or("habitat templates absent")?;
        let mut destinations = BTreeMap::<Id, Vec<Id>>::new();
        for (&first,group) in &sim.state().population.groups {
            let Some(j) = &group.entity.journey else { continue };
            if group.entity.body.is_none() || j.arrives < sim.state().tick || j.arrives > sim.state().tick+1 { continue; }
            destinations.entry(j.to).or_default().extend(first..first+group.count);
        }
        let mut out = Vec::new();
        for (site,ids) in destinations {
            let mut placed: BTreeMap<_,_> = self.members.iter().filter_map(|(&id,p)| {
                let e = sim.state().population.get(id)?;
                (p.site.0 == site && e.resident_site() == Some(site) && p.resident_since == Some(e.arrived))
                    .then(|| (id,p.pose.clone()))
            }).collect();
            for id in ids {
                // Search eligibility reads the arriving body at its destination.
                // The native request still binds the actual journey and tick.
                let e = sim.state().population.get(id).ok_or("arriving body absent")?;
                let j = e.journey.as_ref().ok_or("arrival journey absent")?;
                let _span = crate::profile::Span::new("arrival.search");
                let Ok(pose) = body::place_in_site(sim,t,id,site,&placed) else { continue };
                let geometry = native::ArrivalGeometry { window:body::site_window(t,pose.at),
                    eye:std::array::from_fn(|k| pose.at[k]+t.eye[k]),places:t.places,pose:pose.clone() };
                placed.insert(id,pose);
                out.push(native::ArrivalResolution { request:native::ArrivalRequest { actor:id,
                    from:j.from,to:j.to,departed:j.departed,arrives:j.arrives,
                    body_revision:e.body_revision,terrain_revision:sim.terrain_revision() },geometry });
            }
        }
        out.sort_by_key(|r| r.request.actor);
        Ok(out)
    }
}
