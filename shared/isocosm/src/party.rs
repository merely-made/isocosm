// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A player-defined faction is relations and recorded disclosures, not a polity (8, 853–858).

use crate::{Result, Session, Command, Simulation, rules::Rules, schema::*, asserted};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Join { participant: Id, party: Key, member: Id },
    Leave { participant: Id, party: Key, member: Id },
    Disclose { participant: Id, party: Key, member: Id, sites: BTreeSet<Id>, events: BTreeSet<Key> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership { pub participant: Id, pub member: Id }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place { pub site: Id, pub authored: Option<asserted::Place> }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Route { pub from: Id, pub to: Id, pub travel: Tick }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disclosure {
    pub participant: Id, pub party: Key, pub member: Id, pub tick: Tick,
    pub places: Vec<Place>, pub routes: Vec<Route>, pub events: BTreeSet<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome { Joined(Membership), Left(Membership), Disclosed(Disclosure) }

/// A read of accepted records. No shared fact becomes a member's native knowing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedMap {
    pub party: Key, pub members: Vec<Membership>, pub places: BTreeMap<Id, Place>,
    pub routes: BTreeSet<Route>, pub events: BTreeSet<Key>, pub records: Vec<Key>,
}

pub fn validate(rules: &Rules) -> Result<()> {
    let mut kinds = BTreeSet::new();
    for (party, kind) in &rules.party_relations {
        crate::validation::key(party)?;
        crate::validation::key(kind)?;
        let intrinsic = matches!(kind.as_str(), "sim:parent" | "sim:child"
            | crate::directing::PLAYS | crate::directing::BOND);
        let custody = rules.projects.as_ref().is_some_and(|catalogue| catalogue.definitions.values()
            .any(|definition| definition.output_custody.as_ref() == Some(kind)));
        if !rules.relations.contains(kind) || !kinds.insert(kind) || intrinsic || custody {
            return Err("parties require distinct explicitly declared membership relation kinds".into());
        }
    }
    Ok(())
}

fn relation(sim: &Simulation, participant: Id, party: &str, member: Id) -> Result<Relation> {
    if !sim.state().population.get(participant).is_some_and(crate::directing::is_participant) {
        return Err("party caller is not a native participant".into());
    }
    if sim.state().population.get(member).is_none() { return Err("party body is absent".into()); }
    let kind = sim.genesis().rules.party_relations.get(party).ok_or("party handle is not authored in this world")?;
    Ok(Relation::new(participant, kind.clone(), member))
}

pub fn members(sim: &Simulation, party: &str) -> Result<Vec<Membership>> {
    let kind = sim.genesis().rules.party_relations.get(party).ok_or("party handle is not authored in this world")?;
    sim.state().relations.iter().filter(|r| &r.kind == kind).map(|r| {
        if r.value != 0 || !sim.state().population.get(r.subject).is_some_and(crate::directing::is_participant)
            || sim.state().population.get(r.object).is_none() {
            return Err("party relation does not name its actual participant and body".into());
        }
        Ok(Membership { participant: r.subject, member: r.object })
    }).collect()
}

/// The source a participant can deliberately disclose. Journey has no resident origin.
pub fn knowledge(sim: &Simulation, member: Id) -> Result<crate::directing::readings::View> {
    if sim.state().population.get(member).is_none() { return Err("party body is absent".into()); }
    Ok(crate::directing::readings::view(sim, member, crate::directing::readings::Mode::Survival))
}

impl Simulation {
    pub(crate) fn party_change(&mut self, change: &Change) -> Result<String> {
        let (participant, party, member) = match change {
            Change::Join { participant, party, member } | Change::Leave { participant, party, member }
            | Change::Disclose { participant, party, member, .. } => (*participant, party, *member),
        };
        let held = relation(self, participant, party, member)?;
        let outcome = match change {
            Change::Join { .. } => {
                if self.plays(participant) != Some(member) { return Err("only the player's currently played body opts in".into()); }
                self.state.relations.insert(held);
                Outcome::Joined(Membership { participant, member })
            },
            Change::Leave { .. } => {
                // 857: the participant owns its earlier opt-in even after Take.
                if !self.state.relations.remove(&held) { return Err("participant has no such earlier party membership".into()); }
                Outcome::Left(Membership { participant, member })
            },
            Change::Disclose { sites, events, .. } => {
                if self.plays(participant) != Some(member) || !self.state.relations.contains(&held) {
                    return Err("only the opted-in currently played body may disclose its map".into());
                }
                let known = knowledge(self, member)?;
                if !sites.is_subset(&known.sites) || !events.is_subset(&known.events)
                    || sites.iter().any(|id| !self.state.sites.contains_key(id)) {
                    return Err("party disclosure contains facts outside that body's native knowledge".into());
                }
                let places = sites.iter().map(|&site| Place { site, authored: self.state.sites[&site].authored.clone() }).collect();
                let routes = sites.iter().flat_map(|from| self.state.sites[from].routes.iter()
                    .filter(|r| sites.contains(&r.to)).map(move |r| Route { from: *from, to: r.to, travel: r.travel })).collect();
                Outcome::Disclosed(Disclosure { participant, party: party.clone(), member, tick: self.state.tick,
                    places, routes, events: events.clone() })
            },
        };
        serde_json::to_string(&outcome).map_err(|error| error.to_string())
    }
}

/// The caller borrows an already authenticated Session. This getter never replays another world.
pub fn shared_map(session: &Session, party: &str) -> Result<SharedMap> {
    let mut map = SharedMap { party: party.into(), members: members(&session.sim, party)?,
        places: BTreeMap::new(), routes: BTreeSet::new(), events: BTreeSet::new(), records: Vec::new() };
    for entry in &session.entries {
        let Command::Party(Change::Disclose { participant, party: declared, member, sites, events }) = &entry.command else { continue; };
        if declared != party { continue; }
        let Outcome::Disclosed(record) = serde_json::from_str(&entry.outcome).map_err(|error| error.to_string())? else {
            return Err("party disclosure has no accepted native outcome".into());
        };
        let named: BTreeSet<_> = record.places.iter().map(|p| p.site).collect();
        if record.party != party || record.participant != *participant || record.member != *member
            || record.tick != entry.tick || named != *sites || record.places.len() != sites.len()
            || record.events != *events || record.routes.iter().any(|r| !sites.contains(&r.from) || !sites.contains(&r.to)) {
            return Err("party record differs from its accepted native disclosure".into());
        }
        // 858: leaving withdraws membership, not already disclosed records.
        for place in record.places { map.places.insert(place.site, place); }
        map.routes.extend(record.routes);
        map.events.extend(record.events);
        map.records.push(entry.id.clone());
    }
    Ok(map)
}
