// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, schema::*};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

/// A contiguous identity interval sharing the complete causal state. No means,
/// rounded bins or lost tails: different reserves/traits remain different rows,
/// except a band of the population model (the execution plan's E4, D28), whose
/// members' bodies differ and whose `pool` holds what they hold beyond
/// `count` times its representative `entity`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cohort {
    pub count: u64,
    /// Shared between Session copies; a write copies the one entity
    /// (`Arc::make_mut`), so a clone of the population copies only the map.
    pub entity: Arc<Entity>,
    /// A band's ledger beyond its representative's, per account, signed;
    /// empty for any other cohort, which serializes and hashes as before.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub pool: Pool,
}

/// A band's signed ledger (D28).
pub type Pool = BTreeMap<Key, i128>;

/// `pool` split for a piece of `count` members out of `of`: its share of
/// each account, truncated toward nought, the remainder left behind.
fn share(pool: &Pool, count: u64, of: u64) -> Pool {
    pool.iter()
        .map(|(k, v)| (k.clone(), v * i128::from(count) / i128::from(of)))
        .filter(|(_, v)| *v != 0)
        .collect()
}

/// What remains of `pool` once `taken` is drawn from it.
fn less(pool: &Pool, taken: &[&Pool]) -> Pool {
    let mut left = pool.clone();
    for t in taken {
        for (k, v) in *t {
            *left.entry(k.clone()).or_default() -= v;
        }
    }
    left.retain(|_, v| *v != 0);
    left
}

impl Cohort {
    /// The cohort's entity for writing: copied first if a Session copy shares it.
    pub fn entity_mut(&mut self) -> &mut Entity {
        Arc::make_mut(&mut self.entity)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Population {
    pub groups: BTreeMap<Id, Cohort>,
    pub next_id: Id,
}

impl Population {
    pub fn count(&self) -> u64 {
        self.groups.values().map(|g| g.count).sum()
    }
    pub fn get_shared(&self, id: Id) -> Option<&Arc<Entity>> {
        let (&first, group) = self.groups.range(..=id).next_back()?;
        (id - first < group.count).then_some(&group.entity)
    }
    pub fn get(&self, id: Id) -> Option<&Entity> {
        let (&first, group) = self.groups.range(..=id).next_back()?;
        (id - first < group.count).then_some(&group.entity)
    }
    pub fn insert(&mut self, entity: impl Into<Arc<Entity>>, count: u64) -> Result<Id> {
        if count == 0 {
            return Err("empty cohort".into());
        }
        let first = self.next_id;
        self.next_id = first.checked_add(count).ok_or("identity exhausted")?;
        self.groups.insert(first, Cohort { count, entity: entity.into(), pool: Pool::new() });
        Ok(first)
    }
    pub fn lift(&mut self, id: Id) -> Result<&mut Entity> {
        Ok(self.lift_shared(id)?.entity_mut())
    }
    /// Lifts `id` into a group of its own still sharing its entity, so a
    /// caller that replaces the entity copies nothing (ruling 975).
    pub fn lift_shared(&mut self, id: Id) -> Result<&mut Cohort> {
        let (&first, group) = self
            .groups
            .range(..=id)
            .next_back()
            .ok_or("unknown entity")?;
        if id - first >= group.count {
            return Err("unknown entity".into());
        }
        let group = group.clone();
        let end = first + group.count;
        self.groups.remove(&first);
        // A band's pool goes with its pieces by their counts, the remainder
        // to the member lifted.
        let (before, after) = (id - first, end - id - 1);
        let pools = [share(&group.pool, before, group.count), share(&group.pool, after, group.count)];
        let own = less(&group.pool, &[&pools[0], &pools[1]]);
        let [pool_before, pool_after] = pools;
        if id > first {
            self.groups.insert(
                first,
                Cohort {
                    count: before,
                    entity: group.entity.clone(),
                    pool: pool_before,
                },
            );
        }
        if id + 1 < end {
            self.groups.insert(
                id + 1,
                Cohort {
                    count: after,
                    entity: group.entity.clone(),
                    pool: pool_after,
                },
            );
        }
        self.groups.insert(
            id,
            Cohort {
                count: 1,
                entity: group.entity,
                pool: own,
            },
        );
        Ok(self.groups.get_mut(&id).unwrap())
    }
    pub fn restrict(&mut self, protected: &BTreeSet<Id>) {
        self.restrict_logged(protected, &mut |_, _| {});
    }
    /// Restricts, first showing `log` every group it is about to change, by
    /// first identity, as it stands; a group not yet there shows as `None`.
    pub(crate) fn restrict_logged(
        &mut self,
        protected: &BTreeSet<Id>,
        log: &mut dyn FnMut(Id, Option<&Cohort>),
    ) {
        for &id in protected {
            if let Some((&first, _)) = self.groups.range(..=id).next_back() {
                for key in [first, id, id.saturating_add(1)] {
                    log(key, self.groups.get(&key));
                }
            }
            let _ = self.lift(id);
        }
        let mut merged: BTreeMap<Id, Cohort> = BTreeMap::new();
        for (first, group) in std::mem::take(&mut self.groups) {
            if let Some((&previous, last)) = merged.last_key_value()
                && previous + last.count == first
                && last.entity == group.entity
                && last.pool.is_empty()
                && group.pool.is_empty()
                && !protected.contains(&previous)
                && !protected.contains(&first)
            {
                log(previous, Some(last));
                log(first, Some(&group));
                merged.get_mut(&previous).unwrap().count += group.count;
                continue;
            }
            merged.insert(first, group);
        }
        self.groups = merged;
    }
    pub fn normalized(&self) -> Self {
        let mut result = self.clone();
        result.restrict(&BTreeSet::new());
        result
    }
    pub fn totals(&self) -> Result<BTreeMap<Key, u128>> {
        let mut totals = BTreeMap::new();
        let mut pooled: BTreeMap<Key, i128> = BTreeMap::new();
        for group in self.groups.values() {
            for (key, value) in &crate::anatomy::books(&group.entity) {
                let slot = totals.entry(key.clone()).or_insert(0u128);
                *slot = slot
                    .checked_add(u128::from(*value) * u128::from(group.count))
                    .ok_or("account total overflow")?;
            }
            for (key, value) in &group.pool {
                *pooled.entry(key.clone()).or_default() += value;
            }
        }
        for (key, value) in pooled {
            let slot = totals.entry(key).or_insert(0u128);
            let total = i128::try_from(*slot).map_err(|_| "account total overflow")? + value;
            *slot = u128::try_from(total).map_err(|_| "a pool holds more than its band")?;
        }
        Ok(totals)
    }
    pub fn validate(&self, limit: u64) -> Result<()> {
        let mut end = 0;
        let mut total = 0u64;
        for (&first, group) in &self.groups {
            if group.count == 0 || first < end {
                return Err("overlapping or empty population".into());
            }
            end = first.checked_add(group.count).ok_or("identity overflow")?;
            total = total
                .checked_add(group.count)
                .ok_or("population overflow")?;
        }
        if end > self.next_id || total > limit {
            return Err("population limit or next identity invalid".into());
        }
        Ok(())
    }
}
