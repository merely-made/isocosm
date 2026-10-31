// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the creature skin reads from one part (case 1049).
//!
//! Here rather than beside the skin so the sim can produce it without a
//! renderer: Cosmograph reads its entities into these, and isometer's skin
//! draws from them. Presentation input only; nothing here is anatomy.

use std::collections::BTreeMap;

use crate::PartId;

/// One part, as far as its look needs: the matter it holds, by key and
/// amount, and the cells each function holds out of its capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartSkin {
    pub part: PartId,
    pub matter: BTreeMap<String, u64>,
    pub functions: BTreeMap<String, u32>,
    pub capacity: u32,
}

impl PartSkin {
    /// A part with nothing read into it yet.
    pub fn new(part: PartId) -> Self {
        Self { part, matter: BTreeMap::new(), functions: BTreeMap::new(), capacity: 0 }
    }
}
