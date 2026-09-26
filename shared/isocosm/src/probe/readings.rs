// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What later processes read (ruling 113), taken mechanically from the
//! definitions. Each threshold a process, a need or the competition asks of
//! a member counts the members of its kind that meet it, alive or dead; each
//! asked of a site counts the sites; each competing kind counts its living,
//! and its living whose strain has passed the bearing its fights read. An
//! inspection shows one member drawn uniformly, field by field, for every
//! field an effect can write or the founding sets.

use super::{
    Crowd, ExactRun, ProbeWorld,
    aggregate::{self, Seen},
    draws::Stream,
    fight,
};
use crate::{
    Result,
    rules::{Binding, Effect, Query},
    schema::*,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Field {
    Lineage,
    Place,
    Alive,
    Born,
    Account(Key),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Probe {
    Alive {
        identity: Key,
    },
    Members {
        selector: Vec<Key>,
        query: Query,
    },
    Sites {
        query: Query,
    },
    /// Members of a kind whose kept strain has passed their bearing.
    PastBearing {
        identity: Key,
    },
    Inspect {
        field: Field,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct Reading {
    pub key: Key,
    pub source: Key,
    pub probe: Probe,
    /// Read from a process whose effect is death: the starvation check.
    pub starvation: bool,
}

fn name(field: &Field) -> String {
    match field {
        Field::Lineage => "lineage".into(),
        Field::Place => "place".into(),
        Field::Alive => "alive".into(),
        Field::Born => "born".into(),
        Field::Account(k) => format!("account:{k}"),
    }
}

fn effects(world: &ProbeWorld) -> Vec<&Effect> {
    world
        .genesis
        .rules
        .processes
        .values()
        .flat_map(|p| p.commitments.iter().chain(&p.effects))
        .collect()
}

fn inspected(world: &ProbeWorld) -> Vec<Field> {
    let effects = effects(world);
    let mut accounts = BTreeSet::new();
    for e in &effects {
        match e {
            Effect::Transform {
                who: Binding::Actor,
                take,
                give,
            } => accounts.extend(take.keys().chain(give.keys()).cloned()),
            Effect::Transfer {
                from, to, account, ..
            } if *from == Binding::Actor || *to == Binding::Actor => {
                accounts.insert(account.clone());
            },
            Effect::Ease {
                who: Binding::Actor,
                key,
                ..
            } => {
                accounts.insert(key.clone());
            },
            _ => {},
        }
    }
    for group in world.genesis.population.groups.values() {
        accounts.extend(group.entity.accounts.keys().cloned());
    }
    let mut fields = vec![Field::Lineage, Field::Place];
    if effects.iter().any(|e| matches!(e, Effect::Death)) {
        fields.push(Field::Alive);
    }
    if effects.iter().any(|e| matches!(e, Effect::Birth { .. })) {
        fields.push(Field::Born);
    }
    fields.extend(accounts.into_iter().map(Field::Account));
    fields
}

/// The count a threshold yields: of members of the selected kind for what
/// a member shows, of sites for what a site shows.
fn threshold(selector: &[Key], q: &Query) -> Option<Probe> {
    Some(match q {
        Query::Account {
            who: Binding::Actor,
            ..
        }
        | Query::Below {
            who: Binding::Actor,
            ..
        }
        | Query::Age { .. }
        | Query::Mood { .. }
        | Query::MoodBelow { .. } => Probe::Members {
            selector: selector.to_vec(),
            query: q.clone(),
        },
        Query::Account {
            who: Binding::Place,
            ..
        }
        | Query::Below {
            who: Binding::Place,
            ..
        }
        | Query::Condition { .. } => Probe::Sites { query: q.clone() },
        _ => return None,
    })
}

pub fn derive(world: &ProbeWorld) -> Vec<Reading> {
    let competitions = &world.genesis.rules.competitions;
    let kinds = || competitions.values().flat_map(|c| &c.kinds);
    let mut out: Vec<Reading> = Vec::new();
    // One reading per threshold of each definition. Thresholds that happen to
    // coincide in one drawn world stay apart, so every world of a domain
    // yields the same readings.
    let mut push = |key: String, source: &str, probe: Probe, starvation: bool| {
        if !out.iter().any(|r| r.source == source && r.probe == probe) {
            out.push(Reading {
                key,
                source: source.into(),
                probe,
                starvation,
            });
        }
    };
    for kind in kinds() {
        let probe = Probe::Alive {
            identity: kind.identity.clone(),
        };
        push(
            format!("alive:{}", kind.identity),
            "competition",
            probe,
            false,
        );
    }
    for p in world.genesis.rules.processes.values() {
        let selector: Vec<Key> = p
            .requires
            .iter()
            .filter_map(|q| match q {
                Query::Trait {
                    who: Binding::Actor,
                    key,
                } => Some(key.clone()),
                _ => None,
            })
            .collect();
        let death = p
            .commitments
            .iter()
            .chain(&p.effects)
            .any(|e| matches!(e, Effect::Death));
        for (j, q) in p.requires.iter().enumerate() {
            let Some(probe) = threshold(&selector, q) else {
                continue;
            };
            push(format!("{}#{j}", p.id), &p.id, probe, death);
        }
    }
    for (id, c) in competitions {
        for kind in &c.kinds {
            let probe = Probe::Members {
                selector: vec![kind.identity.clone()],
                query: kind.hungry.clone(),
            };
            push(format!("{id}#hungry:{}", kind.identity), id, probe, false);
            let probe = Probe::PastBearing {
                identity: kind.identity.clone(),
            };
            push(
                format!("{id}#past-bearing:{}", kind.identity),
                id,
                probe,
                false,
            );
        }
    }
    if let Some(mind) = &world.genesis.rules.mind {
        for (j, need) in mind.needs.iter().enumerate() {
            let selector: Vec<Key> = need.traits.iter().cloned().collect();
            if let Some(probe) = threshold(&selector, &need.query) {
                let key = format!("mind#need:{j}");
                push(key.clone(), &key, probe, false);
            }
        }
    }
    for field in inspected(world) {
        let key = format!("inspect:{}", name(&field));
        push(key, "inspection", Probe::Inspect { field }, false);
    }
    out
}

/// What the rules read of a member or a site; the crowd keys its bins by
/// these and by whatever an inspection shows.
pub fn read_set(world: &ProbeWorld) -> BTreeSet<String> {
    let rules = &world.genesis.rules;
    let needs = rules.mind.iter().flat_map(|m| &m.needs);
    let queries = rules
        .processes
        .values()
        .flat_map(|p| &p.requires)
        .chain(
            rules
                .competitions
                .values()
                .flat_map(|c| c.kinds.iter().map(|k| &k.hungry)),
        )
        .chain(needs.clone().map(|n| &n.query));
    let mut read: BTreeSet<String> = queries
        .map(|q| match q {
            Query::Alive(_) => "alive".into(),
            Query::Trait { key, .. } => format!("trait:{key}"),
            Query::Account { who, key, .. } | Query::Below { who, key, .. } => match who {
                Binding::Place => format!("site:{key}"),
                _ => format!("account:{key}"),
            },
            Query::Age { .. } => "born".into(),
            Query::Condition { key, .. } => format!("site-condition:{key}"),
            Query::Part { .. } => "parts".into(),
            Query::Related { kind } => format!("relation:{kind}"),
            Query::Mood { .. } | Query::MoodBelow { .. } => "mood".into(),
        })
        .collect();
    read.extend(needs.flat_map(|n| &n.traits).map(|t| format!("trait:{t}")));
    // A competition pairs within a site, reads the leaning and sizes up by
    // body, and rations the site's food; its fights read strain against
    // bearing, and the traits that set bearing and the way a break goes.
    for c in rules.competitions.values() {
        read.insert("place".into());
        read.insert(format!("trait:{}", c.contest));
        read.insert(format!("site:{}", c.food));
        read.extend(c.kinds.iter().map(|k| format!("account:{}", k.body)));
    }
    if let Some(m) = rules
        .mind
        .as_ref()
        .filter(|_| !rules.competitions.is_empty())
    {
        read.insert(format!("account:{}", m.strain));
        let traits = m.bearing_traits.keys().chain(m.rise_traits.keys());
        read.extend(traits.map(|t| format!("trait:{t}")));
    }
    read
}

pub fn exact_members(run: &ExactRun) -> Vec<(&Entity, u64)> {
    run.sim
        .state()
        .population
        .groups
        .values()
        .filter(|g| g.entity.kingdom != "kingdom:world")
        .map(|g| (&g.entity, g.count))
        .collect()
}

pub fn crowd_members<'a>(crowd: &'a Crowd) -> Vec<(&'a Entity, u64)> {
    crowd
        .bins
        .iter()
        .filter(|(e, _)| e.kingdom != "kingdom:world")
        .map(|(e, &n)| (e, n))
        .collect()
}

/// Living members, and how many distinct states they hold once zero
/// ledger entries are dropped: members per state is the crowd's density.
pub fn alive_states(members: &[(&Entity, u64)]) -> (u64, usize) {
    let alive: Vec<_> = members.iter().filter(|(e, _)| e.alive).collect();
    let states: BTreeSet<Entity> = alive
        .iter()
        .map(|(e, _)| aggregate::normalize((*e).clone()))
        .collect();
    (alive.iter().map(|(_, n)| n).sum(), states.len())
}

/// One member drawn uniformly: what a single inspection shows.
pub fn inspect<'a>(members: &[(&'a Entity, u64)], seed: u64) -> Option<&'a Entity> {
    let total: u64 = members.iter().map(|m| m.1).sum();
    if total == 0 {
        return None;
    }
    let mut pick = Stream::new(seed).below(total);
    for &(e, n) in members {
        if pick < n {
            return Some(e);
        }
        pick -= n;
    }
    None
}

pub fn evaluate(
    readings: &[Reading],
    world: &ProbeWorld,
    members: &[(&Entity, u64)],
    sites: &BTreeMap<Id, Site>,
    tick: Tick,
    inspected: Option<&Entity>,
) -> Result<Vec<u64>> {
    let lineages: Vec<&Key> = world.genesis.lineages.keys().collect();
    let needs = crate::meaning::needs(&world.genesis.rules);
    let seen = |member, site| Seen {
        member,
        site,
        tick,
        needs,
    };
    let mut values = Vec::with_capacity(readings.len());
    for r in readings {
        values.push(match &r.probe {
            Probe::Alive { identity } => members
                .iter()
                .filter(|(e, _)| e.alive && e.traits.contains(identity))
                .map(|(_, n)| n)
                .sum(),
            Probe::Members { selector, query } => {
                let mut total = 0;
                for &(e, n) in members {
                    let site = sites.get(&e.place).ok_or("a member at an unknown site")?;
                    if selector.iter().all(|t| e.traits.contains(t))
                        && seen(Some(e), site).holds(query)?
                    {
                        total += n;
                    }
                }
                total
            },
            Probe::Sites { query } => {
                let mut total = 0;
                for site in sites.values() {
                    total += u64::from(seen(None, site).holds(query)?);
                }
                total
            },
            Probe::PastBearing { identity } => {
                let mind = world.mind()?;
                let past = |e: &Entity| e.alive && fight::past_bearing(mind, e);
                members
                    .iter()
                    .filter(|(e, _)| e.traits.contains(identity) && past(e))
                    .map(|(_, n)| n)
                    .sum()
            },
            Probe::Inspect { field } => {
                let e = inspected.ok_or("no member to inspect")?;
                match field {
                    Field::Lineage => {
                        lineages.iter().position(|l| **l == e.lineage).unwrap_or(0) as u64
                    },
                    Field::Place => e.place,
                    Field::Alive => u64::from(e.alive),
                    Field::Born => e.born,
                    Field::Account(k) => crate::meaning::value(&e.accounts, k),
                }
            },
        });
    }
    Ok(values)
}
