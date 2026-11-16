// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Exact physical inputs only. Biology and detection are re-read before lookup.
use super::*;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Hits checked against a fresh search so far, in verify mode.
pub(crate) static VERIFIED: AtomicUsize = AtomicUsize::new(0);

const CAPACITY: usize = 64;
/// Set to check every hit against a fresh search and panic on any difference.
const VERIFY_ENV: &str = "COSMOGRAPH_ROUTE_CACHE_VERIFY";
fn verifying() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os(VERIFY_ENV).is_some())
}
#[derive(Debug)]
struct Entry {
    input: Input,
    /// What the search asked of the other bodies; `input.blocked` is not
    /// compared, this is.
    asked: embodiment::Footprint,
    result: RouteSearch,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Cache(Arc<Mutex<Vec<Entry>>>);

impl Input {
    /// Every input but `blocked`, which a remembered search's footprint judges.
    fn matches(&self, other: &Self) -> bool {
        self.actor == other.actor
            && self.site == other.site
            && (Arc::ptr_eq(&self.terrain, &other.terrain)
                || self.terrain.same_source(&other.terrain))
            && self.document == other.document
            && self.before == other.before
            && self.profile == other.profile
            && self.foods == other.foods
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
            let held = |e: &Entry| e.input.matches(&input) && e.asked.holds_for(&input.blocked);
            if let Some(index) = entries.iter().position(held) {
                let found = entries.remove(index);
                let result = found.result.clone();
                entries.push(found);
                let _hit = crate::profile::Span::new("foraging.route.hit");
                if verifying() {
                    let fresh = search::find(&input)?;
                    assert_eq!(
                        fresh, result,
                        "{VERIFY_ENV}: a remembered search differs from a fresh one (actor {})",
                        input.actor
                    );
                    let n = VERIFIED.fetch_add(1, Ordering::Relaxed) + 1;
                    eprintln!("[route-cache-verify] hit {n} equals a fresh search (actor {})", input.actor);
                }
                return Ok(result);
            }
        }
        let _span = crate::profile::Span::new("foraging.route.search");
        let (result, asked) = search::find_watched(&input)?;
        let mut entries = self.0.lock().map_err(|_| "foraging route cache poisoned")?;
        if entries.len() == CAPACITY {
            entries.remove(0);
        }
        entries.push(Entry {
            input,
            asked,
            result: result.clone(),
        });
        Ok(result)
    }
}

#[cfg(test)]
mod differential;
#[cfg(test)]
mod tests;
