// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arrival {
    pub tick: Tick,
    pub source: Option<Id>,
    pub strength: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reach {
    pub arrivals: BTreeMap<Key, BTreeMap<Id, Arrival>>,
}

impl Reach {
    /// Earliest arrival, then strongest route, then lowest predecessor id.
    /// Positive travel times make the predecessor graph acyclic.
    pub fn seed(&mut self, event: &Event, sites: &BTreeMap<Id, Site>) -> Result<()> {
        if !sites.contains_key(&event.place) {
            return Err("event has no site".into());
        }
        if event.legend {
            return Ok(());
        }
        let mut entries = BTreeMap::new();
        let mut queue =
            BTreeSet::from([(event.tick, u32::MAX - event.strength, event.place, None)]);
        while let Some((tick, inverse, place, source)) = queue.pop_first() {
            if entries.contains_key(&place) {
                continue;
            }
            let strength = u32::MAX - inverse;
            entries.insert(
                place,
                Arrival {
                    tick,
                    source,
                    strength,
                },
            );
            for route in &sites[&place].routes {
                if route.travel == 0
                    || !sites.contains_key(&route.to)
                    || route.transmission > 1_000_000
                {
                    return Err("invalid reach route".into());
                }
                let strength =
                    (u64::from(strength) * u64::from(route.transmission) / 1_000_000) as u32;
                if strength > 0 {
                    queue.insert((
                        tick.checked_add(route.travel)
                            .ok_or("arrival time overflow")?,
                        u32::MAX - strength,
                        route.to,
                        Some(place),
                    ));
                }
            }
        }
        self.arrivals.insert(event.id.clone(), entries);
        Ok(())
    }
    pub fn strength(&self, event: &Event, place: Id, tick: Tick, policy: &FieldPolicy) -> u32 {
        if tick < event.tick {
            return 0;
        }
        if event.legend {
            return policy.legend_floor;
        }
        self.arrivals
            .get(&event.id)
            .and_then(|a| a.get(&place))
            .map_or(0, |a| {
                if tick < a.tick {
                    return 0;
                }
                a.strength.saturating_sub(
                    tick.saturating_sub(a.tick)
                        .saturating_mul(u64::from(policy.decay_per_tick))
                        .min(u64::from(u32::MAX)) as u32,
                )
            })
    }
    pub fn learning_path(&self, event: &Event, place: Id) -> Vec<Id> {
        let Some(arrivals) = self.arrivals.get(&event.id) else {
            return vec![];
        };
        let mut path = Vec::new();
        let mut next = Some(place);
        let mut seen = BTreeSet::new();
        while let Some(id) = next {
            if !seen.insert(id) {
                break;
            }
            let Some(a) = arrivals.get(&id) else {
                break;
            };
            path.push(id);
            next = a.source;
        }
        path
    }
    /// Integrated integer exposure over [from, until), capped at certainty.
    /// Closed-form arithmetic avoids iterating through unobserved lifetimes.
    pub fn exposure(
        &self,
        event: &Event,
        place: Id,
        from: Tick,
        until: Tick,
        policy: &FieldPolicy,
    ) -> u64 {
        let from = from.max(event.tick);
        if until <= from {
            return 0;
        }
        if event.legend {
            return (until - from)
                .saturating_mul(u64::from(policy.legend_floor))
                .min(1_000_000);
        }
        let Some(a) = self
            .arrivals
            .get(&event.id)
            .and_then(|places| places.get(&place))
        else {
            return 0;
        };
        let from = from.max(a.tick);
        if until <= from {
            return 0;
        }
        let first = u64::from(self.strength(event, place, from, policy));
        if first == 0 {
            return 0;
        }
        let decay = u64::from(policy.decay_per_tick);
        let n = (until - from).min(first.div_ceil(decay));
        let sum = u128::from(n) * u128::from(first)
            - u128::from(decay) * u128::from(n) * u128::from(n - 1) / 2;
        sum.min(1_000_000) as u64
    }
    pub fn collect(&mut self, tick: Tick, policy: &FieldPolicy) {
        self.arrivals.retain(|_, places| {
            places.values().any(|a| {
                tick < a.tick
                    || tick
                        .saturating_sub(a.tick)
                        .saturating_mul(u64::from(policy.decay_per_tick))
                        < u64::from(a.strength)
            })
        });
    }
}
