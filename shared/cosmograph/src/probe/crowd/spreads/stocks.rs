// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::AccountKind;
use crate::spread::{CapacitySource, Stacking, Supply, Placement};

impl Model {
    fn ledger(&self, at:Substrate) -> Result<&Ledger> {
        match at {
            Substrate::Site(id) => self.sites.get(&id).map(|s| &s.accounts).ok_or("unknown substrate site".into()),
            Substrate::Host(id) => self.details.named.get(&id).map(|e| &e.accounts)
                .ok_or("host address was not declared at independent founding".into()),
            Substrate::Place(_) => Err("independent spread crowd is site grain".into()),
        }
    }

    pub(super) fn ledger_mut(&mut self, at:Substrate) -> Result<&mut Ledger> {
        match at {
            Substrate::Site(id) => self.sites.get_mut(&id).map(|s| &mut s.accounts).ok_or("unknown substrate site".into()),
            Substrate::Host(id) => self.details.named.get_mut(&id).map(|e| &mut e.accounts)
                .ok_or("host address was not declared at independent founding".into()),
            Substrate::Place(_) => Err("independent spread crowd is site grain".into()),
        }
    }

    pub(crate) fn held(&self, at:Substrate, identity:&Identity) -> Result<u64> {
        if !self.details.identities.contains(identity) { return Err("unregistered spread identity".into()); }
        if matches!((at,identity),(Substrate::Host(_),Identity::Territory(_))) {
            return Err("territory stocks belong to places".into());
        }
        Ok(self.ledger(at)?.get(&identity.key()).copied().unwrap_or(0))
    }

    pub(crate) fn lineage(&self, identity:&Identity) -> Result<Key> {
        if !self.details.identities.contains(identity) { return Err("unregistered spread identity".into()); }
        match identity {
            Identity::Territory(id) => self.details.named.get(id).map(|e| e.lineage.clone())
                .ok_or("territory has no actual founding identity".into()),
            Identity::Surface(key) if self.lineages.contains_key(key) => Ok(key.clone()),
            _ => Err("spread lineage is absent".into()),
        }
    }

    pub(crate) fn capacity(&self, rules:&Rules, at:Option<Substrate>, identity:&Identity) -> Result<u64> {
        if let Some(at) = at { self.held(at,identity)?; }
        let traits = &self.lineages.get(&self.lineage(identity)?).ok_or("capacity lineage is absent")?.traits;
        rules.spread_capacity.as_ref().ok_or("world has no declared spread capacity mechanic")?
            .resolve(traits,at,identity,&self.details.modifiers)
    }

    fn resident(&self, at:Substrate) -> Result<Id> {
        self.ledger(at)?;
        match at {
            Substrate::Site(id) => Ok(id),
            Substrate::Host(id) => self.details.named[&id].resident_site()
                .ok_or("local spread placement has no resident host".into()),
            Substrate::Place(_) => Err("independent spread crowd is site grain".into()),
        }
    }

    pub(super) fn place(&mut self, rules:&Rules, placement:&Placement) -> Result<u64> {
        let from = placement.source.from();
        if placement.amount == 0 || self.resident(from)? != self.resident(placement.at)? {
            return Err("spread placement needs positive stock within one actual resident site".into());
        }
        self.held(placement.at,&placement.identity)?;
        let lineage = self.lineage(&placement.identity)?;
        match &placement.source {
            Supply::Matter { from, account } => {
                if let Substrate::Host(id) = from && self.details.named[id].parts.values()
                    .any(|p| p.matter.get(account).is_some_and(|n| *n > 0)) {
                    return Err("anatomical spread supply needs an admitted allocation path".into());
                }
                if !matches!(rules.accounts.get(account),Some(AccountKind::Matter {lineage:source,..}) if source == &lineage) {
                    return Err("spread supply does not declare the exact target lineage".into());
                }
            },
            Supply::Spread { identity, .. } => {
                if identity != &placement.identity { return Err("spread placement changes identity".into()); }
                self.held(from,identity)?;
            },
        }
        crate::meaning::debit(self.ledger_mut(from)?,&placement.source.account(),placement.amount)?;
        crate::meaning::credit(self.ledger_mut(placement.at)?,&placement.identity.key(),placement.amount)?;
        Ok(placement.amount)
    }

    pub(crate) fn validate(&self, rules:&Rules) -> Result<()> {
        let mut keys = BTreeSet::new();
        for identity in &self.details.identities {
            crate::validation::key(&identity.key())?;
            if rules.accounts.contains_key(&identity.key()) || !keys.insert(identity.key()) {
                return Err("spread identity aliases a stock".into());
            }
            self.lineage(identity)?;
        }
        let bodies = self.bins.keys().chain(self.details.named.values());
        for body in bodies {
            for (key,n) in &body.accounts {
                if *n != 0 && self.details.identities.iter().any(|i|
                    matches!(i,Identity::Territory(_)) && i.key() == *key) {
                    return Err("territory stock is duplicated in a body".into());
                }
                if !rules.accounts.contains_key(key) && !keys.contains(key) { return Err("unknown host stock".into()); }
            }
            if body.parts.values().any(|p| p.matter.keys().any(|key| keys.contains(key))) {
                return Err("spread stock cannot be part tissue".into());
            }
        }
        if self.sites.values().any(|s| s.accounts.keys().any(|k| !rules.accounts.contains_key(k) && !keys.contains(k))) {
            return Err("unknown site stock".into());
        }
        self.validate_modifiers(rules)?;
        for batch in &self.details.transit {
            let t = &batch.transfer;
            if batch.count == 0 || t.amount == 0 || !self.details.identities.contains(&t.identity)
                || t.from == t.to || t.departed > self.tick || t.arrives <= self.tick
                || t.arrives <= t.departed || !self.sites.contains_key(&t.to)
                || !self.sites.get(&t.from).is_some_and(|s| s.routes.iter().any(|r|
                    r.to == t.to && r.travel == t.arrives - t.departed)) {
                return Err("invalid independent spread transit".into());
            }
        }
        Ok(())
    }

    fn validate_modifiers(&self, rules:&Rules) -> Result<()> {
        let Some(policy) = &rules.spread_capacity else {
            return if self.details.modifiers.is_empty() { Ok(()) } else { Err("modifiers lack declared mechanics".into()) };
        };
        let mut keys = BTreeSet::new();
        for m in &self.details.modifiers {
            crate::validation::key(&m.key)?;
            if !keys.insert(&m.key) || !policy.layers.iter().any(|l| l.key == m.layer)
                || m.identity.as_ref().is_some_and(|id| !self.details.identities.contains(id)) {
                return Err("invalid modifier key, layer or identity".into());
            }
            i64::try_from(m.milligrams).map_err(|_| "capacity modifier exceeds signed expression range")?;
            if let CapacitySource::Substrate(at) = m.source {
                self.ledger(at)?;
                if matches!((&m.identity,at),(Some(Identity::Territory(_)),Substrate::Host(_))) {
                    return Err("territory capacity belongs to a place".into());
                }
            }
        }
        if self.details.modifiers.windows(2).any(|p| p[0].key >= p[1].key) {
            return Err("capacity modifiers are not in canonical order".into());
        }
        for layer in policy.layers.iter().filter(|l| l.stacking == Stacking::Replace) {
            let matching:Vec<_> = self.details.modifiers.iter().filter(|m| m.layer == layer.key).collect();
            for (i,a) in matching.iter().enumerate() {
                for b in &matching[i+1..] {
                    let identity = a.identity.is_none() || b.identity.is_none() || a.identity == b.identity;
                    let source = a.source == CapacitySource::World || b.source == CapacitySource::World || a.source == b.source;
                    let host = matches!(a.source,CapacitySource::Substrate(Substrate::Host(_)))
                        || matches!(b.source,CapacitySource::Substrate(Substrate::Host(_)));
                    let territory = matches!(a.identity.as_ref().or(b.identity.as_ref()),Some(Identity::Territory(_)));
                    if identity && source && !(host && territory) { return Err("overlapping replace capacity values".into()); }
                }
            }
        }
        Ok(())
    }
}
