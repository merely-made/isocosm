// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The systems sheet (case 1215; lineage-rounds LR4): what a fully grown
//! body of a recipe is and can do, read off that grown body and never
//! summed from the recipe. A pure reading: it develops a body and measures
//! it, running no sim and drawing nothing, so a candidate recipe's sheet is
//! what its body would be.

use super::draft::grown_body;
use crate::{
    Result,
    anatomy,
    process::FeedingMode,
    rules::{Amount, Binding, Development, Effect, Expr, Query, Reading, Rules, default_systems},
    schema::*,
    trophic::{self, Trophic},
};
use isometer_core::BodyDocument;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What a fully grown body of a recipe is and can do (case 1215).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sheet {
    /// Adult mass, in mg.
    pub mass: u128,
    /// Milli-cells a world second, where the world has speed rules.
    pub speed: Option<u64>,
    /// How far the body reaches.
    pub reach: i32,
    /// Eggs a birth lays.
    pub clutch: u32,
    /// How it makes its living, read off its feeding parts.
    pub trophic: Option<Trophic>,
    /// What it does with matter, read from its ports.
    pub feeding: FeedingMode,
    /// Cells by function over the whole body.
    pub functions: BTreeMap<Key, u32>,
    /// The systems the body realises.
    pub systems: BTreeSet<Key>,
    /// Matter a tick its upkeep takes, where the world prices upkeep by the body.
    pub upkeep: Option<u64>,
    /// The comfort band in degrees, where the world has a climate (case 1215;
    /// lineage-rounds D14), shifted by the development's offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comfort: Option<[i64; 2]>,
    /// The least `world:water` the body is comfortable with, where the world
    /// has a climate (lineage-rounds D14): store cells buffer drought.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<i64>,
    /// The light the body needs, on the ecology's sunlight scale, where the
    /// world has a climate: fixing cells set it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<i64>,
}

/// The sheet of a fully grown body of `lineage` under `d`. The adult is
/// built as a birth builds a child, minus its random variation: its body
/// embodied, the development's systems (else the world's defaults) laid on
/// it, and its own tissue filled to what it may hold where upkeep reads it.
///
/// # Errors
/// Fails if `d` cannot develop a body.
pub fn of(
    rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>,
    lineage: &str,
    d: &Development,
) -> Result<Sheet> {
    let body = grown_body(rules, d)?;
    let b = rules.body();
    let mass = body
        .doc
        .parts
        .iter()
        .map(|p| u128::from(anatomy::ceiling(p.half_extent, b)))
        .sum();
    let held = lineages.get(lineage);
    let mut e = Entity {
        drawn: None,
        lineage: lineage.into(),
        kingdom: held.map_or_else(String::new, |l| l.kingdom.clone()),
        scale: "scale:meso".into(),
        provenance: Provenance::Born(lineage.into()),
        method: Method::Reactive,
        place: 0,
        arrived: 0,
        visits: vec![],
        journey: None,
        born: 0,
        alive: true,
        body_revision: 1,
        body: None,
        parts: BTreeMap::new(),
        traits: held.map(|l| l.traits.clone()).unwrap_or_default(),
        accounts: Ledger::new(),
        skills: BTreeMap::new(),
        tenets: BTreeMap::new(),
        disposition: [0; 5],
        soma: d.recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
        systems: BTreeMap::new(),
        varied: vec![],
        patch: None,
        authored: None,
        settled: None,
    };
    e.embody(body);
    let carried = match d.systems.is_empty() {
        true => default_systems(),
        false => d.systems.clone(),
    };
    e.systems = carried.clone();
    crate::systems::inherit(&mut e);
    let mut functions = BTreeMap::<Key, u32>::new();
    for p in e.parts.values() {
        for (f, n) in &p.cells {
            *functions.entry(f.clone()).or_default() += n;
        }
    }
    let doc = e.body.as_ref().expect("embodied above");
    let [store, fix, cells] = crate::climate::tally(&functions);
    let needs = rules.climate.as_ref().map(|c| {
        (crate::climate::thirst(c, store, cells), crate::climate::light(c, fix, cells))
    });
    Ok(Sheet {
        mass,
        speed: rules.speed.as_ref().map(|s| s.of(doc)),
        reach: reach(doc),
        clutch: d.clutch,
        trophic: trophic::of(&e),
        feeding: crate::mosaic::ports::feeding(&e),
        functions,
        systems: carried
            .iter()
            .filter(|(_, s)| crate::systems::realizes(&e, s))
            .map(|(k, _)| k.clone())
            .collect(),
        upkeep: upkeep(rules, lineages, &mut e),
        comfort: rules
            .climate
            .as_ref()
            .map(|c| crate::climate::band(c, mass, i64::from(d.comfort))),
        water: needs.map(|n| n.0),
        light: needs.map(|n| n.1),
    })
}

fn reach(doc: &BodyDocument) -> i32 {
    use crate::process::BodyProcesses;
    doc.reach()
}

/// What the world's body-priced upkeep process for `e`'s lineage charges a
/// fully grown member a tick: its amount evaluated on the adult, filled
/// with its own tissue as a grown member holds it. None where the world has
/// no such process.
fn upkeep(rules: &Rules, lineages: &BTreeMap<Key, Lineage>, e: &mut Entity) -> Option<u64> {
    let process = rules.processes.values().find(|p| {
        p.id.starts_with("body:upkeep-")
            && p.requires.iter().any(|q| {
                matches!(q, Query::Trait { who: Binding::Actor, key } if e.traits.contains(key))
            })
    })?;
    let expr = process.effects.iter().find_map(|f| match f {
        Effect::Spend { amount: Amount::Computed(x), .. } => Some(x),
        _ => None,
    })?;
    if let Some(tissue) = anatomy::own(rules, &e.lineage).tissue {
        let room = anatomy::room(e, rules, &tissue);
        if let Some(filled) = anatomy::give(e, rules, &tissue, room) {
            filled.ok()?;
        }
    }
    let entity: &Entity = e;
    let mut read = |r: &Reading| -> Result<i64> {
        match r {
            Reading::Account { key, .. } => {
                i64::try_from(anatomy::held(entity, rules, key)).map_err(|x| x.to_string())
            },
            r if r.who() == Binding::Actor && !matches!(r, Reading::Kept { .. }) => {
                Ok(crate::meaning::body_reading(entity, r, rules, Some(lineages)))
            },
            _ => Err("upkeep reads only the body".into()),
        }
    };
    let mut parts = |who: Binding| {
        if who != Binding::Actor {
            return Err("upkeep sums only the body's parts".to_string());
        }
        let living = entity.living().map(|(id, p)| (entity.extent(id), p.clone()));
        Ok((living.collect(), rules.body()))
    };
    let value = Expr::eval_in(expr, &mut read, &mut |_, _| Err("upkeep draws nothing".into()), &mut parts)
        .ok()?;
    u64::try_from(value.max(0)).ok()
}
