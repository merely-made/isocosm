// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Model {
    pub(super) fn fork(&mut self, rules:&Rules, seed:u64, parent:&str, at:Substrate,
        amount:u64, traits:BTreeSet<Key>) -> Result<Key> {
        if !traits.is_subset(&rules.traits) { return Err("strain selects an unknown trait".into()); }
        let source = Identity::Surface(parent.into());
        if amount == 0 || self.held(at,&source)? < amount { return Err("strain lacks selected local matter".into()); }
        let original = self.lineages.get(parent).ok_or("unknown strain parent")?;
        let key = spread::strain_key(seed,parent,at,&traits);
        if rules.accounts.contains_key(&key) { return Err("strain aliases an ordinary account".into()); }
        let child = Lineage { parent:Some(parent.into()), traits,
            revision:original.revision.checked_add(1).ok_or("strain revision overflows")?, ..original.clone() };
        if self.lineages.get(&key).is_some_and(|l| l != &child) { return Err("strain identity has different genes".into()); }
        crate::meaning::debit(self.ledger_mut(at)?,&source.key(),amount)?;
        crate::meaning::credit(self.ledger_mut(at)?,&key,amount)?;
        self.lineages.insert(key.clone(),child);
        self.details.identities.insert(Identity::Surface(key.clone()));
        Ok(key)
    }
}
