// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Turn groups: processes that share ground and name one group take first
//! place in rotation, so that no identity decides who takes first. The order
//! of same-tick passes stays (due, priority, rank, identity); rank is nothing
//! for a process naming no group, which therefore orders as it always did.

use crate::{rules::Process, schema::{Key, Tick}};
use std::collections::BTreeMap;

/// The members of each turn group at one priority, in identity order.
#[derive(Default)]
pub(crate) struct Turns(BTreeMap<(Key, i32), Vec<Key>>);

impl Turns {
    pub(crate) fn of<'a>(processes: impl Iterator<Item = &'a Process>) -> Self {
        let mut groups: BTreeMap<(Key, i32), Vec<Key>> = BTreeMap::new();
        for p in processes {
            if let Some(group) = &p.turn {
                groups.entry((group.clone(), p.priority)).or_default().push(p.id.clone());
            }
        }
        for members in groups.values_mut() {
            members.sort();
        }
        Self(groups)
    }

    /// `p`'s place among its group due at `due`: its index in identity order
    /// turned by the occasion the group falls due (the tick over the period,
    /// not its parity, so equal even periods still rotate). A pure function
    /// of the process and the tick, so every way of cutting an advance agrees.
    pub(crate) fn rank(&self, p: &Process, due: Tick) -> u64 {
        let Some(group) = &p.turn else { return 0; };
        let Some(members) = self.0.get(&(group.clone(), p.priority)) else { return 0; };
        let n = members.len() as u64;
        let at = members.iter().position(|m| *m == p.id).unwrap_or(0) as u64;
        let occasion = due / p.period.unwrap_or(1).max(1);
        (at + occasion) % n.max(1)
    }
}
