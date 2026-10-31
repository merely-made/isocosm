// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::spread::{Identity, Substrate};

impl Simulation {
    /// Current physical stock, independent of pass-frame title snapshots.
    pub fn stock_balance(&self, source:StockSource, account:&str) -> Result<u64> {
        if let Some(identity) = self.state.spreads.identities.iter().find(|identity| identity.key() == account) {
            return self.spread_balance(source,identity);
        }
        if !crate::meaning::matter(&self.genesis.rules,account) {
            return Err("stock balance names no ordinary matter or registered native identity".into());
        }
        let ledger = match source {
            StockSource::Entity(id) => crate::anatomy::books(self.state.population.get(id).ok_or("stock body is absent")?),
            StockSource::Site(id) => self.state.sites.get(&id).ok_or("stock site is absent")?.accounts.clone(),
            StockSource::Place(id) => self.state.spreads.places.iter().find(|place| place.place == id)
                .ok_or("stock finer place is absent")?.accounts.clone(),
            StockSource::Transit(_) => return Err("spread transit cannot hold an ordinary rule account".into()),
        };
        Ok(ledger.get(account).copied().unwrap_or(0))
    }
    pub fn spread_balance(&self, source:StockSource, identity:&Identity) -> Result<u64> {
        if !self.state.spreads.identities.contains(identity) {
            return Err("spread balance needs an exact registered native identity".into());
        }
        let at = match source {
            StockSource::Entity(id) => Substrate::Host(id),
            StockSource::Site(id) => Substrate::Site(id),
            StockSource::Place(id) => Substrate::Place(id),
            StockSource::Transit(id) => return debit::transit_amount(&self.state,id,&identity.key()),
        };
        crate::spread::held(&self.state,at,identity)
    }
}
