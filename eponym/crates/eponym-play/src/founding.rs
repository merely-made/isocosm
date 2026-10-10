// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's world as a native founding (wing ruling 755): a grid of sites
//! with terrain, and Eponym's pack laid over the rules. A sophont's hunger
//! reads its matter reserve and its fatigue its `Energy` (767); eating, rest
//! and exertion are processes; deeds, knowing and names keep their notes;
//! items are inert bodies of their own lineages, food holding matter a
//! sophont eats.

use isocosm::{
    Founding, Result,
    map::{Grid, Layout},
    rules::{AccountKind, Amount, Binding, Causation, Effect, Mind, Need, Process, Query, Target},
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

pub const SOPHONT: &str = "lineage:eponym";
pub const TISSUE: &str = "eponym:tissue";
pub const RESERVE: &str = "eponym:reserve";
pub const ENERGY: &str = "sim:energy";
/// What a meal of food holds, in its own account.
pub const FOOD: &str = "item:food";
pub const DRESSING: &str = "item:dressing";
pub const SCRAP: &str = "item:scrap";
/// A sophont's needs are felt below these.
pub const FULL: u64 = 100;
/// What one food holds and one rest restores.
pub const MEAL: u64 = 40;
pub const REST: u64 = 50;

pub const EAT: &str = "eponym:eat";
pub const RESTS: &str = "eponym:rest";
pub const HUNGER: &str = "eponym:hunger";
pub const FATIGUE: &str = "eponym:fatigue";
pub const TAKE: &str = "eponym:take";
pub const DROP: &str = "eponym:drop";
pub const SPENT: &str = "eponym:spent";
/// What a sophont carries, worn or not.
pub const CARRIES: &str = "item:carried";
/// An inherited site's kind, a note about the site.
pub const SITE_FACT: &str = "eponym:site";

fn set(values: &[&str]) -> BTreeSet<Key> {
    values.iter().map(|s| s.to_string()).collect()
}

fn process(id: &str, causation: Causation, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: set(&["sim:matter", "sim:nonnegative"]),
        note: false,
    }
}

fn anyone(among: &[&str]) -> Option<Target> {
    Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: None,
        among: set(among),
        weighted: false,
    })
}

/// The world `seed` founds: `side` sites a side, each `extent` base units
/// across, Eponym's pack laid over its rules.
pub fn genesis(seed: u64, side: u32, extent: u64) -> Result<Genesis> {
    let grid = Grid {
        shape: "shape:plane".into(),
        width: side,
        height: side,
        side: extent,
        elevation: [0, extent as i64 / 4],
        relief: [0, extent as i64 / 16],
        sea_per_mille: 0,
    };
    let founding = Founding {
        seed,
        sites: side * side,
        population: 1,
        lineages: 1,
        cohort_size: 1,
        map: Some(Layout::Grid(grid)),
        ..Founding::default()
    };
    let mut g = founding.generate()?;
    let lineage = |kingdom: &str| Lineage {
        parent: None,
        revision: 1,
        traits: BTreeSet::new(),
        kingdom: kingdom.into(),
        development: None,
    };
    g.lineages.insert(SOPHONT.into(), lineage("kingdom:fauna"));
    for item in [FOOD, DRESSING, SCRAP] {
        g.lineages.insert(item.into(), lineage("kingdom:made"));
    }
    let matter = |lineage: &str, reserve| AccountKind::Matter {
        lineage: lineage.into(),
        reserve,
        provision: false,
    };
    let r = &mut g.rules;
    r.accounts.insert(TISSUE.into(), matter(SOPHONT, false));
    r.accounts.insert(RESERVE.into(), matter(SOPHONT, true));
    for item in [FOOD, DRESSING, SCRAP] {
        r.accounts.insert(item.into(), matter(item, false));
    }
    r.relations.insert(CARRIES.into());
    r.relations.insert(isocosm::social::TENANT.into());
    r.note_kinds.extend(set(&[isocosm::arrival::NAME, SITE_FACT]));
    r.note_kinds.extend(isocosm::knowing::NOTE_KINDS.iter().map(|k| k.to_string()));
    r.processes.extend(isocosm::social::processes());
    r.processes.extend(processes());
    r.mind = Some(Mind {
        strain: "sim:strain".into(),
        needs: [RESERVE, ENERGY].map(need).to_vec(),
        bearing: 100,
        bearing_traits: BTreeMap::new(),
        rise: 0,
        rise_traits: BTreeMap::new(),
        stake: 0,
    });
    r.accounts.insert("sim:strain".into(), AccountKind::Strain);
    // A sophont carves through its own ledger (412, 739).
    for m in g.world.materials.iter_mut().filter(|m| m.key == "world:soil" || m.key == "world:rock") {
        m.density = Some(1);
        m.account = Some("world:soil".into());
    }
    Ok(g)
}

fn need(account: &str) -> Need {
    Need {
        traits: BTreeSet::new(),
        query: Query::Below {
            who: Binding::Actor,
            key: account.into(),
            amount: FULL,
        },
        weight: -1,
    }
}

/// Eponym's processes: eating, rest, a unit of each exertion, carrying and
/// an item spent.
fn processes() -> BTreeMap<Key, Process> {
    let mut eat = process(EAT, Causation::Choice, vec![Effect::Eat {
        from: Binding::Target,
        amount: Amount::Fixed(MEAL),
        into: RESERVE.into(),
        of: vec![FOOD.into()],
        whole: false,
    }]);
    eat.target = anyone(&[FOOD]);
    let rest = process(RESTS, Causation::Choice, vec![Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([(ENERGY.into(), REST)]),
        conversion: None,
    }]);
    let hunger = process(HUNGER, Causation::Transition, vec![Effect::Spend {
        from: vec![RESERVE.into()],
        to: Binding::Place,
        amount: Amount::Fixed(1),
        into: None,
    }]);
    let fatigue = process(FATIGUE, Causation::Transition, vec![Effect::Ease {
        who: Binding::Actor,
        key: ENERGY.into(),
        amount: Amount::Fixed(1),
    }]);
    let mut take = process(TAKE, Causation::Choice, vec![Effect::Relate {
        kind: CARRIES.into(),
        present: true,
    }]);
    take.target = anyone(&[FOOD, DRESSING, SCRAP]);
    let mut drop = process(DROP, Causation::Choice, vec![Effect::Relate {
        kind: CARRIES.into(),
        present: false,
    }]);
    drop.target = anyone(&[FOOD, DRESSING, SCRAP]);
    let spent = process(SPENT, Causation::Transition, vec![Effect::Death]);
    [eat, rest, hunger, fatigue, take, drop, spent]
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect()
}
