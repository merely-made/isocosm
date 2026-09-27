// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The dev source (rulings 271, 344 and 358): matter a dev places at a site,
//! into any matter account the rules declare, enters from outside the
//! conserved total. The world then holds that matter and hashes as it does;
//! what has been issued is kept beside the world, never in it, and
//! recomputed on replay from the history, whose placing command is the
//! record and labels the run assisted.

use crate::{Result, meaning::credit, rules::AccountKind, schema::*, simulation::Simulation};

impl Simulation {
    /// Places `amount` of the matter account `account` at `site`.
    pub fn place(&mut self, site: Id, account: &str, amount: u64) -> Result<()> {
        if !matches!(
            self.genesis.rules.accounts.get(account),
            Some(AccountKind::Matter { .. })
        ) {
            return Err(format!("{account} is not a matter account"));
        }
        if amount == 0 {
            return Err("nothing to place".into());
        }
        let wide = u128::from(amount);
        let conserved = self.conserved.checked_add(wide).ok_or("matter overflow")?;
        let issued = self.issued.checked_add(wide).ok_or("matter overflow")?;
        let s = self.state.sites.get_mut(&site).ok_or("unknown site")?;
        credit(&mut s.accounts, account, amount)?;
        (self.conserved, self.issued) = (conserved, issued);
        Ok(())
    }

    /// The matter the dev source has issued into this run.
    pub fn issued(&self) -> u128 {
        self.issued
    }
}
