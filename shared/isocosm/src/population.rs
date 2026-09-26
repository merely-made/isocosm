// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A contiguous identity interval sharing the complete causal state. No means,
/// rounded bins or lost tails: different reserves/traits remain different rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cohort {
    pub count: u64,
    pub entity: Entity,
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
    pub fn get(&self, id: Id) -> Option<&Entity> {
        let (&first, group) = self.groups.range(..=id).next_back()?;
        (id - first < group.count).then_some(&group.entity)
    }
    pub fn insert(&mut self, entity: Entity, count: u64) -> Result<Id> {
        if count == 0 {
            return Err("empty cohort".into());
        }
        let first = self.next_id;
        self.next_id = first.checked_add(count).ok_or("identity exhausted")?;
        self.groups.insert(first, Cohort { count, entity });
        Ok(first)
    }
    pub fn lift(&mut self, id: Id) -> Result<&mut Entity> {
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
        if id > first {
            self.groups.insert(
                first,
                Cohort {
                    count: id - first,
                    entity: group.entity.clone(),
                },
            );
        }
        if id + 1 < end {
            self.groups.insert(
                id + 1,
                Cohort {
                    count: end - id - 1,
                    entity: group.entity.clone(),
                },
            );
        }
        self.groups.insert(
            id,
            Cohort {
                count: 1,
                entity: group.entity,
            },
        );
        Ok(&mut self.groups.get_mut(&id).unwrap().entity)
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
        for group in self.groups.values() {
            for (key, value) in &group.entity.accounts {
                let slot = totals.entry(key.clone()).or_insert(0u128);
                *slot = slot
                    .checked_add(u128::from(*value) * u128::from(group.count))
                    .ok_or("account total overflow")?;
            }
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
