// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent C11 territory direction (498; 881): a territory chooses the
//! routes its overflow takes. Un-nudged choice reads priority alone, ties to
//! the first process; nudges, owned stock and titles are refused (881, 882).

use super::*;
use crate::rules::{Binding, Causation, Effect, Process, Query};
use crate::schema::Method;

/// A scheduled choice that directs a territory's overflow and does nothing
/// else, with requirements read from the actor alone.
pub(crate) fn directing(p: &Process) -> Option<(Id, &BTreeSet<Id>)> {
    if p.period.is_none() || p.causation != Causation::Choice || !p.commitments.is_empty()
        || p.risk.is_some() || p.target.is_some() || !p.goal_directives.is_empty()
        || p.need_account.is_some() || p.port_feeding.is_some() { return None; }
    let [Effect::DirectOverflow { from, destinations }] = p.effects.as_slice() else { return None };
    let actor = |q: &Query| matches!(q, Query::Alive(Binding::Actor)
        | Query::Trait { who: Binding::Actor, .. });
    p.requires.iter().all(actor).then_some((*from, destinations))
}

fn admits(p: &Process, body: &Entity) -> bool {
    body.alive && body.method != Method::Inert && p.requires.iter().all(|q| match q {
        Query::Alive(_) => body.alive,
        Query::Trait { key, .. } => body.traits.contains(key),
        _ => false,
    })
}

impl Model {
    /// The first tick after `done`, up to `until`, at which a direction is due.
    pub(super) fn next_direction(&self, rules: &Rules, done: Tick, until: Tick) -> Option<Tick> {
        rules.processes.values().filter(|p| directing(p).is_some())
            .filter_map(|p| p.period.and_then(|n| done.checked_add(n - done % n)))
            .filter(|t| *t <= until).min()
    }

    /// Each selected destination by exactly one transmitting authored route.
    fn ready(&self, rules: &Rules, actor: Id, from: Id, destinations: &BTreeSet<Id>)
        -> Result<Option<Vec<(Id, u64, Tick)>>> {
        let Some(site) = self.sites.get(&from) else { return Ok(None) };
        if destinations.is_empty() { return Ok(None); }
        let mut routes = Vec::new();
        for to in destinations {
            let mut matching = site.routes.iter().filter(|r| r.to == *to);
            let (Some(route), None) = (matching.next(), matching.next()) else { return Ok(None) };
            if route.transmission == 0 || route.transmission > 1_000_000 || route.travel == 0
                || *to == from || !self.sites.contains_key(to) { return Ok(None); }
            routes.push((*to, u64::from(route.transmission), route.travel));
        }
        let identity = Identity::Territory(actor);
        let at = Substrate::Site(from);
        let excess = self.held(at, &identity)? > self.capacity(rules, Some(at), &identity)?;
        Ok(excess.then_some(routes))
    }

    /// Equal shares of the local excess over the selected routes, each
    /// transmitted in whole milligrams; remainders stay at the source.
    fn direct(&mut self, rules: &Rules, actor: Id, from: Id, routes: &[(Id, u64, Tick)]) -> Result<u64> {
        let identity = Identity::Territory(actor);
        let at = Substrate::Site(from);
        let excess = self.held(at, &identity)?.saturating_sub(self.capacity(rules, Some(at), &identity)?);
        let offered = excess / routes.len() as u64;
        let mut sent = 0u64;
        let mut queue = Vec::new();
        for &(to, transmission, travel) in routes {
            let amount = (u128::from(offered) * u128::from(transmission) / 1_000_000) as u64;
            let arrives = self.tick.checked_add(travel).ok_or("spread arrival time overflows")?;
            sent = sent.checked_add(amount).ok_or("directed overflow sum overflows")?;
            if amount != 0 {
                queue.push(Batch { transfer: Transfer { identity: identity.clone(), from, to,
                    departed: self.tick, arrives, amount }, count: 1 });
            }
        }
        if sent == 0 { return Ok(0); }
        crate::meaning::debit(self.ledger_mut(at)?, &identity.key(), sent)?;
        self.details.transit.extend(queue);
        Ok(sent)
    }

    /// An un-nudged deliberative territory's choice among the due options:
    /// the feasible one of highest priority, the first on a tie.
    fn choose(&self, rules: &Rules, tick: Tick, actor: Id, body: &Entity) -> Result<Option<Key>> {
        let mut best = None;
        let mut top = i64::MIN;
        for p in rules.processes.values().filter(|p| p.causation == Causation::Choice) {
            if !p.period.is_some_and(|n| tick % n == 0) { continue; }
            let Some((from, destinations)) = directing(p) else {
                return Err("the independent chooser weighs only territory directions".into());
            };
            if !admits(p, body) || self.ready(rules, actor, from, destinations)?.is_none() { continue; }
            if i64::from(p.priority) > top {
                top = i64::from(p.priority);
                best = Some(p.id.clone());
            }
        }
        Ok(best)
    }

    /// The directions due at `tick`, pass by pass in priority and identity
    /// order. A chosen process lasts the tick; an empty choice one pass (813).
    pub(super) fn directions(&mut self, rules: &Rules, tick: Tick) -> Result<()> {
        let mut due: Vec<&Process> = rules.processes.values()
            .filter(|p| directing(p).is_some() && p.period.is_some_and(|n| tick % n == 0)).collect();
        due.sort_by(|a, b| (a.priority, &a.id).cmp(&(b.priority, &b.id)));
        let mut chosen: BTreeMap<Id, Option<Key>> = BTreeMap::new();
        for p in due {
            let (from, destinations) = directing(p).expect("filtered above");
            chosen.retain(|_, c| c.is_some());
            if self.bins.keys().any(|body| admits(p, body)) {
                return Err("an unaddressed body would direct overflow".into());
            }
            let actors: Vec<(Id, Entity)> = self.details.named.iter()
                .filter(|(_, body)| admits(p, body)).map(|(id, b)| (*id, b.clone())).collect();
            for (actor, body) in actors {
                match body.method {
                    Method::Deliberative => {
                        if !chosen.contains_key(&actor) {
                            chosen.insert(actor, self.choose(rules, tick, actor, &body)?);
                        }
                        if chosen[&actor].as_deref() != Some(p.id.as_str()) { continue; }
                    },
                    Method::Reactive => {},
                    _ => return Err("owned or normative direction is outside the crowd (882)".into()),
                }
                let Some(routes) = self.ready(rules, actor, from, destinations)? else { continue };
                let before = self.clone();
                self.direct(rules, actor, from, &routes)?;
                let handled = (Substrate::Site(from), Identity::Territory(actor));
                self.changed_except(&before, rules, Some(&handled))?;
            }
        }
        Ok(())
    }
}
