// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use super::table::{ActionKey, Cell};
use crate::EntityHandle;

/// The VTT's handoff vocabulary: a ruleset's resolved action in the sim's
/// terms (rulings 114, 123, 154). The dice and the beats stay at the table,
/// since the sim records the band and never the dice (the record's §5.1);
/// what crosses is what the action changed, which must pass the sim's
/// invariants, the accounts conserved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VttHandoff {
    Resolved(Resolved),
}

/// One adjudicated action, resolved once by the ruleset and applied by every
/// peer (`isonetry`'s `ActionResolved`, less its rolls and beats).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolved {
    /// The table's request this answers; applying two resolutions that carry
    /// the same id applies one of them.
    pub request: RequestId,
    /// The ruleset that resolved it; Pathfinder 2e is calibrated first
    /// (ruling 249).
    pub ruleset: RulesetKey,
    pub actor: EntityHandle,
    pub target: EntityHandle,
    pub action: ActionKey,
    pub harm: Harm,
    /// Put out of play by the rules, so the substrate skips their turns and
    /// refuses them as targets.
    pub defeated: Vec<EntityHandle>,
    /// Dead, which is final in the sim (ruling 61); the table's reversal by
    /// rules is a later re-embodiment (ruling 62).
    pub dead: Vec<EntityHandle>,
    pub conditions: Vec<Condition>,
    /// Forced movement: who this action relocated, and to where in the site.
    pub displaced: Vec<(EntityHandle, Cell)>,
    pub transferred: Vec<Transfer>,
    /// Whether the ruleset that resolved this passed ruling 114's check. The
    /// mark rides on every receipt and save of an uncalibrated campaign, and
    /// there is no standing banner (rulings 189, 250).
    pub calibration: Calibration,
}

/// The table's own identity for a request, echoed so a resolution applies
/// once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

/// A ruleset named by opaque key, as a pack names itself (the record's
/// §5.1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RulesetKey(pub String);

/// A condition applied or cleared, `(subject, name, magnitude)`: a magnitude
/// of zero clears it. The substrate stores the number blind; only the rules
/// know what it means, and the sim reads it through the ledger (ruling 38).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    pub subject: EntityHandle,
    pub condition: String,
    pub magnitude: i64,
}

/// One item moved between holders atomically: possession is asserted, so a
/// transfer is a fact in the record (rulings 53, 94).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    pub item: EntityHandle,
    pub from: EntityHandle,
    pub to: EntityHandle,
}

/// Whether the resolving ruleset has passed ruling 114's calibration
/// against the sim's own model, on the bench (rulings 189, 249, 250).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Calibration {
    Calibrated,
    Uncalibrated,
}

/// Harm in the sim's terms (ruling 123): vigour drained first, and wounds to
/// parts of the body's tree when a blow lands hard or the vigour is gone. A
/// ruleset's hit points calibrate against vigour more than wounds (the
/// record's §3.3.1). Eponym's module carries the same shape; a core `Harm`
/// is a candidate change put to Mark, not made (ruling 197).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Harm {
    pub vigour_drained: u64,
    pub wounds: Vec<Wound>,
}

/// A wound to one part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wound {
    pub part: PartHandle,
    pub severity: WoundSeverity,
}

/// How badly a part is wounded, from a loss that heals to a part severed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WoundSeverity {
    Minor,
    Serious,
    Crippling,
    Severed,
}

/// An opaque reference to one part of a body's tree, minted by the sim.
/// Defined here rather than in the core for the same reason as [`Harm`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartHandle(pub u64);
