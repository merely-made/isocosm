// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::spread::{Identity, Substrate, TransitId};

pub(crate) fn substrate_source(at:Substrate) -> StockSource {
    match at { Substrate::Site(id) => StockSource::Site(id), Substrate::Host(id) => StockSource::Entity(id),
        Substrate::Place(id) => StockSource::Place(id) }
}

pub(crate) fn transit_amount(state:&State, id:TransitId, account:&str) -> Result<u64> {
    let mut matches = state.spreads.transit_marks.iter().filter(|mark| mark.id == id);
    let mark = matches.next().ok_or("native transit ID is absent")?;
    if matches.next().is_some() { return Err("native transit ID is duplicated".into()); }
    let transfer = state.spreads.transit.get(mark.index).ok_or("native transit mark points outside its physical queue")?;
    if transfer.identity.key() != account { return Err("transit claim names a different registered stock identity".into()); }
    Ok(transfer.amount)
}

impl Context {
    pub fn operation_rule(&self) -> Result<OperationRule> {
        match &self.origin {
            DebitOrigin::Process { process, .. } => Ok(OperationRule::Process { process:process.clone(),ordinal:self.operation }),
            DebitOrigin::Command(command) => Ok(OperationRule::Command(types::direct_operation(command)?)),
            DebitOrigin::NativeSpread { operation,identity,at, .. } =>
                Ok(OperationRule::Spread { operation:*operation,identity:identity.clone(),at:*at }),
            DebitOrigin::Clock { .. } => Err("clock debit lacks its actual native operation".into()),
        }
    }
    pub fn emit_spread(&mut self, output:SpreadOutput, source:StockSource, identity:Identity, amount:u64) -> Result<()> {
        let witness = SpreadOutputWitness { action:self.action.clone(),operation:self.operation,
            output,source,identity:identity.clone(),amount };
        if amount == 0 || self.outputs.insert(output,witness.clone()).is_some() {
            return Err("native spread operation produced an ambiguous or empty output".into());
        }
        self.matter.insert(identity.key());
        self.identities.insert(identity.key(),identity);
        self.output_report.push(witness);
        Ok(())
    }
}
