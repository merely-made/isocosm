// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{history::Command, rules::{Binding, Rules}, simulation::Receipt};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitOrigin {
    Process { process: Key, actor: Id, target: Option<Id> },
    Command(Box<Command>),
    Clock { tick: Tick },
    NativeSpread { parent:Box<DebitOrigin>, operation:SpreadDebit,
        identity:crate::spread::Identity, at:crate::spread::Substrate },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebitRequest {
    pub action: Key,
    pub origin: DebitOrigin,
    pub tick: Tick,
    pub revision: Key,
    pub operation: u64,
    pub ordinal: u64,
    pub source: StockSource,
    pub account: Key,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LotQuantity { pub key: Key, pub amount: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebitSelection {
    pub request: DebitRequest,
    pub lots: Vec<LotQuantity>,
    pub unowned: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebitPreview {
    pub requests: Vec<DebitRequest>,
    pub receipt: Option<Receipt>,
    pub refusal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedActed { pub receipt: Receipt, pub debits: Vec<DebitRequest>, pub claims: Vec<Key> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedCommandResult { pub outcome: String, pub debits: Vec<DebitRequest>, pub claims: Vec<Key> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DirectOperation { Wound, Edit, SpreadOverflow, ForkStrain, SpreadPlace }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperationRule {
    Process { process: Key, ordinal: u64 },
    Command(DirectOperation),
    Spread { operation:SpreadDebit, identity:crate::spread::Identity, at:crate::spread::Substrate },
    ProducedSpread { origin:ProducedOrigin, operation:SpreadDebit, at:crate::spread::Substrate,
        input:OutputReference },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpreadDebit { Overflow, ForkStrain, Place }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpreadOutput { Route(usize), GeneratedStrain, Placement }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedOutput { pub input:Key, pub amount:u64, pub output:SpreadOutput }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadOutputWitness {
    pub action:Key, pub operation:u64, pub output:SpreadOutput,
    pub source:StockSource, pub identity:crate::spread::Identity, pub amount:u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadPreview { pub debits:DebitPreview, pub outputs:Vec<SpreadOutputWitness> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination { Binding(Binding), Source(StockSource) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputClaim {
    pub input: Key,
    pub amount: u64,
    pub destination: Destination,
    pub account: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    CloseConsumed,
    Retain { outputs: Vec<OutputClaim>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        closed: Vec<LotQuantity> },
    Outputs { outputs:Vec<RetainedOutput>, closed:Vec<LotQuantity> },
    RetainReferenced { outputs:Vec<ReferencedOutputClaim>, closed:Vec<ClaimQuantity> },
    OutputsReferenced { outputs:Vec<ReferencedRetainedOutput>, closed:Vec<ClaimQuantity> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRule { pub operation: OperationRule, pub disposition: Disposition }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceRule {
    Process { process: Key, operation: u64, from: Binding, account: Key,
        lots: Vec<LotQuantity>, unowned: u64 },
    Spread { operation:SpreadDebit, identity:crate::spread::Identity, at:crate::spread::Substrate,
        lots:Vec<LotQuantity>, unowned:u64 },
    ProducedProcess { process:Key, operation:u64, from:Binding, output:ProducedQuantity },
    ProducedSpread { origin:ProducedOrigin, operation:SpreadDebit, at:crate::spread::Substrate,
        output:ProducedQuantity },
    Placement { identity:crate::spread::Identity, at:crate::spread::Substrate,
        source:StockSource, account:Key, lots:Vec<LotQuantity>, unowned:u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockRules { pub selections: Vec<SourceRule>, pub dispositions: Vec<DispositionRule> }

impl StockRules {
    pub fn validate(&self, rules: &Rules) -> Result<()> { super::validate::stock_rules(self,rules) }
}
pub(crate) fn direct_operation(command: &Command) -> Result<DirectOperation> {
    match command {
        Command::Wound { .. } => Ok(DirectOperation::Wound),
        Command::Edit { by: Some(_), .. } => Ok(DirectOperation::Edit),
        Command::Spread(crate::spread::Command::Overflow { .. }) => Ok(DirectOperation::SpreadOverflow),
        Command::Spread(crate::spread::Command::ForkStrain { .. }) => Ok(DirectOperation::ForkStrain),
        Command::Spread(crate::spread::Command::Place(_)) => Ok(DirectOperation::SpreadPlace),
        _ => Err("owned admission requires a real direct Wound, member Edit or debit-producing Spread command".into()),
    }
}
