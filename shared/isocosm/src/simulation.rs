// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, population::Population, reach::Reach, rules::*, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Execution {
    Individuals,
    Grouped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genesis {
    pub version: u32,
    pub seed: u64,
    /// Seeds how the world unfolds, apart from how it was founded, so one
    /// founded world can run under independent draws. Absent means the world
    /// seed, and absent worlds serialize exactly as before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<u64>,
    pub founding: Option<crate::Founding>,
    pub world: WorldTraits,
    pub rules: Rules,
    pub lineages: BTreeMap<Key, Lineage>,
    pub sites: BTreeMap<Id, Site>,
    pub population: Population,
}

impl Genesis {
    pub fn dynamics_seed(&self) -> u64 {
        self.dynamics.unwrap_or(self.seed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub tick: Tick,
    pub next_action: u64,
    pub population: Population,
    pub sites: BTreeMap<Id, Site>,
    pub lineages: BTreeMap<Key, Lineage>,
    pub locations: BTreeMap<Id, Location>,
    pub polities: BTreeMap<Id, Polity>,
    pub relations: BTreeSet<Relation>,
    pub notes: Vec<Note>,
    pub roots: BTreeSet<Id>,
    pub released: BTreeMap<Id, Tick>,
    pub record: hagiograph::Record<Key, Id>,
    pub events: BTreeMap<Key, Event>,
    pub reach: Reach,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Accepted,
    Blocked(String),
    Refused(String),
    RiskOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub id: Key,
    pub tick: Tick,
    pub revision: Key,
    pub actor: Id,
    pub target: Option<Id>,
    pub count: u64,
    pub chosen: Key,
    pub foregone: Vec<Key>,
    pub cause: Option<Key>,
    pub facts_read: Vec<String>,
    pub effects: Vec<Effect>,
    pub outcome: Outcome,
    pub matter_before: u128,
    pub matter_after: u128,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub evaluations: u64,
    pub represented: u64,
    pub accepted: u64,
    pub blocked: u64,
}

#[derive(Clone, Debug)]
pub struct Simulation {
    pub(crate) genesis: std::sync::Arc<Genesis>,
    pub(crate) state: State,
    pub(crate) mode: Execution,
    pub(crate) revision: Key,
}

impl Simulation {
    pub fn new(genesis: Genesis, mode: Execution) -> Result<Self> {
        genesis.validate()?;
        let revision = genesis.rules.revision();
        let state = State {
            tick: 0,
            next_action: 0,
            population: genesis.population.clone(),
            sites: genesis.sites.clone(),
            lineages: genesis.lineages.clone(),
            locations: BTreeMap::new(),
            polities: BTreeMap::new(),
            relations: BTreeSet::new(),
            notes: Vec::new(),
            roots: BTreeSet::new(),
            released: BTreeMap::new(),
            record: Default::default(),
            events: BTreeMap::new(),
            reach: Reach::default(),
        };
        Ok(Self {
            genesis: std::sync::Arc::new(genesis),
            state,
            mode,
            revision,
        })
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn genesis(&self) -> &Genesis {
        &self.genesis
    }
    pub fn mode(&self) -> Execution {
        self.mode
    }
    pub fn set_mode(&mut self, mode: Execution) {
        self.mode = mode;
    }
    pub fn state_hash(&self) -> Key {
        let mut state = self.state.clone();
        state.population = state.population.normalized();
        crate::digest(&(
            self.genesis.seed,
            &self.revision,
            &self.genesis.world,
            state,
        ))
    }
    pub fn matter(&self) -> u128 {
        matter(&self.state, &self.genesis.rules)
    }
    pub fn execute(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        cause: Option<Key>,
    ) -> Receipt {
        self.apply(actor, target, process, cause, 1)
    }
    pub fn advance(&mut self, ticks: Tick) -> Result<Work> {
        if ticks > self.genesis.rules.limits.advance_ticks {
            return Err("advance exceeds configured tick budget".into());
        }
        let end = self
            .state
            .tick
            .checked_add(ticks)
            .ok_or("clock exhausted")?;
        let mut candidate = self.clone();
        let work = candidate.advance_to(end)?;
        *self = candidate;
        Ok(work)
    }
    fn advance_to(&mut self, end: Tick) -> Result<Work> {
        let mut queue = BTreeSet::new();
        for p in self.genesis.rules.processes.values() {
            if let Some(period) = p.period
                && let Some(due) = self
                    .state
                    .tick
                    .checked_add(period - self.state.tick % period)
                && due <= end
            {
                queue.insert((due, p.priority, p.id.clone()));
            }
        }
        let mut work = Work::default();
        while let Some((due, priority, id)) = queue.pop_first() {
            self.state.tick = due;
            let process = self.genesis.rules.processes[&id].clone();
            let groups: Vec<_> = self
                .state
                .population
                .groups
                .iter()
                .map(|(&first, g)| (first, g.count))
                .collect();
            for (first, count) in groups {
                let Some(entity) = self.state.population.get(first) else {
                    continue;
                };
                if !entity.alive {
                    continue;
                }
                if process.shape == Shape::Agentless {
                    if entity.kingdom != "kingdom:world" {
                        continue;
                    }
                } else if entity.method == Method::Inert {
                    continue;
                }
                if process.need_account.as_ref().is_some_and(|a| {
                    entity.accounts.get(a).copied().unwrap_or(0) >= process.need_below
                }) {
                    continue;
                }
                let bulk = self.mode == Execution::Grouped && process.bulk_safe();
                let calls = if bulk { 1 } else { count };
                if work.evaluations.saturating_add(calls)
                    > self.genesis.rules.limits.events_per_advance as u64
                {
                    return Err(
                        "advance exceeds configured operation budget; use shorter advances".into(),
                    );
                }
                for offset in 0..calls {
                    let multiplicity = if bulk { count } else { 1 };
                    let actor = first + offset;
                    let target = self.choose_target(actor, &process);
                    let r = self.apply(actor, target, &id, None, multiplicity);
                    work.evaluations += 1;
                    work.represented += multiplicity;
                    if matches!(r.outcome, Outcome::Accepted | Outcome::RiskOutcome) {
                        work.accepted += multiplicity;
                    } else {
                        work.blocked += multiplicity;
                    }
                }
            }
            let next = due.checked_add(process.period.unwrap());
            if let Some(next) = next.filter(|next| *next <= end) {
                queue.insert((next, priority, id));
            }
        }
        self.state.tick = end;
        if self.mode == Execution::Grouped {
            let roots = self.kept();
            self.state.population.restrict(&roots);
        }
        Ok(work)
    }
    pub fn inspect(&mut self, id: Id) -> Result<()> {
        self.state.population.lift(id)?;
        self.state.roots.insert(id);
        self.state.released.remove(&id);
        Ok(())
    }
    pub fn release(&mut self, id: Id) {
        if self.state.roots.remove(&id) {
            self.state.released.insert(
                id,
                self.state
                    .tick
                    .saturating_add(self.genesis.rules.collection_buffer),
            );
        }
    }
    pub fn kept(&self) -> BTreeSet<Id> {
        let mut roots = self.state.roots.clone();
        roots.extend(
            self.state
                .released
                .iter()
                .filter(|(_, until)| **until > self.state.tick)
                .map(|(id, _)| *id),
        );
        for note in &self.state.notes {
            if note.core.stance == wing_impresa::Stance::Associate
                && note.expires.is_none_or(|t| t > self.state.tick)
            {
                for subject in [&note.core.subject, &note.core.object] {
                    if let Some(id) = subject.strip_prefix("entity:").and_then(|s| s.parse().ok()) {
                        roots.insert(id);
                    }
                }
            }
        }
        for event in self.state.events.values().filter(|e| e.legend) {
            roots.insert(event.subject);
        }
        loop {
            let before = roots.len();
            for relation in &self.state.relations {
                if roots.contains(&relation.subject) {
                    roots.insert(relation.object);
                }
            }
            for p in self
                .state
                .polities
                .values()
                .filter(|p| p.ended_by.is_none())
            {
                roots.extend(&p.constitution.members);
            }
            if roots.len() == before {
                break;
            }
        }
        roots
    }
    /// Conservative collection: fold only exact equivalent state. No observed
    /// deviation or historical event is deleted to meet a memory budget.
    pub fn collect(&mut self) -> usize {
        let before = self.state.population.groups.len();
        let roots = self.kept();
        self.state.population.restrict(&roots);
        before.saturating_sub(self.state.population.groups.len())
    }
    pub fn knows(&self, entity: Id, event: &str) -> Result<bool> {
        let e = self.state.population.get(entity).ok_or("unknown entity")?;
        let event = self.state.events.get(event).ok_or("unknown event")?;
        if self.state.notes.iter().any(|n| {
            n.core.subject == format!("entity:{entity}")
                && n.core.object == event.id
                && n.core.kind == "sim:discover"
        }) {
            return Ok(true);
        }
        let field = &self.genesis.rules.field;
        let mut exposure = self.state.reach.exposure(
            event,
            e.place,
            e.arrived,
            self.state.tick.saturating_add(1),
            field,
        );
        for visit in &e.visits {
            exposure = exposure.saturating_add(self.state.reach.exposure(
                event,
                visit.place,
                visit.from,
                visit.until,
                field,
            ));
        }
        Ok(crate::draw(
            self.genesis.dynamics_seed(),
            &format!("knowing:{}", event.id),
            &[entity],
        ) % 1_000_000
            < exposure.min(1_000_000))
    }
    pub fn learn(&mut self, entity: Id, event: &str) -> Result<bool> {
        if !self.knows(entity, event)? {
            return Ok(false);
        }
        if self.state.notes.iter().any(|n| {
            n.core.subject == format!("entity:{entity}")
                && n.core.object == event
                && n.core.kind == "sim:discover"
        }) {
            return Ok(true);
        }
        let mut candidate = self.clone();
        candidate.inspect(entity)?;
        candidate.add_note(
            entity,
            event.into(),
            "sim:discover",
            String::new(),
            None,
            event.into(),
        )?;
        self.state = candidate.state;
        Ok(true)
    }
    pub(crate) fn add_note(
        &mut self,
        subject: Id,
        object: Key,
        kind: &str,
        djot: String,
        expires: Option<Tick>,
        cause: Key,
    ) -> Result<()> {
        if self.state.notes.len() >= self.genesis.rules.limits.notes {
            return Err("note budget exhausted".into());
        }
        if !self.genesis.rules.note_kinds.contains(kind) {
            return Err("unknown note kind".into());
        }
        self.state.notes.push(Note {
            core: wing_impresa::Record {
                subject: format!("entity:{subject}"),
                object,
                kind: kind.into(),
                canon_revision: self.genesis.world.canon.revision,
                cause,
                tick: self.state.tick,
                stance: wing_impresa::Stance::Associate,
            },
            djot,
            extra: BTreeMap::new(),
            expires,
        });
        Ok(())
    }
}

pub(crate) fn matter(state: &State, rules: &Rules) -> u128 {
    let mass = |a: &Ledger| -> u128 {
        a.iter()
            .filter(|(k, _)| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. })))
            .map(|(_, v)| u128::from(*v))
            .sum()
    };
    state
        .population
        .groups
        .values()
        .map(|g| mass(&g.entity.accounts) * u128::from(g.count))
        .sum::<u128>()
        + state
            .sites
            .values()
            .map(|s| mass(&s.accounts))
            .sum::<u128>()
        + state
            .polities
            .values()
            .map(|p| mass(&p.accounts))
            .sum::<u128>()
}
