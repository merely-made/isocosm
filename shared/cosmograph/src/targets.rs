// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a scheduled act with a target finds it. By default, the first
//! member in identity order the process's selector and requirements accept,
//! one per stored group. A weighted selector, feeding's (ruling 287), draws
//! instead among every member it would accept, seeded and keyed by the act,
//! each weighted by the matter it holds, so abundant prey are eaten more.
//! During an advance the groups are filed by place, lineage and life, so a
//! selector that names any of them looks only where it could match instead
//! of across the whole population (ruling 237), and the requirements that
//! cannot read the prey are asked once per search. Every member is read as the
//! pass under way began, so what earlier acts of the pass did to a member
//! changes neither whether it is accepted nor what it weighs (ruling 454).

use crate::{meaning::mass, population::Population, rules::*, schema::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

mod edible;

/// Where a group is filed: its place (none while travelling), its lineage,
/// and whether it lives.
type Filing = (Option<Id>, Key, bool);

/// Stored groups by their first identity, filed by place, lineage and life.
#[derive(Clone, Debug, Default)]
pub(crate) struct Targets {
    filed: BTreeMap<Filing, BTreeSet<Id>>,
    at: BTreeMap<Id, Filing>,
}

/// Whether a requirement can read the prey: it binds the target, reads a
/// relation to it, or is one whose reading the search does not know. The
/// rest read the actor, its part, its site or the tick.
fn reads_prey(q: &Query) -> bool {
    match q {
        Query::Alive(who)
        | Query::Trait { who, .. }
        | Query::Account { who, .. }
        | Query::Below { who, .. }
        | Query::Part { who, .. }
        | Query::Holds { who, .. }
        | Query::Wounded { who, .. }
        | Query::Routes { who, .. } => *who == Binding::Target,
        Query::Age { .. } | Query::Condition { .. } | Query::Mood { .. } | Query::MoodBelow { .. }
        | Query::Expresses { .. } | Query::Members { .. } | Query::MembersBelow { .. } | Query::MembersPer { .. }
        | Query::MembersPerHead { .. }
        | Query::FoodPerHead { .. }
        | Query::EdiblePerHead { .. }
        | Query::EmptyBody
        | Query::Sensed { .. } => false,
        Query::Computed(x) => x.reads().iter().any(|u| u.body() == Binding::Target),
        Query::Related { .. }
        | Query::Project { .. }
        | Query::ProjectDecision
        | Query::InstitutionDecision
        | Query::Institution { .. } => true,
    }
}

impl Targets {
    /// Files a lazily preserved body by its place, lineage and life at pass
    /// start.
    pub(crate) fn keep(&mut self, first: Id, entity: &Entity) {
        let place = entity.journey.is_none().then_some(entity.place);
        let key = (place, entity.lineage.clone(), entity.alive);
        self.filed.entry(key.clone()).or_default().insert(first);
        self.at.insert(first, key);
    }

    pub(crate) fn new(population: &Population) -> Self {
        let mut t = Self::default();
        t.touch(
            population,
            population.groups.keys().copied().collect::<Vec<_>>(),
        );
        t
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there.
    pub(crate) fn touch(&mut self, population: &Population, firsts: impl IntoIterator<Item = Id>) {
        for first in firsts {
            if let Some(key) = self.at.remove(&first)
                && let Some(set) = self.filed.get_mut(&key)
            {
                set.remove(&first);
                if set.is_empty() {
                    self.filed.remove(&key);
                }
            }
            if let Some(group) = population.groups.get(&first) {
                self.keep(first, &group.entity);
            }
        }
    }

    /// The filed groups a selector could accept from `place`, each list in
    /// identity order.
    pub(crate) fn lists<'a>(
        &'a self,
        selector: &'a Target,
        place: Id,
    ) -> Box<dyn Iterator<Item = &'a BTreeSet<Id>> + 'a> {
        let lineage = |key: &Filing| {
            selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| *lineage == key.1)
                && (selector.among.is_empty() || selector.among.contains(&key.1))
                && selector.alive.is_none_or(|alive| alive == key.2)
        };
        if selector.same_place {
            let at = (Some(place), Key::new(), false)..;
            let here = self.filed.range(at).take_while(move |(k, _)| k.0 == Some(place));
            Box::new(here.filter(move |(k, _)| lineage(k)).map(|(_, v)| v))
        } else {
            Box::new(
                self.filed
                    .iter()
                    .filter(move |(k, _)| lineage(k))
                    .map(|(_, v)| v),
            )
        }
    }
}

impl Simulation {
    /// Resident food holders admitted by the named physical food contract.
    /// Does not read appetite, due clocks, or role-labelled prey sets.
    pub fn available_food(&self, actor: Id, feed: &str) -> crate::Result<u64> {
        let body = self.body_at_start(actor).ok_or("food actor is missing")?;
        let Some(site) = body.resident_site().filter(|_| body.alive) else { return Ok(0) };
        let p = self.genesis.rules.processes.get(feed).ok_or("food contract is missing")?;
        if p.port_feeding.as_ref().is_none_or(|f| f.meal.is_none()) { return Err("food count requires an actual-part meal contract".into()); }
        if p.requires.iter().any(|q| matches!(q, Query::Trait { who: Binding::Actor, key } if !body.traits.contains(key))) {
            return Err("food actor does not carry the feed's inherited identity".into());
        }
        let mut selector = p.target.clone().ok_or("food contract has no target selector")?;
        selector.same_place = true;
        let mut total = 0u64;
        for first in self.searched(&selector, site) {
            let Some((first, count)) = self.target_span(first) else { continue };
            let Some(prey) = self.body_at_start(first).filter(|e| e.resident_site() == Some(site) && !self.genesis.rules.dormant(e)) else { continue };
            if p.food_weight(body, prey, &self.genesis.rules, &self.state.lineages) > 0 {
                let own = u64::from(first <= actor && actor < first + count);
                total = total.saturating_add(count - own);
            }
        }
        Ok(total)
    }
    fn target_span(&self, id: Id) -> Option<(Id, u64)> {
        let population = &self.state.population;
        match &self.pass {
            Some(pass) => pass.target_span(population, id),
            None => {
                let (&first, group) = population.groups.range(..=id).next_back()?;
                (id < first + group.count).then_some((first, group.count))
            },
        }
    }

    /// The live members of `among` resident at `site` as the pass began,
    /// each group counted whole as `weighed` counts it.
    pub(crate) fn members_at(&self, site: Id, among: &BTreeSet<Key>) -> u64 {
        let selector = Target { same_place: true, alive: Some(true), lineage: None, among: among.clone(), weighted: false };
        let here = |first: &Id| self.body_at_start(*first)
            .is_some_and(|b| b.alive && among.contains(&b.lineage) && b.resident_site() == Some(site) && !self.genesis.rules.dormant(b));
        let firsts = self.searched(&selector, site).into_iter().filter(here);
        firsts.filter_map(|first| self.target_span(first).map(|(_, count)| count)).sum()
    }

    /// The matter the live members of `lineage` hold at `site` as the pass
    /// began, each group counted whole (a territory's gradient, 1071).
    pub(crate) fn matter_at(&self, site: Id, lineage: &Key) -> u128 {
        let among = BTreeSet::from([lineage.clone()]);
        let selector = Target { same_place: true, alive: Some(true), lineage: None, among: among.clone(), weighted: false };
        let here = |first: &Id| self.body_at_start(*first)
            .is_some_and(|b| b.alive && &b.lineage == lineage && b.resident_site() == Some(site));
        self.searched(&selector, site).into_iter().filter(here).filter_map(|first| {
            let (_, count) = self.target_span(first)?;
            let each = crate::meaning::mass(self.books_at_start(first)?.as_ref(), &self.genesis.rules);
            Some(each.saturating_mul(u128::from(count)))
        }).fold(0u128, u128::saturating_add)
    }

    /// Live index pieces and preserved bodies can name the same original
    /// group. Recover its span once, with its original weight and order.
    fn target_firsts(&self, firsts: impl Iterator<Item = Id>) -> Vec<Id> {
        let mut firsts: Vec<_> = firsts
            .filter_map(|id| self.target_span(id).map(|(first, _)| first))
            .collect();
        firsts.sort_unstable();
        firsts.dedup();
        firsts
    }

    pub(crate) fn choose_target(&self, actor: Id, p: &Process) -> Option<Id> {
        if let Some(target)=self.institution_target(actor,p) {return target.ok().flatten();}
        if let Some(target) = self.foraging_target(actor, p) { return target; }
        let selector = p.target.as_ref()?;
        let (place, offered) = self.offering(actor, p)?;
        let accepting = |first: &Id| offered(*first).is_some();
        let found = |firsts: Vec<Id>| {
            if selector.weighted {
                self.drawn(actor, p, firsts.into_iter().filter(accepting))
            } else {
                firsts.into_iter().find_map(&offered)
            }
        };
        let filed = found(self.searched(selector, place));
        debug_assert!(
            self.targets.is_none() || filed == found(self.target_firsts(self.state.population.groups.keys().copied())),
            "the filed search missed a group"
        );
        filed
    }

    /// The prey matter an act of `p` by `actor` could draw from: the weight
    /// the ordinary search shares among the groups it accepts, so read by the
    /// actor's own intake (D35, D37).
    pub(crate) fn prey_matter(&self, actor: Id, p: &Process) -> u128 {
        let (Some(selector), Some((place, offered))) = (p.target.as_ref(), self.offering(actor, p)) else {
            return 0;
        };
        let accepted = self.searched(selector, place).into_iter().filter(|first| offered(*first).is_some());
        self.weighed(actor, p, accepted).1
    }

    /// Where `actor`'s search for `p` looks from, and the member each group
    /// offers it, by the group's first member; None where the actor itself is
    /// refused.
    fn offering<'a>(&'a self, actor: Id, p: &'a Process) -> Option<(Id, impl Fn(Id) -> Option<Id> + 'a)> {
        let selector = p.target.as_ref()?;
        let entity = self.body_at_start(actor)?;
        // A traveller is colocated with no one, so a selector that wants its
        // place accepts nothing; the index files it under its last site, whose
        // whole population the search would otherwise read and refuse.
        if selector.same_place && !self.state.movement.immediate() && entity.resident_site().is_none() {
            return None;
        }
        let place = entity.place;
        let part = self.bind_part(actor, p);
        // A requirement that cannot read the prey holds for every candidate
        // or for none, so it is asked once, before the search; only the rest
        // are asked of each candidate. Apply still checks every requirement.
        let (of_prey, of_actor): (Vec<&Query>, Vec<&Query>) = p.requires.iter().partition(|q| reads_prey(q));
        if !of_actor.iter().all(|q| self.process_query(actor, None, place, part, p, q).is_ok()) {
            return None;
        }
        // A group offers its first member, or its second when the first is
        // the actor; identities never reach the top of their range, so the
        // second always has a number. Equal members read alike, so the
        // requirements asked of one are asked for the group.
        let offered = move |first: Id| -> Option<Id> {
            let (_, count) = self.target_span(first)?;
            let candidate = if first == actor { first + 1 } else { first };
            let accepted = candidate < first + count
                && !self.body_at_start(first).is_some_and(|b| self.genesis.rules.dormant(b))
                && self.target_matches(actor, Some(candidate), p)
                && of_prey
                    .iter()
                    .all(|q| self.process_query(actor, Some(candidate), place, part, p, q).is_ok());
            accepted.then_some(candidate)
        };
        Some((place, offered))
    }

    /// The groups a search by `selector` from `place` reads, by first
    /// member: the filed lists during an advance, every group otherwise.
    fn searched(&self, selector: &Target, place: Id) -> Vec<Id> {
        match &self.targets {
            // Every group outside the lists fails the selector anyway.
            Some(t) => {
                let kept = self
                    .frame
                    .iter()
                    .flat_map(|f| f.targets.lists(selector, place).flatten().copied());
                self.target_firsts(t.lists(selector, place).flatten().copied().chain(kept))
            },
            None => self.target_firsts(self.state.population.groups.keys().copied()),
        }
    }

    /// The accepting groups `firsts` weighed for a draw, in identity order:
    /// each member but the actor by the matter it holds, so the same member
    /// is drawn however its equals are grouped; and their total.
    fn weighed(&self, actor: Id, p: &Process, firsts: impl Iterator<Item = Id>) -> (Vec<(Id, bool, u128, u128)>, u128) {
        let rules = &self.genesis.rules;
        let mut eligible = Vec::new();
        let mut total = 0u128;
        for first in firsts {
            let (_, count) = self.target_span(first).expect("original candidate span");
            let within = first <= actor && actor < first + count;
            let members = u128::from(count - u64::from(within));
            // A group's members began the pass alike: an act lifts out the
            // member it changes, or writes a cohort whole.
            let held = if p.port_feeding.as_ref().is_some_and(|f| f.meal.is_some()) {
                p.food_weight(self.body_at_start(actor).expect("actor exists"), self.body_at_start(first).expect("prey exists"), rules, &self.state.lineages)
            } else { mass(&self.books_at_start(first).expect("filed groups exist"), rules) };
            if members > 0 && held > 0 {
                total += members * held;
                eligible.push((first, within, members, held));
            }
        }
        (eligible, total)
    }

    /// The member drawn among the accepting groups `firsts`, each member
    /// weighted by the matter it holds.
    fn drawn(&self, actor: Id, p: &Process, firsts: impl Iterator<Item = Id>) -> Option<Id> {
        let (eligible, total) = self.weighed(actor, p, firsts);
        if total == 0 {
            return None;
        }
        let key = format!("target:{}", p.id);
        let x = crate::draw(
            self.genesis.dynamics_seed(),
            &key,
            &[self.state.tick, actor],
        );
        let mut pick = (u128::from(x) * total) >> 64;
        for (first, within, members, held) in eligible {
            let weight = members * held;
            if pick < weight {
                let mut id = first + (pick / held) as u64;
                if within && id >= actor {
                    id += 1;
                }
                return Some(id);
            }
            pick -= weight;
        }
        unreachable!("the draw falls within the total weight")
    }
}

#[cfg(test)]
mod edible_tests;
#[cfg(test)]
mod tests;
