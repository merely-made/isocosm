// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A derived clock for equal bins and identified kin (817, 820–823).
//! No identity expansion, draw or process evaluation is an arrival.

use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Landing {
    Bin(Entity),
    Kin(Id),
}

#[derive(Default)]
pub(super) struct Journeys {
    due: BTreeMap<Tick, BTreeSet<Landing>>,
}

impl Journeys {
    fn put(&mut self, e: &Entity, who: Landing) {
        if let Some(j) = e.journey {
            self.due.entry(j.arrives).or_default().insert(who);
        }
    }

    fn remove(&mut self, e: &Entity, who: &Landing) {
        if let Some(j) = e.journey && let Some(at) = self.due.get_mut(&j.arrives) {
            at.remove(who);
            if at.is_empty() { self.due.remove(&j.arrives); }
        }
    }

    fn next(&self, until: Tick) -> Option<(Tick, Landing)> {
        let (&tick, at) = self.due.first_key_value()?;
        if tick > until { return None; }
        Some((tick, at.first()?.clone()))
    }
}

impl Crowd<'_> {
    /// Rebuild only when an averaging control replaces its bins wholesale.
    /// Ordinary accepted moves, splits, births and kin writes update entries.
    pub(super) fn index_journeys(&mut self) {
        self.journeys = Journeys::default();
        for e in self.bins.keys() {
            self.journeys.put(e, Landing::Bin(e.clone()));
        }
        for (&id, e) in &self.kin { self.journeys.put(e, Landing::Kin(id)); }
    }

    pub(super) fn insert_bin(&mut self, e: Entity, n: u64) {
        if n == 0 { return; }
        let e = normalize(e);
        self.journeys.put(&e, Landing::Bin(e.clone()));
        *self.bins.entry(e).or_default() += n;
    }

    pub(super) fn insert_kin(&mut self, id: Id, e: Entity) {
        if let Some(old) = self.kin.get(&id) {
            self.journeys.remove(old, &Landing::Kin(id));
        }
        self.journeys.put(&e, Landing::Kin(id));
        self.kin.insert(id, e);
    }

    pub(super) fn remove_kin(&mut self, id: Id) -> Option<Entity> {
        let old = self.kin.remove(&id)?;
        self.journeys.remove(&old, &Landing::Kin(id));
        Some(old)
    }

    pub(super) fn remove_bin_index(&mut self, e: &Entity) {
        self.journeys.remove(e, &Landing::Bin(e.clone()));
    }

    /// Land before installing any process pass, including a corpse or a
    /// newborn which inherited the remainder of its parent's journey.
    pub(super) fn arrive_until(&mut self, until: Tick) -> Result<()> {
        while let Some((tick, who)) = self.journeys.next(until) {
            let (e, n) = match &who {
                Landing::Bin(e) => (e.clone(), *self.bins.get(e).ok_or("journey bin vanished")?),
                Landing::Kin(id) => (self.kin.get(id).ok_or("journey kin vanished")?.clone(), 1),
            };
            let j = e.journey.ok_or("journey clock lost its authority")?;
            if j.arrives != tick || tick < self.tick || !self.sites.contains_key(&j.to) {
                return Err("journey clock differs from saved authority".into());
            }
            let arrivals = self.work.arrivals.checked_add(n).ok_or("arrival work overflow")?;
            // All refusals above leave both saved authority and derived index intact.
            self.journeys.remove(&e, &who);
            let mut landed = e.clone();
            landed.place = j.to;
            landed.arrived = tick;
            landed.journey = None;
            landed.patch = None;
            // Arrival invents neither a site deposit nor an evaluation.
            match who {
                Landing::Bin(_) => self.moved(&e, landed, n),
                Landing::Kin(id) => self.insert_kin(id, landed),
            }
            self.work.arrivals = arrivals;
        }
        Ok(())
    }
}

/// Only direct route effects can be filed without interpreting a branch.
/// Keep native due/filter policy: no speculative Choice or future-only gate.
pub(super) fn route_ready(p: &Process, e: &Entity, site_id: Id, site: &Site) -> bool {
    let moves = p.commitments.iter().chain(&p.effects).filter_map(|effect| match effect {
        crate::rules::Effect::Move { destination } => Some(*destination),
        _ => None,
    }).collect::<Vec<_>>();
    moves.is_empty() || e.resident_site() == Some(site_id)
        && moves.iter().all(|to| site.routes.iter().any(|r| r.to == *to))
}
