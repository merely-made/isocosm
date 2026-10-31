// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit quantity selections cross as the native stock vocabulary's bytes.

use crate::{ActKey, EntityHandle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockAct {
    pub subject: EntityHandle,
    pub process: ActKey,
    pub target: Option<EntityHandle>,
    /// Serialized native DebitSelections, tied to the preview's exact cut.
    pub selections: Vec<u8>,
}
