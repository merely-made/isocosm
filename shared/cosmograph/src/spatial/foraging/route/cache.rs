// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Exact physical inputs only. Biology and detection are re-read before lookup.
use super::*;
use std::sync::Mutex;

const CAPACITY: usize = 64;
#[derive(Debug)]
struct Entry {
    input: Input,
    result: RouteSearch,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Cache(Arc<Mutex<Vec<Entry>>>);

impl Input {
    fn matches(&self, other: &Self) -> bool {
        self.actor == other.actor
            && self.site == other.site
            && (Arc::ptr_eq(&self.terrain, &other.terrain)
                || self.terrain.same_source(&other.terrain))
            && self.document == other.document
            && self.before == other.before
            && self.profile == other.profile
            && self.foods == other.foods
            && self.blocked == other.blocked
            && self.reach == other.reach
            && self.nodes == other.nodes
            && self.walking == other.walking
            && self.balance == other.balance
    }
}
impl Cache {
    pub(super) fn find(&self, mut input: Input) -> Result<RouteSearch> {
        // Every admitted part and sampled sweep must remain in this volume.
        // A box wholly outside its horizontal window cannot block that motion.
        // Keep crossing boxes whole, including occupied food and local bodies.
        let volume = &input.terrain.volume;
        input.blocked.retain(|[lo, hi]| {
            (0..2).all(|k| lo[2 * k] < volume.max[k] && hi[2 * k] > volume.min[k])
        });
        {
            let mut entries = self.0.lock().map_err(|_| "foraging route cache poisoned")?;
            if let Some(index) = entries.iter().position(|e| e.input.matches(&input)) {
                let found = entries.remove(index);
                let result = found.result.clone();
                entries.push(found);
                let _hit = crate::profile::Span::new("foraging.route.hit");
                return Ok(result);
            }
        }
        let _span = crate::profile::Span::new("foraging.route.search");
        let result = search::find(&input)?;
        let mut entries = self.0.lock().map_err(|_| "foraging route cache poisoned")?;
        if entries.len() == CAPACITY {
            entries.remove(0);
        }
        entries.push(Entry {
            input,
            result: result.clone(),
        });
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
