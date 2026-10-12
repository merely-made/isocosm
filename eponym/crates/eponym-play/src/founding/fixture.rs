// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The versioned native precise-contact pack selected by ruling 825.

use isocosm::rules::{Anchor, Development, Facing, Policy, Recipe, Tagma, Template};
use isocosm::simulation::Genesis;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const GENERATOR_VERSION: u32 = 5;
const ACTOR: &str = "kind:eponym-probe-actor-v1";
const TARGET: &str = "kind:eponym-probe-target-v1";
const STORE: &str = "kind:eponym-probe-store-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFixtureProfile { PreciseCombatV1 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFixtureRecipe { ProbeActorV1, ProbeTargetV1 }

fn template(half_extent: [i32; 3], function: &str, cells: u32) -> Template {
    Template {
        half_extent, cells: BTreeMap::from([(function.into(), cells)]), shape: String::new(),
    }
}

pub(super) fn install(g: &mut Genesis) -> isocosm::Result<()> {
    g.rules.kinds.extend(BTreeMap::from([
        (ACTOR.into(), template([1; 3], "function:intake", 1)),
        (TARGET.into(), template([1, 3, 1], "function:intake", 2)),
        (STORE.into(), template([4, 2, 1], "function:store", 6)),
    ]));
    let d = g.lineages.get_mut(super::SOPHONT).and_then(|l| l.development.as_mut())
        .ok_or("the fixture founding has no compact recipe")?;
    d.lexicon.extend([ACTOR.into(), TARGET.into(), STORE.into()]);
    Ok(())
}

pub(crate) fn development(g: &Genesis, choice: Option<NativeFixtureRecipe>) -> Option<Development> {
    let mut d = g.lineages.get(super::SOPHONT)?.development.clone()?;
    let Some(choice) = choice else { return Some(d) };
    let root = match choice {
        NativeFixtureRecipe::ProbeActorV1 => ACTOR,
        NativeFixtureRecipe::ProbeTargetV1 => TARGET,
    };
    d.recipe = Recipe {
        tagmata: vec![Tagma {
            segments: 1, segment: root.into(), bears: Some(STORE.into()), per_segment: 8,
            parent: None, anchor: Anchor::Base, facing: Facing::Back, socket: Facing::Above,
            variance: None,
        }],
        variance: 0, absence: [0, 1], riff: [0, 1], vary: [0, 1],
    };
    d.policy = Policy { bilateral: false, preferences: BTreeMap::new(), tolerance: 0 };
    Some(d)
}
