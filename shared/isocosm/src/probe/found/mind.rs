// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The mind's part of the probe's domain. What the rulings leave open is
//! drawn per world, from stated ranges: how far each need lowers mood (ruling
//! 227), how low a mood counts as low, how fast strain builds while it is
//! low and bleeds while it is not (ruling 159), the bearing (ruling 164) and
//! the chance a break goes up (ruling 163), each shifted by a member's
//! lineage and leaning.

use super::super::{Mind, Need};
use crate::{generate::process, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The account each member keeps its strain in.
pub const STRAIN: &str = "mind:strain";

/// Inclusive ranges, drawn per world. Mood weights count down from a
/// content zero: a member's mood is minus the weights of its needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindFounding {
    /// Mood lost while wanting a contested thing, drawn for each, while at
    /// low reserve, and while starving, one tick from death.
    pub hungry: [u64; 2],
    pub low: [u64; 2],
    pub starving: [u64; 2],
    /// A mood below minus this is low.
    pub low_mood: [u64; 2],
    /// Strain gained per tick while mood is low, and bled per tick while it
    /// is not.
    pub build: [u64; 2],
    pub bleed: [u64; 2],
    pub bearing: [i64; 2],
    pub lineage_bearing: [i64; 2],
    pub leaning_bearing: [i64; 2],
    /// Per mille.
    pub rise: [i64; 2],
    pub lineage_rise: [i64; 2],
    pub leaning_rise: [i64; 2],
    /// Per mille per point of mood: the moment's pull on a break.
    pub stake: [i64; 2],
}

impl Default for MindFounding {
    fn default() -> Self {
        Self {
            hungry: [1, 3],
            low: [1, 3],
            starving: [2, 5],
            low_mood: [1, 4],
            build: [1, 3],
            bleed: [1, 3],
            bearing: [4, 16],
            lineage_bearing: [-3, 3],
            leaning_bearing: [-2, 2],
            rise: [250, 750],
            lineage_rise: [-150, 150],
            leaning_rise: [-100, 100],
            stake: [-60, 60],
        }
    }
}

/// One competing kind as the mind reads it: its reserve, and what it wants
/// of each contested thing.
pub(super) struct Kind<'a> {
    pub identity: &'a str,
    pub body: &'a str,
    pub wants: Vec<Query>,
}

fn below(key: &str, amount: u64) -> Query {
    Query::Below {
        who: Binding::Actor,
        key: key.into(),
        amount,
    }
}

impl MindFounding {
    pub(super) fn valid(&self) -> bool {
        let plain = [
            self.hungry,
            self.low,
            self.starving,
            self.low_mood,
            self.build,
            self.bleed,
        ];
        let signed = [
            self.bearing,
            self.lineage_bearing,
            self.leaning_bearing,
            self.rise,
            self.lineage_rise,
            self.leaning_rise,
            self.stake,
        ];
        plain.iter().all(|r| r[0] <= r[1]) && signed.iter().all(|r| r[0] <= r[1])
    }

    /// The world's mind, and the two processes that keep strain: one builds
    /// it while mood is low, one bleeds it while mood is not.
    pub(super) fn draw(&self, seed: u64, kinds: &[Kind], hunger: u64) -> (Mind, [Process; 2]) {
        let pick = |domain: &str, i: u64, r: [u64; 2]| {
            r[0] + crate::draw(seed, domain, &[i]) % (r[1] - r[0] + 1)
        };
        let signed = |domain: &str, i: u64, r: [i64; 2]| {
            let span = r[1].abs_diff(r[0]) + 1;
            r[0].saturating_add_unsigned(crate::draw(seed, domain, &[i]) % span)
        };
        let weight = |domain: &str, i: u64, r: [u64; 2]| -(pick(domain, i, r) as i64);
        let (low, starving) = (
            weight("mind-low", 0, self.low),
            weight("mind-starving", 0, self.starving),
        );
        // Low reserve lies between starving and hunger.
        let reserve = pick("mind-reserve", 0, [2, hunger.max(2)]);
        let mut needs = Vec::new();
        let mut bearing_traits = BTreeMap::new();
        let mut rise_traits = BTreeMap::new();
        for (i, k) in kinds.iter().enumerate() {
            let own = BTreeSet::from([k.identity.to_string()]);
            let wanting = k.wants.iter().enumerate().map(|(j, want)| {
                let w = weight("mind-want", j as u64, self.hungry);
                (want.clone(), w)
            });
            let reserves = [(below(k.body, reserve), low), (below(k.body, 2), starving)];
            for (query, weight) in wanting.collect::<Vec<_>>().into_iter().chain(reserves) {
                needs.push(Need {
                    traits: own.clone(),
                    query,
                    weight,
                });
            }
            let i = i as u64;
            let lineage = k.identity.to_string();
            bearing_traits.insert(
                lineage.clone(),
                signed("mind-lineage-bearing", i, self.lineage_bearing),
            );
            rise_traits.insert(lineage, signed("mind-lineage-rise", i, self.lineage_rise));
        }
        for (j, leaning) in ["leaning:contest", "leaning:scramble"].iter().enumerate() {
            let j = j as u64;
            let shift = signed("mind-leaning-bearing", j, self.leaning_bearing);
            bearing_traits.insert(leaning.to_string(), shift);
            let shift = signed("mind-leaning-rise", j, self.leaning_rise);
            rise_traits.insert(leaning.to_string(), shift);
        }
        let mind = Mind {
            strain: STRAIN.into(),
            needs,
            bearing: signed("mind-bearing", 0, self.bearing),
            bearing_traits,
            rise: signed("mind-rise", 0, self.rise),
            rise_traits,
            stake: signed("mind-stake", 0, self.stake),
        };
        let low_mood = -(pick("mind-low-mood", 0, self.low_mood) as i64);
        let mut build = process(
            "mind:strain",
            Shape::Transition,
            vec![Effect::Transform {
                who: Binding::Actor,
                take: BTreeMap::new(),
                give: BTreeMap::from([(STRAIN.into(), pick("mind-build", 0, self.build))]),
            }],
        );
        build.requires.push(Query::MoodBelow { amount: low_mood });
        let mut bleed = process(
            "mind:relief",
            Shape::Transition,
            vec![Effect::Ease {
                who: Binding::Actor,
                key: STRAIN.into(),
                amount: pick("mind-bleed", 0, self.bleed),
            }],
        );
        bleed.requires.extend([
            Query::Mood { at_least: low_mood },
            Query::Account {
                who: Binding::Actor,
                key: STRAIN.into(),
                at_least: 1,
            },
        ]);
        // Mood is read once the tick's upkeep and starvation are settled.
        for p in [&mut build, &mut bleed] {
            p.period = Some(1);
            p.priority = 2;
        }
        (mind, [build, bleed])
    }
}
