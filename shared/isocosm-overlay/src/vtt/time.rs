// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::EventHandle;

/// Time at the table (rulings 104, 105, 188, 246, 248). The DM proposes a
/// downtime; each player's yes is that player's own intent, naming the
/// proposal by the event it made in the record; the sim runs the span only
/// when every player has said yes (ruling 246), since advancing the trunk
/// past what another's foreground can bear is a proposal needing consent
/// (rulings 104, 105). A campaign's sim is switched off or on; with it off
/// the plain tabletop plays as today and its facts are still written as
/// notes (rulings 188, 248).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeIntent {
    /// The DM declares a downtime of this many ticks (`isonetry`'s
    /// `TimeAdvanced`), which waits for every player's yes.
    Downtime {
        ticks: u64,
    },
    /// A player's yes to a proposed downtime, named by the event the proposal
    /// made.
    Consent {
        downtime: EventHandle,
    },
    SimOff,
    SimOn,
}
