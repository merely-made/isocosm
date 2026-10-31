// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded FIFO storage for readings whose physical inputs are bound by owners.
use std::collections::{BTreeMap, VecDeque};

pub(super) struct Memo<K, V> {
    entries: BTreeMap<K, V>,
    order: VecDeque<K>,
    capacity: usize,
}
impl<K: Ord + Clone, V> Memo<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }
    pub fn enabled(&self) -> bool {
        self.capacity > 0
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }
    pub fn insert(&mut self, key: K, value: V) {
        if !self.enabled() {
            return;
        }
        if let Some(old) = self.entries.get_mut(&key) {
            *old = value;
            return;
        }
        if self.entries.len() == self.capacity {
            self.entries.remove(&self.order.pop_front().unwrap());
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, value);
    }
    #[cfg(test)]
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
        self.entries.clear();
        self.order.clear();
    }
}
