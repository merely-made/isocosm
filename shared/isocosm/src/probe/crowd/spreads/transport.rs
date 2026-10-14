// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent integer offers and counted transfer clock (498, 828, 831).

use super::*;

impl Model {
    fn origin(&self, at:Substrate) -> Result<Option<Id>> {
        match at {
            Substrate::Site(id) if self.sites.contains_key(&id) => Ok(Some(id)),
            Substrate::Host(id) => Ok(self.details.named.get(&id)
                .ok_or("overflow host was not addressed at founding")?.resident_site()),
            _ => Err("overflow has no site-grain source".into()),
        }
    }

    /// Per-member rounding precedes multiplication by a histogram count.
    fn offers(&self, identity:&Identity, from:Id, current:u64, limit:u64, count:u64)
        -> Result<(u64,Vec<Batch>)> {
        let site = self.sites.get(&from).ok_or("overflow site is absent")?;
        let routes:Vec<_> = site.routes.iter().filter(|route| route.transmission != 0).collect();
        if routes.is_empty() { return Ok((0,Vec::new())); }
        let offer = current.saturating_sub(limit) / routes.len() as u64;
        let mut sent = 0u64;
        let mut queue = Vec::new();
        for route in routes {
            if route.travel == 0 || route.transmission > 1_000_000 || route.to == from
                || !self.sites.contains_key(&route.to) { return Err("invalid authored overflow route".into()); }
            // C11 planted-control anchor: transmission follows equal offers.
            let amount = (u128::from(offer) * u128::from(route.transmission) / 1_000_000) as u64;
            let arrives = self.tick.checked_add(route.travel).ok_or("spread arrival time overflows")?;
            sent = sent.checked_add(amount).ok_or("sent amount overflows")?;
            if amount != 0 {
                queue.push(Batch { transfer:Transfer { identity:identity.clone(),from,to:route.to,
                    departed:self.tick,arrives,amount },count });
            }
        }
        Ok((sent,queue))
    }

    pub(super) fn overflow(&mut self, rules:&Rules, identity:&Identity, at:Substrate) -> Result<u64> {
        let current = self.held(at,identity)?;
        let limit = self.capacity(rules,Some(at),identity)?;
        let Some(from) = self.origin(at)? else { return Ok(0); };
        let (sent,queue) = self.offers(identity,from,current,limit,1)?;
        crate::meaning::debit(self.ledger_mut(at)?,&identity.key(),sent)?;
        self.details.transit.extend(queue);
        Ok(sent)
    }

    pub(super) fn changed(&mut self, before:&Self, rules:&Rules) -> Result<()> {
        if rules.spread_capacity.is_none() { return Ok(()); }
        let identities:Vec<_> = self.details.identities.iter().cloned().collect();
        let mut sources = Vec::new();
        for (&id,site) in &self.sites {
            for identity in &identities {
                if site.accounts.get(&identity.key()).is_some_and(|n| *n != 0) {
                    sources.push((Substrate::Site(id),identity.clone()));
                }
            }
        }
        for (&id,body) in &self.details.named {
            for identity in &identities {
                if matches!(identity,Identity::Surface(_))
                    && body.accounts.get(&identity.key()).is_some_and(|n| *n != 0) {
                    sources.push((Substrate::Host(id),identity.clone()));
                }
            }
        }
        sources.sort();
        for (at,identity) in sources {
            let current = self.held(at,&identity)?;
            let limit = self.capacity(rules,Some(at),&identity)?;
            if current <= limit { continue; }
            let previous = before.held(at,&identity).unwrap_or(0);
            let lower = before.held(at,&identity).is_ok()
                && before.capacity(rules,Some(at),&identity)? > limit;
            let arrived = before.origin(at).ok().flatten().is_none() && self.origin(at)?.is_some();
            if current > previous || lower || arrived { self.overflow(rules,&identity,at)?; }
        }
        // Each host's own capacity and integer remainder, not capacity for
        // the total bin. No source stock is copied into a parallel ledger.
        let bins:Vec<_> = self.bins.iter().map(|(e,n)| (e.clone(),*n)).collect();
        for (body,n) in bins {
            let mut next = body.clone();
            let Some(from) = body.resident_site() else { continue; };
            for identity in identities.iter().filter(|id| matches!(id,Identity::Surface(_))) {
                let current = next.accounts.get(&identity.key()).copied().unwrap_or(0);
                if current == 0 { continue; }
                let limit = self.capacity(rules,None,identity)?;
                let previous = if before.bins.contains_key(&body) { current } else { 0 };
                let lower = before.details.identities.contains(identity)
                    && before.capacity(rules,None,identity)? > limit;
                if current > limit && (current > previous || lower) {
                    let (sent,queue) = self.offers(identity,from,current,limit,n)?;
                    crate::meaning::debit(&mut next.accounts,&identity.key(),sent)?;
                    self.details.transit.extend(queue);
                }
            }
            if next != body {
                self.bins.remove(&body);
                let count = self.bins.entry(normalize(next)).or_default();
                *count = count.checked_add(n).ok_or("overflow host bin count overflows")?;
            }
        }
        Ok(())
    }

    fn settle(&mut self, tick:Tick) -> Result<()> {
        let mut pending = Vec::new();
        for batch in &self.details.transit {
            let transfer = &batch.transfer;
            // C11 planted-control anchor: a departure is not an arrival.
            if transfer.arrives > tick { pending.push(batch.clone()); continue; }
            let amount = u128::from(transfer.amount) * u128::from(batch.count);
            let amount = u64::try_from(amount).map_err(|_| "destination spread credit overflows")?;
            let site = self.sites.get_mut(&transfer.to).ok_or("spread destination is absent")?;
            crate::meaning::credit(&mut site.accounts,&transfer.identity.key(),amount)?;
        }
        self.details.transit = pending;
        Ok(())
    }

    pub(super) fn advance(&mut self, ticks:Tick, rules:&Rules) -> Result<()> {
        let until = self.tick.checked_add(ticks).ok_or("independent spread clock overflows")?;
        loop {
            let bodies = self.bins.keys().chain(self.details.named.values())
                .filter_map(|body| body.journey.map(|j| j.arrives));
            let spreads = self.details.transit.iter().map(|batch| batch.transfer.arrives);
            let Some(tick) = bodies.chain(spreads).min().filter(|tick| *tick <= until) else { break; };
            if tick < self.tick { return Err("independent arrival precedes the clock".into()); }
            let before = self.clone();
            self.tick = tick;
            // Actual bodies land first at a tie, then conserved spread stock.
            self.land_bodies(tick)?;
            self.changed(&before,rules)?;
            let before_credit = self.clone();
            self.settle(tick)?;
            self.changed(&before_credit,rules)?;
            self.validate(rules)?;
        }
        self.tick = until;
        self.validate(rules)
    }
}
