// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A generated world's played lineage (rulings 682 and 683): its members
//! deliberate, the world's mind gives it a need, and directing's rules give
//! each trophic level its slot.

use crate::{Founding, Result, rules::*, schema::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The lineage a founding gives a mind, by index, and how many sites'
/// worth of each trophic level, as founded, a region's slot holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Played {
    pub lineage: u32,
    pub region_sites: u32,
}

/// The trophic levels generated ecologies name (ruling 39).
pub const LEVELS: [&str; 3] = ["life:producer", "life:consumer", "life:decomposer"];

pub(crate) fn found(f: &Founding, g: &mut Genesis, played: Played) -> Result<()> {
    if played.lineage >= f.lineages || played.region_sites == 0 {
        return Err("the played lineage or its region is outside the founding".into());
    }
    let i = played.lineage;
    let lineage = format!("lineage:{i}");
    let body = format!("matter:{i}-0");
    for group in g.population.groups.values_mut() {
        if group.entity.lineage == lineage {
            group.entity.method = Method::Deliberative;
        }
    }
    let rules = &mut g.rules;
    let strain: Key = "sim:strain".into();
    rules
        .accounts
        .entry(strain.clone())
        .or_insert(AccountKind::Strain);
    // Hungry below what a body is founded with.
    let hunger = Need {
        traits: BTreeSet::from([format!("ability:cycle-{i}")]),
        query: Query::Below {
            who: Binding::Actor,
            key: body,
            amount: f.stock_min + 2,
        },
        weight: -10,
    };
    let mind = rules.mind.get_or_insert_with(|| Mind {
        strain,
        needs: vec![],
        bearing: 100,
        bearing_traits: BTreeMap::new(),
        rise: 0,
        rise_traits: BTreeMap::new(),
        stake: 0,
    });
    mind.needs.push(hunger);
    let mut slots = BTreeMap::new();
    for level in LEVELS.into_iter().filter(|l| rules.traits.contains(*l)) {
        let founded: u128 = g
            .population
            .groups
            .values()
            .filter(|c| c.entity.traits.contains(level))
            .map(|c| {
                crate::meaning::mass(&crate::anatomy::books(&c.entity), rules) * u128::from(c.count)
            })
            .sum();
        let slot = (founded * u128::from(played.region_sites)).div_ceil(u128::from(f.sites.max(1)));
        slots.insert(level.into(), slot.max(1));
    }
    rules.directing = Some(Directing {
        slots,
        ..Directing::default()
    });
    Ok(())
}
