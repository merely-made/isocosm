// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Identity-ordered component access (sim-execution D42), initially for
//! derived views. Callers supply native identities; stores allocate none.
//!
//! [`ReadStore`] separates queries from physical layout. Only [`SparseStore`]
//! is implemented here: interval, paged-column and singleton storage remain
//! E9 work. A derived store is rebuilt from accepted native state, not saved
//! as another authority. Source/history stamps belong to its caller.

use crate::schema::Id;
use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};

/// A component read by native identity, independently of its storage layout.
///
/// `iter` must yield each present identity exactly once, in strictly ascending
/// order, with the same value `get` reads. Membership is explicit: storing a
/// cohort's first identity does not insert its other members. A future
/// interval backend must declare how it presents its spans under this contract.
pub trait ReadStore<T> {
    type Iter<'a>: Iterator<Item = (Id, &'a T)>
    where
        Self: 'a,
        T: 'a;

    fn get(&self, id: Id) -> Option<&T>;
    fn iter(&self) -> Self::Iter<'_>;
}

/// Sparse, typed components keyed by caller-provided native identities.
///
/// Cloning shares the ordered map. The first subsequent write copies its keys
/// and payload handles; unchanged payloads stay shared and need not implement
/// `Clone`. `set` and `remove` never change another snapshot. As with `Arc`,
/// interior mutability inside a payload remains the payload's own contract.
/// There is deliberately no mutable payload reference or serialization here.
#[derive(Debug)]
pub struct SparseStore<T> {
    rows: Arc<BTreeMap<Id, Arc<T>>>,
}

impl<T> Clone for SparseStore<T> {
    fn clone(&self) -> Self {
        Self {
            rows: self.rows.clone(),
        }
    }
}

impl<T> Default for SparseStore<T> {
    fn default() -> Self {
        Self {
            rows: Arc::new(BTreeMap::new()),
        }
    }
}

impl<T> SparseStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.rows.get(&id).map(Arc::as_ref)
    }

    /// Inserts or replaces one component. No other identity is affected.
    pub fn set(&mut self, id: Id, value: T) {
        Arc::make_mut(&mut self.rows).insert(id, Arc::new(value));
    }

    /// Removes one component, reporting whether it was present. A missing
    /// identity does not copy a shared map.
    pub fn remove(&mut self, id: Id) -> bool {
        if !self.rows.contains_key(&id) {
            return false;
        }
        Arc::make_mut(&mut self.rows).remove(&id).is_some()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            rows: self.rows.iter(),
        }
    }
}

impl<T> ReadStore<T> for SparseStore<T> {
    type Iter<'a>
        = Iter<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn get(&self, id: Id) -> Option<&T> {
        SparseStore::get(self, id)
    }

    fn iter(&self) -> Self::Iter<'_> {
        SparseStore::iter(self)
    }
}

/// Borrowed sparse rows in ascending native identity order.
pub struct Iter<'a, T> {
    rows: std::collections::btree_map::Iter<'a, Id, Arc<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next().map(|(&id, value)| (id, value.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rows.size_hint()
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> std::iter::FusedIterator for Iter<'_, T> {}

/// Intersects two component memberships by merging their ordered reads.
///
/// Yields `(identity, left, right)` once per common identity in ascending
/// order, independent of insertion/removal history. It borrows both stores,
/// allocates nothing and reads at most the sum of their row counts.
pub fn intersection<'a, A: 'a, B: 'a, L, R>(
    left: &'a L,
    right: &'a R,
) -> impl Iterator<Item = (Id, &'a A, &'a B)> + 'a
where
    L: ReadStore<A> + ?Sized,
    R: ReadStore<B> + ?Sized,
{
    let mut left = left.iter().peekable();
    let mut right = right.iter().peekable();
    std::iter::from_fn(move || {
        loop {
            let (a, b) = (left.peek()?.0, right.peek()?.0);
            match a.cmp(&b) {
                Ordering::Less => {
                    left.next();
                },
                Ordering::Greater => {
                    right.next();
                },
                Ordering::Equal => {
                    let (id, a) = left.next()?;
                    let (_, b) = right.next()?;
                    return Some((id, a, b));
                },
            }
        }
    })
}
