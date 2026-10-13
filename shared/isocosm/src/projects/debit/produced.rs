// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{history::Command, rules::Rules, spread::{Identity, Substrate}};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProducedOrigin {
    Process { process: Key, actor: Id, target: Option<Id> },
    Command(Box<Command>),
    Clock { tick: Tick },
}

// Ordering is configuration equality bookkeeping, never output selection.
impl Ord for ProducedOrigin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let rank = |origin: &Self| match origin { Self::Process { .. } => 0, Self::Command(_) => 1, Self::Clock { .. } => 2 };
        rank(self).cmp(&rank(other)).then_with(|| match (self, other) {
            (Self::Process { process:a,actor:b,target:c }, Self::Process { process:x,actor:y,target:z }) => (a,b,c).cmp(&(x,y,z)),
            (Self::Clock { tick:a }, Self::Clock { tick:b }) => a.cmp(b),
            (Self::Command(a), Self::Command(b)) => serde_json::to_vec(a).expect("native command serialization")
                .cmp(&serde_json::to_vec(b).expect("native command serialization")),
            _ => std::cmp::Ordering::Equal,
        })
    }
}
impl PartialOrd for ProducedOrigin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedSpread { pub operation: SpreadDebit, pub identity: Identity, pub at: Substrate }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedOperation {
    pub origin: ProducedOrigin,
    pub ordinal: u64,
    pub spread: Option<ProducedSpread>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProducedOutput { Claim(u64), Spread(SpreadOutput) }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputReference {
    pub producer: ProducedOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Key>,
    pub output: ProducedOutput,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedQuantity { pub reference: OutputReference, pub amount: u64 }

/// Immutable creation facts, separate from the current title and physical stock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedClaim {
    pub producer: ProducedOperation,
    pub action: Key,
    pub output: u64,
    pub input: Key,
    pub source: StockSource,
    pub account: Key,
    pub amount: u64,
    pub identity: Option<Identity>,
    pub spread_output: Option<SpreadOutputWitness>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedOutput {
    pub lot: LotQuantity,
    pub source: StockSource,
    pub account: Key,
    pub identity: Option<Identity>,
    pub remaining: u64,
    pub balance: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimInput { Lot(Key), Produced(OutputReference) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimQuantity { pub input: ClaimInput, pub amount: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencedOutputClaim {
    pub input: ClaimInput, pub amount: u64, pub destination: Destination, pub account: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencedRetainedOutput { pub input: ClaimInput, pub amount: u64, pub output: SpreadOutput }

impl ProducedOrigin {
    pub(crate) fn of(origin: &DebitOrigin) -> Result<Self> {
        match origin {
            DebitOrigin::Process { process,actor,target } => Ok(Self::Process { process:process.clone(),actor:*actor,target:*target }),
            DebitOrigin::Command(command) => { types::direct_operation(command)?; Ok(Self::Command(command.clone())) },
            DebitOrigin::Clock { tick } => Ok(Self::Clock { tick:*tick }),
            DebitOrigin::NativeSpread { parent, .. } => match parent.as_ref() {
                DebitOrigin::NativeSpread { .. } => Err("produced reference does not support nested spread origins".into()),
                other => Self::of(other),
            },
        }
    }
    pub(crate) fn validate(&self, rules: &Rules) -> Result<()> {
        match self {
            Self::Process { process, .. } => {
                crate::validation::key(process)?;
                if !rules.processes.contains_key(process) { return Err("produced origin names no native process".into()); }
            },
            Self::Command(command) => { types::direct_operation(command)?; },
            Self::Clock { .. } => {},
        }
        Ok(())
    }
}

impl OutputReference {
    pub(crate) fn validate(&self, rules: &Rules) -> Result<()> {
        self.producer.origin.validate(rules)?;
        if let Some(action) = &self.action { crate::validation::key(action)?; }
        if let Some(spread) = &self.producer.spread { crate::validation::key(&spread.identity.key())?; }
        if let ProducedOutput::Spread(output) = self.output {
            if !self.producer.spread.as_ref().is_some_and(|spread| matches!((spread.operation,output),
                (SpreadDebit::Overflow,SpreadOutput::Route(_)) | (SpreadDebit::ForkStrain,SpreadOutput::GeneratedStrain)
                    | (SpreadDebit::Place,SpreadOutput::Placement)))
            { return Err("reference names no output of its producing spread operation".into()); }
        }
        Ok(())
    }
    pub(crate) fn resolve<'a>(&self, records: &'a BTreeMap<Key,ProducedClaim>) -> Result<(&'a Key,&'a ProducedClaim)> {
        let mut matching = records.iter().filter(|(_,record)| record.producer == self.producer
            && self.action.as_ref().is_none_or(|action| action == &record.action)
            && match self.output {
                ProducedOutput::Claim(index) => index == record.output,
                ProducedOutput::Spread(output) => record.spread_output.as_ref().is_some_and(|witness| witness.output == output),
            });
        let found = matching.next().ok_or("named native produced output is absent")?;
        if matching.next().is_some() { return Err("named native produced output is ambiguous across accepted claims".into()); }
        Ok(found)
    }
}

impl Simulation {
    /// Quote one named accepted output, without substituting stock or owners.
    pub fn resolve_output(&self, reference: &OutputReference, amount: u64) -> Result<ResolvedOutput> {
        reference.validate(&self.genesis.rules)?;
        if amount == 0 { return Err("produced selection needs an exact positive quantity".into()); }
        let (key, record) = reference.resolve(&self.state.projects.produced)?;
        let held = self.state.projects.ownership.get(key).ok_or("produced claim no longer has a native title")?;
        let source = held.current_source().ok_or("produced claim is spent or unavailable")?;
        if held.lot.key != *key || held.lot.source != record.source || held.lot.account != record.account || held.lot.owners.is_empty()
            || amount > held.lot.amount || held.lot.amount > record.amount
        { return Err("produced claim quantity or native account is stale".into()); }
        for owner in &held.lot.owners { self.owner_exists(*owner)?; }
        if let Some(identity) = &record.identity {
            if identity.key() != record.account || !self.state.spreads.identities.contains(identity) {
                return Err("produced claim identity is no longer registered".into());
            }
        } else if !crate::meaning::matter(&self.genesis.rules,&record.account) {
            return Err("produced claim account is not registered native matter".into());
        }
        let balance = self.stock_balance(source,&record.account)?;
        let claimed = self.state.projects.ownership.values().filter(|lot|
            lot.current_source() == Some(source) && lot.lot.account == record.account)
            .try_fold(0u64, |sum,lot| sum.checked_add(lot.lot.amount).ok_or("produced source claims overflow"))?;
        if claimed > balance { return Err("produced claim lacks its actual physical stock".into()); }
        Ok(ResolvedOutput { lot:LotQuantity { key:key.clone(),amount },source,account:record.account.clone(),
            identity:record.identity.clone(),remaining:held.lot.amount,balance })
    }

    pub(crate) fn produced_at_start(&self) -> BTreeMap<Key,ProducedClaim> {
        let mut records = self.state.projects.produced.clone();
        if let Some(frame) = &self.frame {
            for (key,was) in frame.produced_claims() {
                match was { Some(record) => { records.insert(key.clone(),record.clone()); }, None => { records.remove(key); } }
            }
        }
        records
    }
}

impl Context {
    pub(super) fn producer(&self) -> Result<ProducedOperation> {
        let spread = match &self.origin {
            DebitOrigin::NativeSpread { operation,identity,at, .. } => Some(ProducedSpread {
                operation:*operation,identity:identity.clone(),at:*at }), _ => None,
        };
        Ok(ProducedOperation { origin:ProducedOrigin::of(&self.origin)?,ordinal:self.operation,spread })
    }
    pub(crate) fn produced_changes(&self) -> BTreeMap<Key,ProducedClaim> {
        self.produced.iter().filter(|(key,record)| self.initial_produced.get(*key) != Some(*record))
            .map(|(key,record)| (key.clone(),record.clone())).collect()
    }
}
