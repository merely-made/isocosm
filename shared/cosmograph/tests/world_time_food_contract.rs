// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Newly founded world-time diets use declared native ports as their food
//! authority, including animal food from another predator's lineage.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    bodied::Bodies,
    ecology::{Rates, Web},
    mosaic::ports::{self, admission},
    population::Population,
    process::NisKind,
    rules::{Binding, Causation, Effect, FeedingSources, Query},
    schema::{Entity, Id, Ledger, Part, PartId},
    simulation::{Genesis, Receipt},
};

#[path = "world_time_food_contract/compatibility.rs"]
mod compatibility;
#[path = "world_time_food_contract/counts.rs"]
mod counts;
#[path = "world_time_food_contract/mixed.rs"]
mod mixed;
#[path = "world_time_food_contract/native.rs"]
mod native;
#[path = "world_time_food_contract/partition.rs"]
mod partition;
#[path = "world_time_food_contract/scope.rs"]
mod scope;

const DETRITUS: &str = "world:detritus";
const INGESTA: &str = "world:ingesta";

fn founding() -> Genesis {
    Founding {
        seed: 9,
        sites: 1,
        population: 60,
        cohort_size: 1,
        lineages: 10,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        web: Web::Diets,
        bodies: Some(Bodies::default()),
        ..Founding::default()
    }
    .generate()
    .expect("the ordinary ten-lineage bodied world-time founding")
}

fn founder(g: &Genesis, index: u32) -> &Entity {
    let line = format!("lineage:{index}");
    g.population
        .groups
        .values()
        .find(|group| group.entity.lineage == line)
        .expect("the founding floor supplies every lineage")
        .entity
        .as_ref()
}

fn intake(g: &Genesis, index: u32) -> &Part {
    let feed = &g.rules.processes[&format!("ecology:feed-{index}")];
    let feeding = feed
        .port_feeding
        .as_ref()
        .expect("a native animal feed declares its intake organ");
    let (_, part) = founder(g, index)
        .living()
        .find(|(_, part)| part.port.is_some() && part.functions.contains(&feeding.function))
        .expect("the founder carries its declared intake organ");
    assert!(
        ports::active(part).is_some(),
        "the intake has living support"
    );
    part
}

fn offered(g: &Genesis, eater: u32, prey: u32) -> Ledger {
    let feeding = g.rules.processes[&format!("ecology:feed-{eater}")]
        .port_feeding
        .as_ref()
        .unwrap();
    let sources = match &feeding.sources {
        FeedingSources::All => admission::Sources::All,
        FeedingSources::Only(lines) => admission::Sources::Only(lines),
    };
    let prey = founder(g, prey);
    admission::offered(
        intake(g, eater),
        sources,
        &anatomy::books(prey),
        prey.alive,
        &g.rules,
        &g.lineages,
    )
    .expect("native admission resolves the actual source accounts")
}

fn member_id(g: &Genesis, index: u32) -> Id {
    *g.population
        .groups
        .iter()
        .find(|(_, group)| group.entity.lineage == format!("lineage:{index}"))
        .unwrap()
        .0
}

fn controlled(eater_line: u32, prey_line: u32) -> (Genesis, Id, Id) {
    let mut g = founding();
    let (eater, prey) = (member_id(&g, eater_line), member_id(&g, prey_line));
    g.population
        .groups
        .retain(|id, _| *id == eater || *id == prey);
    for id in [eater, prey] {
        g.population.lift(id).unwrap().place = 0;
    }
    let feed = format!("ecology:feed-{eater_line}");
    let birth = format!("ecology:birth-{eater_line}");
    g.rules
        .processes
        .retain(|id, _| id == &feed || id == &birth);
    g.rules.processes.get_mut(&feed).unwrap().period = Some(1);
    g.rules.processes.get_mut(&birth).unwrap().period = None;
    (g, eater, prey)
}

fn act(s: &mut Session, actor: Id, target: Option<Id>, process: &str) -> Receipt {
    serde_json::from_str(
        &s.command(Command::Act {
            actor,
            target,
            process: process.into(),
            cause: None,
        })
        .unwrap(),
    )
    .unwrap()
}

fn books(s: &Session, id: Id) -> Ledger {
    anatomy::books(s.sim.state().population.get(id).unwrap())
}

fn held(s: &Session, id: Id, key: &str) -> u64 {
    books(s, id).get(key).copied().unwrap_or(0)
}

fn site_held(s: &Session, key: &str) -> u64 {
    s.sim.state().sites[&0]
        .accounts
        .get(key)
        .copied()
        .unwrap_or(0)
}

fn replay(s: &Session) {
    let saved = s.save();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load(saved.clone(), mode).unwrap();
        assert_eq!(loaded.save(), saved);
        assert_eq!(loaded.sim.matter(), s.sim.matter());
    }
}

fn put_tissue(e: &mut Entity, key: &str, amount: u64) {
    e.accounts.remove(key);
    let parts: Vec<_> = e.parts.keys().copied().collect();
    for id in parts {
        e.part_mut(&id).unwrap().matter.remove(key);
    }
    let root = e.body.as_ref().unwrap().root;
    e.part_mut(&root).unwrap().matter.insert(key.into(), amount);
}

fn intake_id(e: &Entity, function: &str) -> PartId {
    e.living()
        .find(|(_, part)| part.port.is_some() && part.functions.contains(function))
        .expect("an actual native intake")
        .0
}

fn put_ingesta(g: &mut Genesis, id: Id, line: u32, amount: u64) {
    let function = g.rules.processes[&format!("ecology:feed-{line}")]
        .port_feeding
        .as_ref()
        .unwrap()
        .function
        .clone();
    let e = g.population.lift(id).unwrap();
    let part = intake_id(e, &function);
    e.part_mut(&part)
        .unwrap()
        .matter
        .insert(INGESTA.into(), amount);
}

fn stripped_plant(g: &mut Genesis, id: Id, amount: u64) -> PartId {
    let rule = g.rules.clone();
    let e = g.population.lift(id).unwrap();
    let a = anatomy::root_store::allocation(e, &rule, "matter:0-0").unwrap();
    let chosen = *a
        .exposed
        .iter()
        .find(|(_, share)| **share >= amount)
        .unwrap()
        .0;
    let parts: Vec<_> = e.parts.keys().copied().collect();
    for part in parts {
        e.part_mut(&part).unwrap().matter.remove("matter:0-0");
    }
    e.part_mut(&a.store)
        .unwrap()
        .matter
        .insert("matter:0-0".into(), 20);
    if amount > 0 {
        e.part_mut(&chosen)
            .unwrap()
            .matter
            .insert("matter:0-0".into(), amount);
    }
    chosen
}

fn auxiliary(g: &mut Genesis, feed: &str, key: &str, effects: Vec<Effect>) {
    let mut p = g.rules.processes[feed].clone();
    p.id = key.into();
    p.causation = Causation::Transition;
    p.requires = vec![];
    p.commitments.clear();
    p.effects = effects;
    p.risk = None;
    p.target = None;
    p.port_feeding = None;
    p.period = None;
    p.note = false;
    p.need_account = None;
    p.goal_directives.clear();
    g.rules.processes.insert(key.into(), p);
}

fn food_gate(g: &mut Genesis, feed: &str, at_least: u64, per_members: u64) -> String {
    let key = "test:food-gate";
    g.rules.traits.insert("test:enough-food".into());
    auxiliary(
        g,
        feed,
        key,
        vec![Effect::Trait {
            who: Binding::Actor,
            key: "test:enough-food".into(),
            present: true,
        }],
    );
    g.rules.processes.get_mut(key).unwrap().requires = vec![
        Query::Alive(Binding::Actor),
        Query::FoodPerHead {
            feed: feed.into(),
            at_least,
            per_members,
        },
    ];
    key.into()
}

#[test]
fn fresh_world_time_diets_ports_admit_predators_and_mixed_foods() {
    let g = founding();
    // Diets repeats producer, grazer, decomposer, predator and omnivore.
    // These are actual founders and declared account sources, not new
    // hand-authored ports that bypass the ordinary founding path.
    for (eater, other_predator) in [(3, 8), (8, 3)] {
        let port = ports::active(intake(&g, eater)).unwrap();
        assert!(
            port.admits_live(NisKind::Consumer),
            "world-time predator {eater} must admit living animal matter"
        );
        assert!(
            !port.admits_live(NisKind::Producer),
            "world-time predator {eater} must not admit living producer matter"
        );
        assert!(!port.admits_deadstock(), "this predator hunts living food");
        let animal = offered(&g, eater, other_predator);
        assert!(
            animal
                .get(&format!("matter:{other_predator}-0"))
                .copied()
                .unwrap_or(0)
                > 0,
            "the feed's source selector must admit the other predator's actual tissue"
        );
        assert!(
            offered(&g, eater, 0).is_empty(),
            "producer tissue is outside this animal-eating port"
        );
    }
    for eater in [4, 9] {
        let port = ports::active(intake(&g, eater)).unwrap();
        assert!(port.admits_live(NisKind::Producer));
        assert!(port.admits_live(NisKind::Consumer));
        assert!(!offered(&g, eater, 0).is_empty());
        assert!(!offered(&g, eater, 3).is_empty());
    }
}
