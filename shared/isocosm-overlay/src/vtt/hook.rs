// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::EventHandle;

/// What the DM does with an arc the sim offers as a hook (rulings 103, 191):
/// take it up, drop it, or reshape it. An arc is a derived thread in the
/// record, named here by the note its consequential goal left, an event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookIntent {
    TakeUp {
        arc: EventHandle,
    },
    Drop {
        arc: EventHandle,
    },
    /// Taken up with the DM's own turn on it, a note the record keeps beside
    /// the arc.
    Reshape {
        arc: EventHandle,
        note: String,
    },
}
