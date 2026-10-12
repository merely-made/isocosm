// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use isocosm::history::Saved as NativeSaved;
use serde::{Deserialize, Serialize};

pub(crate) const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorldSave {
    pub format: u32,
    pub native: NativeSaved,
    // Unknown modes are carried as bytes, never parsed as JSON or native facts.
    pub sections: BTreeMap<String, Vec<u8>>,
}

impl WorldSave {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|why| why.to_string())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|why| why.to_string())
    }
}
