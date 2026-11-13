// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Small builders the generator and the probes share.

use crate::{rules::*, schema::*};
use std::collections::BTreeSet;

pub(super) fn set(values: &[&str]) -> BTreeSet<Key> {
    values.iter().map(|s| s.to_string()).collect()
}
pub(crate) fn process(id: &str, causation: Causation, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: set(&["sim:matter", "sim:nonnegative"]),
        note: false,
        port_feeding: None,
        goal_directives: Vec::new(), turn: None,
    }
}
