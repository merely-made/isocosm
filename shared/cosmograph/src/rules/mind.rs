// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the core reads of minds: needs and the mind (rulings 158, 159, 163,
//! 164 and 227).

use super::{Binding, Causation, Effect, Process, Query};
use crate::{generate::process, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A need, as the core has needs now (ruling 227): members carrying every
/// one of `traits` for whom `query` holds have their mood moved by `weight`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Need {
    pub traits: BTreeSet<Key>,
    pub query: Query,
    pub weight: i64,
}

/// What the core reads of a mind now. Mood is read from needs and never
/// kept; strain is kept in its account (rulings 158, 159 and 227). A mind
/// bears strain up to its bearing, set by its traits (ruling 164); past it,
/// it breaks, up with the chance its traits and the moment give (ruling 163).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mind {
    pub strain: Key,
    pub needs: Vec<Need>,
    pub bearing: i64,
    pub bearing_traits: BTreeMap<Key, i64>,
    /// Per mille chance that a break goes up.
    pub rise: i64,
    pub rise_traits: BTreeMap<Key, i64>,
    /// The moment: per mille added to the rise for each point of mood.
    pub stake: i64,
    /// Strain kept as a rate and worked out when read (`crate::strain`),
    /// where every need reads only the actor's own body; absent where the
    /// world keeps strain by its two processes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<Rate>,
}

/// Strain built a tick while mood lies below minus `low_mood`, and bled a
/// tick while it does not (ruling 159; the execution plan's E7, D16).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rate {
    pub low_mood: u64,
    pub build: u64,
    pub bleed: u64,
}

impl Mind {
    /// Whether every need reads only the actor's own body, so that mood
    /// changes only when the body is written.
    pub fn bodily(&self) -> bool {
        self.needs.iter().all(|n| {
            matches!(
                n.query,
                Query::Below { who: Binding::Actor, .. }
                    | Query::Account { who: Binding::Actor, .. }
                    | Query::Wounded { who: Binding::Actor, .. }
            )
        })
    }

    /// Strain's per-world ranges, inclusive (ruling 159), each drawn from the
    /// world's seed: minus the mood a low mood lies below, and the strain
    /// built and bled a tick. The probe's minds and the played lineages share
    /// them.
    pub const LOW_MOOD: [u64; 2] = [1, 4];
    pub const BUILD: [u64; 2] = [1, 3];
    pub const BLEED: [u64; 2] = [1, 3];

    /// The two processes that keep strain (ruling 159): one builds `build` a
    /// tick while mood is below minus `low_mood`, one bleeds `bleed` while it
    /// is not. Mood is read once the tick's upkeep and starvation are settled.
    pub fn strain_processes(&self, low_mood: u64, build: u64, bleed: u64) -> [Process; 2] {
        let low = -(low_mood as i64);
        let mut builds = process(
            "mind:strain",
            Causation::Transition,
            vec![Effect::Transform {
                who: Binding::Actor,
                take: BTreeMap::new(),
                give: BTreeMap::from([(self.strain.clone(), build)]),
                conversion: None,
            }],
        );
        builds.requires.push(Query::MoodBelow { amount: low });
        let mut bleeds = process(
            "mind:relief",
            Causation::Transition,
            vec![Effect::Ease {
                who: Binding::Actor,
                key: self.strain.clone(),
                amount: bleed.into(),
            }],
        );
        bleeds.requires.extend([
            Query::Mood { at_least: low },
            Query::Account {
                who: Binding::Actor,
                key: self.strain.clone(),
                at_least: 1,
            },
        ]);
        for p in [&mut builds, &mut bleeds] {
            p.period = Some(1);
            p.priority = 2;
        }
        [builds, bleeds]
    }
}
