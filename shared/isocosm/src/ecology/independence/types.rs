// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{rules::AccountKind, schema::*, ecology::Selection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeReference {
    Account(Key), Trait(Key), Condition(Key), Relation(Key), Shape(Key), Function(Key),
    Process(Key), Lineage(Key), Material(Key), Kind(Key), System(Key),
    /// Unsupported functional extensions stay in coverage instead of disappearing.
    RuleField(Key), WorldField(Key),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectPhase { Commitment, Outcome, Risk }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectStep { Item(u32), Then, Otherwise }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectAddress { pub process: Key, pub phase: EffectPhase, pub path: Vec<EffectStep> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePair { pub candidate: NativeReference, pub compared: NativeReference }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanicalAxis { Cadence, PaidStock, TypedFlow }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraNoun {
    pub noun: NativeReference,
    pub consumer: Selection,
    pub effect: EffectAddress,
    pub compare: EffectAddress,
    pub equivalences: Vec<ReferencePair>,
    pub axis: MechanicalAxis,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScalarSource {
    Period(Key), Priority(Key), Amount(ScalarEffect),
    Input { effect: ScalarEffect, account: Key }, Output { effect: ScalarEffect, account: Key },
    Control { effect: ScalarEffect, field: ControlField },
    Requirement { process: Key, index: u32 },
    NeedBelow(Key),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ControlField { ConditionDelta, Cells }

/// Exact top-level consumed fields; guarded/dynamic fields are refused in this slice.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScalarEffect { pub process: Key, pub commitment: bool, pub index: u32 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension { Ticks, Priority, Stock(AccountKind), MatterQuantity, Cells, ConditionLevel, Count }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conservation { NativeMatter, OtherStock, NotStock }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarDeclaration {
    pub name: Key,
    pub source: ScalarSource,
    pub dimension: Dimension,
    pub consumer: Key,
    pub cadence: Tick,
    pub conservation: Conservation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependenceSpec { pub version: u32, pub extras: Vec<ExtraNoun>, pub scalars: Vec<ScalarDeclaration> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationFamily { Transfer, Transform, Spend, Eat, Convert, Birth, Condition, Wound, Allocate, Practice, Ease, Rot }

/// An explicit correspondence token; native identities remain in the source references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockType {
    Matter { lineage: u32, world: bool, reserve: bool, provision: bool }, Energy, Attention, Time, Obligation, Strain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockPort { pub binding: crate::rules::Binding, pub account: u32, pub kind: StockType, pub quantity: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MechanicalProjection {
    pub cadence: Tick,
    pub priority: i32,
    pub family: OperationFamily,
    pub inputs: Vec<StockPort>,
    pub outputs: Vec<StockPort>,
    /// Actual non-stock controls, excluding display text and random keys/slots.
    pub controls: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MechanicalEvidence {
    pub noun: NativeReference,
    pub consumer: Selection,
    pub effect: EffectAddress,
    pub compared: EffectAddress,
    pub family: OperationFamily,
    pub compared_family: OperationFamily,
    pub axis: MechanicalAxis,
    pub actual: MechanicalProjection,
    pub baseline: MechanicalProjection,
    pub accepted_actions: Vec<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalarValue { Unsigned(u64), Signed(i64) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumedScalar { pub declaration: ScalarDeclaration, pub value: ScalarValue }
