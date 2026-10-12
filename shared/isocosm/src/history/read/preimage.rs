// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The original fourteen-field whole-state preimage, not today's witness.
//! Geometry metadata comes only from the already authenticated legacy genesis.

use super::{wire::*, *};
use crate::population::Population;
use std::collections::BTreeSet;

pub(super) struct Family {
    parts: BTreeMap<(Key, Id), LegacyPart>,
}

impl Family {
    pub(super) fn new(genesis: &LegacyGenesis) -> Result<Self> {
        if genesis.lineages.values().any(|l| l.development.is_some()) {
            return Err("unsupported legacy development".into());
        }
        let mut parts = BTreeMap::new();
        for group in genesis.population.groups.values() {
            let e = &group.entity;
            if !matches!(e.method, Method::Inert | Method::Reactive) {
                return Err("unsupported legacy deliberation".into());
            }
            for (&id, part) in &e.parts {
                if !part.virtual_part() || u32::try_from(id).is_err() {
                    return Err("unsupported legacy part geometry or physiology".into());
                }
                // Traits can differ among members. The retained geometry cannot.
                let mut metadata = part.clone();
                metadata.traits.clear();
                let key = (e.lineage.clone(), id);
                if parts.get(&key).is_some_and(|p| *p != metadata) {
                    return Err("ambiguous legacy part metadata".into());
                }
                parts.insert(key, metadata);
            }
        }
        Ok(Self { parts })
    }

    fn entity(&self, e: &Entity) -> Result<LegacyEntity> {
        if e.journey.is_some() || e.body.is_some() || !e.systems.is_empty()
            || !e.varied.is_empty() || e.patch.is_some() || e.authored.is_some()
            || !matches!(e.method, Method::Inert | Method::Reactive)
        {
            return Err("unsupported facts in legacy replay entity".into());
        }
        let mut parts = BTreeMap::new();
        for (id, part) in &e.parts {
            if !part.functions.is_empty() || !part.cells.is_empty() || !part.matter.is_empty()
                || !part.tracts.is_empty() || !part.lost.is_empty()
                || !part.losses.is_empty() || part.port.is_some()
            {
                return Err("unsupported physiology in legacy replay part".into());
            }
            let key = (e.lineage.clone(), u64::from(id.0));
            let mut held = self.parts.get(&key)
                .ok_or("legacy replay part has no authenticated metadata")?.clone();
            held.traits = part.traits.clone();
            parts.insert(u64::from(id.0), held);
        }
        Ok(LegacyEntity {
            lineage:e.lineage.clone(), kingdom:e.kingdom.clone(), scale:e.scale.clone(),
            provenance:e.provenance.clone(), method:e.method, place:e.place, arrived:e.arrived,
            visits:e.visits.clone(), born:e.born, alive:e.alive, body_revision:e.body_revision,
            parts, traits:e.traits.clone(), accounts:e.accounts.clone(), skills:e.skills.clone(),
            tenets:e.tenets.clone(), disposition:e.disposition, soma:e.soma.clone(),
        })
    }

    fn population(&self, population: &Population) -> Result<LegacyPopulation> {
        let population = population.normalized();
        let groups = population.groups.iter().map(|(&first, g)| {
            Ok((first, LegacyCohort { count:g.count, entity:self.entity(&g.entity)? }))
        }).collect::<Result<_>>()?;
        Ok(LegacyPopulation { groups, next_id:population.next_id })
    }
}

#[derive(Serialize)]
struct Event<'a> {
    id: &'a Key,
    tick: Tick,
    place: Id,
    subject: Id,
    process: &'a Key,
    cause: &'a Option<Key>,
    strength: u32,
    legend: bool,
}

#[derive(Serialize)]
struct State<'a> {
    tick: Tick,
    next_action: u64,
    population: LegacyPopulation,
    sites: &'a BTreeMap<Id, Site>,
    lineages: &'a BTreeMap<Key, Lineage>,
    locations: &'a BTreeMap<Id, Location>,
    polities: &'a BTreeMap<Id, Polity>,
    relations: &'a BTreeSet<Relation>,
    notes: &'a Vec<Note>,
    roots: &'a BTreeSet<Id>,
    released: &'a BTreeMap<Id, Tick>,
    record: &'a hagiograph::Record<Key, Id>,
    events: BTreeMap<&'a Key, Event<'a>>,
    reach: &'a crate::reach::Reach,
}

impl<'a> State<'a> {
    fn new(sim: &'a Simulation, family: &Family) -> Result<Self> {
        let s = sim.state();
        if !sim.genesis().spreads.is_empty() || !s.spreads.is_empty()
            || s.movement != Movement::ImmediateV3 || !s.edits.is_empty() || !s.nudges.is_empty()
            || !s.laws.is_empty() || !s.agreements.is_empty() || !s.asserted.is_empty()
            || !s.attention.is_empty() || s.relations.iter().any(|r| r.value != 0)
            || s.polities.values().any(|p| p.authored.is_some())
            || s.sites.values().any(|p| p.authored.is_some()
                || p.routes.iter().any(|r| r.authored.is_some()))
            || s.lineages.values().any(|l| l.development.is_some())
        {
            return Err("unsupported facts in legacy replay state".into());
        }
        let events = s.events.iter().map(|(key, e)| {
            if e.object.is_some() || e.agreement.is_some() || e.authored.is_some() {
                return Err("unsupported facts in legacy replay event".into());
            }
            let place = e.place.site().ok_or("legacy event has no resident site")?;
            Ok((key, Event { id:&e.id, tick:e.tick, place, subject:e.subject,
                process:&e.process, cause:&e.cause, strength:e.strength, legend:e.legend }))
        }).collect::<Result<_>>()?;
        Ok(Self { tick:s.tick, next_action:s.next_action,
            population:family.population(&s.population)?, sites:&s.sites, lineages:&s.lineages,
            locations:&s.locations, polities:&s.polities, relations:&s.relations, notes:&s.notes,
            roots:&s.roots, released:&s.released, record:&s.record, events, reach:&s.reach })
    }
}

pub(super) fn sha(sim: &Simulation, family: &Family) -> Result<Key> {
    Ok(crate::digest(&(sim.genesis().seed, &sim.revision, &sim.genesis().world,
        State::new(sim, family)?)))
}

pub(super) fn fnv(sim: &Simulation, family: &Family) -> Result<u64> {
    use isometer_core::snapshot::{encode, hash_bytes};
    let bytes = encode(&(sim.genesis().seed, &sim.revision, &sim.genesis().world,
        State::new(sim, family)?)).map_err(|e| format!("{e:?}"))?;
    Ok(hash_bytes(&bytes))
}
