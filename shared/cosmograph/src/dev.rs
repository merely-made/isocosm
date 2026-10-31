// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The dev source (rulings 271, 344 and 358): matter a dev places at a site,
//! into any matter account the rules declare, enters from outside the
//! conserved total. The world then holds that matter and hashes as it does;
//! what has been issued is kept beside the world, never in it, and
//! recomputed on replay from the history, whose placing command is the
//! record and labels the run assisted. A dev may also force a birth, its
//! needs made up from the same source, or kill a member (ruling 782).

use crate::{
    Result,
    flows::{Holder, Leg, MadeBy},
    meaning::credit,
    rules::{AccountKind, Binding, Effect, Query},
    schema::*,
    simulation::{Receipt, Simulation},
};

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
        let leg = Leg {
            from: (Holder::Dev, account.into()),
            to: (Holder::Site(site), account.into()),
            amount,
        };
        self.flowed(MadeBy::Command("PlaceMatter".into()), vec![leg], 1);
        Ok(())
    }

    /// Issues `amount` of matter account `account` to `entity`'s ledger.
    fn issue(&mut self, entity: Id, account: &str, amount: u64) -> Result<()> {
        if !matches!(
            self.genesis.rules.accounts.get(account),
            Some(AccountKind::Matter { .. })
        ) {
            return Err(format!("{account} is not a matter account"));
        }
        let wide = u128::from(amount);
        let conserved = self.conserved.checked_add(wide).ok_or("matter overflow")?;
        let issued = self.issued.checked_add(wide).ok_or("matter overflow")?;
        credit(&mut self.state.population.lift(entity)?.accounts, account, amount)?;
        (self.conserved, self.issued) = (conserved, issued);
        let leg = Leg {
            from: (Holder::Dev, account.into()),
            to: (Holder::Entity(entity), account.into()),
            amount,
        };
        self.flowed(MadeBy::Command("ForceBirth".into()), vec![leg], 1);
        Ok(())
    }

    /// Forces `parent`'s birth: its lineage's birth process, the accounts
    /// it requires made up by the dev source.
    pub fn force_birth(&mut self, parent: Id) -> Result<Receipt> {
        let pop = &self.state.population;
        let e = pop.get(parent).filter(|e| e.alive).cloned().ok_or("no living parent")?;
        let ours = |q: &Query| match q {
            Query::Trait {
                who: Binding::Actor,
                key,
            } => e.traits.contains(key),
            _ => true,
        };
        let births = |d: &crate::rules::Process| {
            d.effects.iter().any(|f| matches!(f, Effect::Birth { .. })) && d.requires.iter().all(ours)
        };
        let (key, def) = self
            .genesis
            .rules
            .processes
            .iter()
            .find(|(_, d)| births(d))
            .map(|(k, d)| (k.clone(), d.clone()))
            .ok_or("its lineage has no birth process")?;
        for q in &def.requires {
            if let Query::Account {
                who: Binding::Actor,
                key,
                at_least,
            } = q
            {
                let held = e.accounts.get(key).copied().unwrap_or(0);
                if *at_least > held {
                    self.issue(parent, key, at_least - held)?;
                }
            }
        }
        Ok(self.execute(parent, None, &key, Some("dev:force-birth".into())))
    }

    /// Kills `entity` where it stands, its body left as it was.
    pub fn kill(&mut self, entity: Id) -> Result<()> {
        let e = self.state.population.lift(entity)?;
        if !e.alive {
            return Err("not alive".into());
        }
        e.alive = false;
        Ok(())
    }

    /// The matter the dev source has issued into this run.
    pub fn issued(&self) -> u128 {
        self.issued
    }
}
