// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Each player signs their current body's explicit native departure (859/860).
//! Native paced acts own costs and journeys; these records advance no clock.

use isocosm::{Session, Command, schema::{Id, Key, Tick}};
use isocosm_overlay::{EventHandle, OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod types;
pub use types::{DebitTerm, DepartureInput, JourneyInput, JourneyEnvelope, Witness,
    Departure, DepartureKind, Declaration, Approval, DepartureResult, Completion, Proposal};
use types::Record;
mod records;
mod admission;
pub use admission::{submit_journey, quote_departures, preview_departure};
mod reading;
pub use reading::{JourneyView, read_journeys};
mod transition;
pub use transition::{Change,validate_transition};

pub(super) fn validate_bindings(session: &Session, proposals: &BTreeMap<Key,Proposal>) -> Result<(),String> {
    records::validate(session,proposals)
}
