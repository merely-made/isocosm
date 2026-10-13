// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{history::Command, rules::{Binding, Rules}, simulation::Receipt};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitOrigin {
    Process { process: Key, actor: Id, target: Option<Id> },
    Command(Box<Command>),
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
pub enum DirectOperation { Wound, Edit, SpreadOverflow, ForkStrain }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperationRule {
    Process { process: Key, ordinal: u64 },
    Command(DirectOperation),
}

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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRule { pub operation: OperationRule, pub disposition: Disposition }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceRule {
    Process { process: Key, operation: u64, from: Binding, account: Key,
        lots: Vec<LotQuantity>, unowned: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockRules { pub selections: Vec<SourceRule>, pub dispositions: Vec<DispositionRule> }

impl StockRules {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        let mut dispositions = BTreeSet::new();
        for rule in &self.dispositions {
            if !dispositions.insert(rule.operation.clone()) { return Err("stock operation disposition is ambiguous".into()); }
            if let OperationRule::Process { process, .. } = &rule.operation {
                crate::validation::key(process)?;
                if !rules.processes.contains_key(process) { return Err("stock disposition names no process".into()); }
            }
            if let Disposition::Retain { outputs, closed } = &rule.disposition {
                if outputs.is_empty() { return Err("retained stock disposition has no explicit outputs".into()); }
                for output in outputs {
                    crate::validation::key(&output.input)?;
                    matter_key(rules, &output.account)?;
                    if output.amount == 0 { return Err("retained stock claim needs a positive exact quantity".into()); }
                }
                let mut seen = BTreeSet::new();
                for portion in closed {
                    crate::validation::key(&portion.key)?;
                    if portion.amount == 0 || !seen.insert(&portion.key) {
                        return Err("closed stock portion needs an exact positive nonduplicated input lot".into());
                    }
                }
            }
        }
        let mut selections = BTreeSet::new();
        for SourceRule::Process { process, operation, from, account, lots, unowned } in &self.selections {
            crate::validation::key(process)?;
            matter_key(rules, account)?;
            if !rules.processes.contains_key(process) || *from == Binding::Part
                || !selections.insert((process.clone(), *operation, format!("{from:?}"), account.clone())) {
                return Err("stock source selection is absent, ambiguous or not a native body/site binding".into());
            }
            let mut seen = BTreeSet::new();
            let mut amount = *unowned;
            for lot in lots {
                crate::validation::key(&lot.key)?;
                if lot.amount == 0 || !seen.insert(&lot.key) { return Err("stock selector repeats a lot or names zero quantity".into()); }
                amount = amount.checked_add(lot.amount).ok_or("stock selector quantity overflow")?;
            }
            if amount == 0 { return Err("stock selector needs an exact positive debit".into()); }
        }
        Ok(())
    }
}

fn matter_key(rules: &Rules, key: &str) -> Result<()> {
    crate::validation::key(key)?;
    if !crate::meaning::matter(rules, key) { return Err("stock declaration names no native matter account".into()); }
    Ok(())
}

pub(crate) fn direct_operation(command: &Command) -> Result<DirectOperation> {
    match command {
        Command::Wound { .. } => Ok(DirectOperation::Wound),
        Command::Edit { by: Some(_), .. } => Ok(DirectOperation::Edit),
        Command::Spread(crate::spread::Command::Overflow { .. }) => Ok(DirectOperation::SpreadOverflow),
        Command::Spread(crate::spread::Command::ForkStrain { .. }) => Ok(DirectOperation::ForkStrain),
        _ => Err("owned admission requires a real direct Wound, member Edit or debit-producing Spread command".into()),
    }
}
