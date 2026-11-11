// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The edible matter a feed could draw from at an eater's site, set against
//! what one member asks of it: the companion reading to `available_food`'s
//! headcount. A meadow of grazed-down stems still has many holders and
//! little to eat; this sums what they hold.

use super::*;
use crate::process::IntakePort;

impl Process {
    /// The matter one member asks of this feed at each feeding: the first
    /// `Eat`'s amount, times how often the eater's lineage decides where a
    /// choice takes it (`decides`), as `asked` resolves a meal. A feed run
    /// by anything but a choice is not paced.
    ///
    /// # Errors
    /// Fails where the process eats nothing or its amount is unresolved.
    pub fn meal_demand(&self, decides: Option<u64>) -> crate::Result<u64> {
        let amount = self
            .effects
            .iter()
            .find_map(|e| match e {
                Effect::Eat { amount, .. } => Some(amount),
                _ => None,
            })
            .ok_or("the feed has no ordinary Eat")?;
        let pace = if self.causation == Causation::Choice { decides.unwrap_or(1) } else { 1 };
        amount.resolved()?.checked_mul(pace).ok_or_else(|| "a meal overflows".into())
    }

    /// What one prey member offers `actor`'s feed: under a feeding contract
    /// the admitted matter of its exposed parts, else (no contract, the flat
    /// A1 world) what an ordinary `Eat` can take of the prey's matter
    /// accounts, the selector's life and lineages applied.
    fn offer_weight(&self, actor: &Entity, prey: &Entity, first_books: impl FnOnce() -> Option<Ledger>, sim: &Simulation) -> u128 {
        let rules = &sim.genesis.rules;
        if self.port_feeding.is_some() {
            return self.food_weight(actor, prey, rules, &sim.state.lineages);
        }
        let Some(target) = &self.target else { return 0 };
        if crate::directing::is_participant(prey)
            || prey.journey.is_some()
            || target.alive.is_some_and(|alive| alive != prey.alive)
            || target.lineage.as_ref().is_some_and(|l| *l != prey.lineage)
            || (!target.among.is_empty() && !target.among.contains(&prey.lineage))
        {
            return 0;
        }
        let of = match self.effects.first() {
            Some(Effect::Eat { of, .. }) => of.as_slice(),
            _ => return 0,
        };
        let Some(books) = first_books() else { return 0 };
        mass(&crate::schedule::edible(&books, of), rules)
    }
}

impl Simulation {
    /// The matter one member of `lineage` asks of `feed` at each feeding.
    ///
    /// # Errors
    /// Fails where the feed is missing or eats nothing.
    pub fn feed_demand(&self, feed: &str, lineage: &Key) -> crate::Result<u64> {
        let p = self.genesis.rules.processes.get(feed).ok_or("food contract is missing")?;
        p.meal_demand(self.state.lineages.get(lineage).and_then(|l| l.decides))
    }

    /// The matter the eater's feed could draw from at its site: the summed
    /// weight of every resident, non-dormant holder the feed's target
    /// selector admits (same place forced), cohort multiplicity counted and
    /// the actor's own member left out, as `available_food` counts heads. A
    /// feed with a feeding contract weighs a holder by the matter its intake
    /// admits from exposed parts (`food_weight`); one with none, the flat
    /// A1 feed, by what its plain `Eat` can take of the prey's matter
    /// accounts. Independent of appetite, clocks and reach.
    ///
    /// An upper bound on what bites can take: it ignores per-bite caps,
    /// defence thresholds, the feed's other requires and the competitors'
    /// shares of a pass.
    ///
    /// # Errors
    /// Fails where the feed or its selector is missing, or the actor lacks
    /// the feed's inherited identity.
    pub fn edible_matter(&self, actor: Id, feed: &str) -> crate::Result<u64> {
        let body = self.body_at_start(actor).ok_or("food actor is missing")?;
        let Some(site) = body.resident_site().filter(|_| body.alive) else { return Ok(0) };
        let p = self.genesis.rules.processes.get(feed).ok_or("food contract is missing")?;
        if p.requires.iter().any(|q| matches!(q, Query::Trait { who: Binding::Actor, key } if !body.traits.contains(key))) {
            return Err("food actor does not carry the feed's inherited identity".into());
        }
        let mut selector = p.target.clone().ok_or("food contract has no target selector")?;
        selector.same_place = true;
        let weigh = |first: Id, prey: &Entity| p.offer_weight(body, prey, || self.books_at_start(first).map(|b| (*b).clone()), self);
        // What the weight reads of the actor is its intake organ's port.
        let port: Option<IntakePort> = p.port_feeding.as_ref().and_then(|f| {
            let (id, _) = f.part(body).ok()?;
            crate::mosaic::ports::active(f.intake(body, id).ok()?)
        });
        let whole = || {
            let mut total = 0u128;
            for first in self.searched(&selector, site) {
                let Some((first, count)) = self.target_span(first) else { continue };
                let Some(prey) = self.body_at_start(first).filter(|e| e.resident_site() == Some(site) && !self.genesis.rules.dormant(e)) else { continue };
                total = total.saturating_add(weigh(first, prey).saturating_mul(u128::from(count)));
            }
            total
        };
        let total = match &self.frame {
            Some(frame) => frame.edible((site, feed.to_owned(), port), whole),
            None => whole(),
        };
        let own = self.target_span(actor)
            .filter(|_| !self.genesis.rules.dormant(body))
            .map_or(0, |(first, _)| weigh(first, body));
        Ok(u64::try_from(total.saturating_sub(own)).unwrap_or(u64::MAX))
    }

    /// Whether `EdiblePerHead` holds for `actor` at `place`, with its reading.
    pub(crate) fn edible_per_head(&self, actor: Id, place: Id, feed: &str, at_least: u64, per_members: u64) -> Result<String, String> {
        let body = self.body_at_start(actor).filter(|e| e.resident_site() == Some(place));
        let matter = self.edible_matter(actor, feed)?;
        let heads = body.map_or(0, |e| self.members_at(place, &BTreeSet::from([e.lineage.clone()])));
        let yes = heads > 0 && per_members > 0
            && u128::from(matter) * u128::from(per_members) >= u128::from(heads) * u128::from(at_least);
        let reading = format!("EdiblePerHead {{ feed: {feed:?}, at_least: {at_least}, per_members: {per_members} }} = {matter}/{heads}");
        if yes { Ok(reading) } else { Err(format!("missing requirement: {reading}")) }
    }
}
