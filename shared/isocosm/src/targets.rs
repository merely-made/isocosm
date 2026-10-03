// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a scheduled act with a target finds it. By default, the first
//! member in identity order the process's selector and requirements accept,
//! one per stored group. A weighted selector, feeding's (ruling 287), draws
//! instead among every member it would accept, seeded and keyed by the act,
//! each weighted by the matter it holds, so abundant prey are eaten more.
//! During an advance the groups are filed by place and lineage, so a
//! selector that names either looks only where it could match instead of
//! across the whole population (ruling 237). Every member is read as the
//! pass under way began, so what earlier acts of the pass did to a member
//! changes neither whether it is accepted nor what it weighs (ruling 454).

use crate::{meaning::mass, population::Population, rules::*, schema::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

/// Stored groups by their first identity, filed by place and lineage.
#[derive(Clone, Debug, Default)]
pub(crate) struct Targets {
    filed: BTreeMap<(Id, Key), BTreeSet<Id>>,
    at: BTreeMap<Id, (Id, Key)>,
}

impl Targets {
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
                let key = (group.entity.place, group.entity.lineage.clone());
                self.filed.entry(key.clone()).or_default().insert(first);
                self.at.insert(first, key);
            }
        }
    }

    /// The filed groups a selector could accept from `place`, each list in
    /// identity order.
    fn lists<'a>(
        &'a self,
        selector: &'a Target,
        place: Id,
    ) -> Box<dyn Iterator<Item = &'a BTreeSet<Id>> + 'a> {
        let lineage = |key: &(Id, Key)| {
            selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| *lineage == key.1)
                && (selector.among.is_empty() || selector.among.contains(&key.1))
        };
        if selector.same_place {
            let at = (place, Key::new())..;
            let here = self.filed.range(at).take_while(move |(k, _)| k.0 == place);
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
    pub(crate) fn choose_target(&self, actor: Id, p: &Process) -> Option<Id> {
        let selector = p.target.as_ref()?;
        let entity = self.body_at_start(actor)?;
        // Reject an ineligible actor before searching the population for food.
        // This is only a read shortcut; apply still checks every requirement.
        if p.requires.iter().any(|q| {
            matches!(q, Query::Trait {who:Binding::Actor,key}
            if !entity.traits.contains(key))
        }) {
            return None;
        }
        let place = entity.place;
        let part = self.bind_part(actor, p);
        let groups = &self.state.population.groups;
        // A group offers its first member, or its second when the first is
        // the actor; identities never reach the top of their range, so the
        // second always has a number. Equal members read alike, so the
        // requirements asked of one are asked for the group.
        let offered = |first: Id| -> Option<Id> {
            let count = groups.get(&first)?.count;
            let candidate = if first == actor { first + 1 } else { first };
            let accepted = candidate < first + count
                && self.target_matches(actor, Some(candidate), p)
                && p.requires
                    .iter()
                    .all(|q| self.query(actor, Some(candidate), place, part, q).is_ok());
            accepted.then_some(candidate)
        };
        let accepting = |first: &Id| offered(*first).is_some();
        let everywhere = || {
            if selector.weighted {
                self.drawn(actor, p, groups.keys().copied().filter(accepting))
            } else {
                groups.keys().find_map(|&first| offered(first))
            }
        };
        match &self.targets {
            // Every group outside the lists fails the selector anyway.
            Some(t) => {
                let found = if selector.weighted {
                    let mut filed: Vec<Id> = t.lists(selector, place).flatten().copied().collect();
                    filed.sort_unstable();
                    self.drawn(actor, p, filed.into_iter().filter(accepting))
                } else {
                    // Each list is in identity order, so its first accepted
                    // member is its least.
                    t.lists(selector, place)
                        .filter_map(|list| list.iter().find_map(|&first| offered(first)))
                        .min()
                };
                debug_assert_eq!(found, everywhere(), "the filed search missed a group");
                found
            },
            None => everywhere(),
        }
    }

    /// The member drawn among the accepting groups `firsts`, in identity
    /// order: each member but the actor weighted by the matter it holds, so
    /// the same member is drawn however its equals are grouped.
    fn drawn(&self, actor: Id, p: &Process, firsts: impl Iterator<Item = Id>) -> Option<Id> {
        let rules = &self.genesis.rules;
        let groups = &self.state.population.groups;
        let mut eligible = Vec::new();
        let mut total = 0u128;
        for first in firsts {
            let group = &groups[&first];
            let within = first <= actor && actor < first + group.count;
            let members = u128::from(group.count - u64::from(within));
            // A group's members began the pass alike: an act lifts out the
            // member it changes, or writes a cohort whole.
            let began = self.body_at_start(first).expect("filed groups exist");
            let held = mass(&crate::anatomy::books(began), rules);
            if members > 0 && held > 0 {
                total += members * held;
                eligible.push((first, within, members, held));
            }
        }
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
