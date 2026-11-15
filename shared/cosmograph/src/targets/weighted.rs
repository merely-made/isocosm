// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Pass-start target weights for unplaced actors sharing a site and intake.
//! Store cumulative weights in identity order to preserve the native draw.

use super::*;
use crate::process::IntakePort;
use std::sync::{Arc, Mutex};

type CacheKey = (Id, Key, Option<IntakePort>);

#[derive(Debug, Default)]
pub(crate) struct Cache(Mutex<BTreeMap<CacheKey, Arc<Index>>>);

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Cache {
    fn get(&self, key: CacheKey, build: impl FnOnce() -> Index) -> Arc<Index> {
        if let Some(index) = self
            .0
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&key)
            .cloned()
        {
            return index;
        }
        let index = Arc::new(build());
        self.0
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(key)
            .or_insert(index)
            .clone()
    }
}

#[derive(Debug)]
struct Span {
    first: Id,
    count: u64,
    weight: u128,
    start: u128,
}

#[derive(Debug, Default)]
pub(super) struct Index {
    spans: Vec<Span>,
    total: u128,
}

impl Index {
    fn push(&mut self, first: Id, count: u64, weight: u128) {
        if weight > 0 {
            self.spans.push(Span {
                first,
                count,
                weight,
                start: self.total,
            });
            self.total += weight * u128::from(count);
        }
    }

    pub(super) fn pick(&self, actor: Id, draw: u64) -> Option<Id> {
        let prior = self.spans.partition_point(|span| span.first <= actor);
        let own = prior.checked_sub(1).and_then(|i| {
            let span = &self.spans[i];
            let offset = actor - span.first;
            (offset < span.count)
                .then(|| (span.start + u128::from(offset) * span.weight, span.weight))
        });
        let total = self.total - own.map_or(0, |(_, weight)| weight);
        if total == 0 {
            return None;
        }
        let mut pick = (u128::from(draw) * total) >> 64;
        if let Some((start, weight)) = own
            && pick >= start
        {
            pick += weight;
        }
        let span = &self.spans[self.spans.partition_point(|span| span.start <= pick) - 1];
        Some(span.first + ((pick - span.start) / span.weight) as u64)
    }
}

impl Simulation {
    /// The caller has checked the actor's own requirements. Sharing is
    /// valid only where prey eligibility is independent of actor identity,
    /// apart from self-exclusion, and reads this site's pass-start state.
    pub(super) fn weighted_targets(&self, actor: Id, p: &Process, place: Id) -> Option<Arc<Index>> {
        let frame = self.frame.as_ref()?;
        self.pass.as_ref()?;
        let selector = p.target.as_ref()?;
        if !selector.same_place
            || !selector.weighted
            || self.state.spatial_poses.contains_key(&actor)
            || p.requires.iter().any(|query| {
                reads_prey(query)
                    && !matches!(
                        query,
                        Query::Holds {
                            who: Binding::Target,
                            ..
                        } | Query::Alive(Binding::Target)
                    )
            })
        {
            return None;
        }
        let body = self.body_at_start(actor)?;
        let port = match &p.port_feeding {
            Some(feeding) => Some(crate::mosaic::ports::active(feeding.part(body).ok()?.1)?),
            None => None,
        };
        Some(frame.weighted_targets.get((place, p.id.clone(), port), || {
            let _span = crate::profile::Span::new("targets.weighted_index");
            let mut index = Index::default();
            for first in self.searched(selector, place) {
                let Some((_, count)) = self.target_span(first) else {
                    continue;
                };
                let Some(prey) = self.body_at_start(first) else {
                    continue;
                };
                if self.genesis.rules.dormant(prey)
                    || !self.target_bodies_match(body, prey, first, selector, p)
                    || p.requires
                        .iter()
                        .filter(|query| reads_prey(query))
                        .any(|query| {
                            self.process_query(actor, Some(first), place, None, p, query)
                                .is_err()
                        })
                {
                    continue;
                }
                index.push(first, count, self.prey_weight(body, first, p));
            }
            index
        }))
    }
}
