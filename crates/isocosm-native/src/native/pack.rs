// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Saved adventure proposals; native history owns registration and acceptance.

use cosmograph::{Command, Session, schema::{Id, Key, Tick}};
use cosmograph_overlay::{EventHandle, OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use super::GameBindings;

mod types;
mod records;
mod admission;
mod reading;
mod transition;
pub use types::*;
pub use admission::submit_pack;
pub use reading::{read_packs, preview_pack, active_content, accepted_choreography, ActiveContent};
pub use records::validate_bindings;
pub use transition::{Change, validate_transition};

fn dm(session: &Session, access: OrdinaryAccess) -> Result<(), String> {
    super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm { return Err("adventure catalogue and acceptance require the actual DM seat".into()); }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Record {
    Content { participant: Id, content: crate::founding::Content },
    Registered { participant: Id, definition: Definition },
    Previewed { participant: Id, key: Key, prepared: Prepared },
    Activated { participant: Id, key: Key, preview: EventHandle, forced: bool },
}
