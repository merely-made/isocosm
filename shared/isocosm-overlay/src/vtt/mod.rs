// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The VTT's side of the overlay contract (VTT overlay plan §3; wing design
//! record §5.7), the game that fills the handoff. The table's acts cross per
//! tick as one batch, never a call per token (ruling 244; §5.2 point 1); the
//! DM's assertions are the world editor's (rulings 89, 156, 190); downtime
//! runs when every player has said yes (rulings 104, 105, 246); a campaign
//! may switch the sim off or on (ruling 188), a sim-off campaign still
//! writing its facts as notes (ruling 248); a party travels the place graph
//! (rulings 72, 205); the arcs the sim runs reach the DM as hooks (ruling
//! 191). The ruleset's resolved action comes back through the handoff in
//! the sim's terms, marked calibrated or not (rulings 114, 189, 249, 250).
//! No dev intents: the DM's edit mode is play at the table (ruling 156).
//! See the crate README for the table mapping `isonetry`'s `GameEvent` onto
//! this module.

mod assertion;
mod handoff;
mod hook;
mod table;
mod time;
mod travel;

pub use assertion::{Assertion, CellEdit, Fact, MapEdit, NewCharacter};
pub use handoff::{Calibration, Condition, RequestId, Resolved, RulesetKey, Transfer, VttHandoff};
pub use hook::HookIntent;
pub use table::{Cell, TableAct, TableActKind, TableBatch};
pub use time::TimeIntent;
pub use travel::{Pace, Travel};

use serde::{Deserialize, Serialize};

use crate::{HandoffEnvelope, IntentEnvelope};

/// Everything a VTT host may send the sim: the table's acts for a tick, an
/// assertion from edit mode, a change to the campaign's time, a party's
/// travel, or what the DM does with a hook.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VttIntent {
    Table(TableBatch),
    Assert(Assertion),
    Time(TimeIntent),
    Travel(Travel),
    Hook(HookIntent),
}

impl VttIntent {
    /// Whether this intent asserts facts from edit mode (rulings 89, 156),
    /// which a sim-off campaign still writes as notes (ruling 248).
    pub fn is_assertion(&self) -> bool {
        matches!(self, Self::Assert(_))
    }
}

/// A VTT intent, stamped for its tick and naming its participant, the DM or
/// a player.
pub type VttIntentEnvelope = IntentEnvelope<VttIntent>;

/// The VTT's handoff envelope: a ruleset's resolved action, handed back in
/// the sim's terms.
pub type VttHandoffEnvelope = HandoffEnvelope<VttHandoff>;
