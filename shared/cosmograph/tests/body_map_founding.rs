// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{
    Execution, Founding, Session,
    ecology::Ward,
    history::Command,
    rules::{Rule, unfold},
    schema::{Entity, Id},
};
use std::collections::{BTreeMap, BTreeSet};

const PLANT: &str = "lineage:0";
const GRAZER: &str = "lineage:1";
const REGION: &str = "region:defence";
const TISSUE: &str = "function:secrete";
const FOOD: &str = "ecology:feed-1";

fn founding() -> Founding {
    toml::from_str(include_str!("../examples/body-map.toml")).unwrap()
}

fn plain(authored: &Founding) -> Founding {
    let mut plain = authored.clone();
    plain
        .body_maps
        .get_mut(PLANT)
        .unwrap()
        .regions
        .insert(REGION.into(), "kind:frond".into());
    plain
}

fn glands(body: &Entity) -> u32 {
    body.parts
        .values()
        .filter(|p| p.cells.contains_key(TISSUE))
        .count() as u32
}

fn member(session: &Session, lineage: &str, shape: impl Fn(&Entity) -> bool) -> Id {
    session
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, group)| {
            group.entity.lineage == lineage && group.entity.alive && shape(&group.entity)
        })
        .map(|(&id, _)| id)
        .expect("a matching founder on the site")
}

#[test]
fn the_saved_grammar_grows_founder_variants_with_the_same_geometry_as_the_plain_edit() {
    let authored = founding();
    let toxic = authored.generate().unwrap();
    let plain = plain(&authored).generate().unwrap();
    let mut longer = authored.clone();
    longer.body_maps.get_mut(PLANT).unwrap().grammar.rules[0] = Rule::Repeat { tagma: 1, by: 2 };
    let longer = longer.generate().unwrap();
    assert_eq!(toxic.founding.as_ref(), Some(&authored));
    assert_eq!(toxic, authored.generate().unwrap());
    let d = toxic.lineages[PLANT].development.as_ref().unwrap();
    assert_eq!(
        unfold(d.grammar.as_ref().unwrap()).unwrap(),
        d.recipe.tagmata
    );
    assert_eq!(d.recipe.tagmata[1].segments, 2);
    assert!(d.lexicon.contains("kind:catalog-cyanide-leaf"));
    let mut sizes = BTreeMap::new();
    for (&id, group) in &toxic.population.groups {
        let body = &group.entity;
        let control = &plain.population.groups[&id].entity;
        assert_eq!(body.body, control.body);
        if body.lineage == PLANT {
            assert_eq!(glands(body), u32::from(body.soma[1]));
            assert_eq!(glands(control), 0);
            assert_eq!(
                longer.population.groups[&id].entity.soma[1],
                body.soma[1] + 1
            );
            *sizes.entry(body.soma[1]).or_insert(0u64) += group.count;
        } else {
            assert_eq!(body, control);
        }
    }
    assert_eq!(sizes.keys().copied().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(
        Session::new(toxic, Execution::Grouped)
            .unwrap()
            .sim
            .matter(),
        Session::new(plain, Execution::Grouped)
            .unwrap()
            .sim
            .matter()
    );
    println!(
        "seed {}, founded producers by number of repeated regions: {sizes:?}",
        authored.seed
    );
}

fn bite(session: &mut Session, target: Id) -> (u64, u32) {
    let actor = member(session, GRAZER, |_| true);
    let lost = |session: &Session| {
        session
            .sim
            .state()
            .population
            .get(actor)
            .unwrap()
            .parts
            .values()
            .map(|part| part.lost.len())
            .sum::<usize>()
    };
    let before = lost(session);
    let result = session
        .command_with_flows(Command::Act {
            actor,
            target: Some(target),
            process: FOOD.into(),
            cause: None,
        })
        .unwrap();
    let matter = result
        .flows
        .iter()
        .filter(|f| {
            f.by_process(FOOD) && f.from.0.body() == Some(target) && f.to.0.body() == Some(actor)
        })
        .map(|f| f.amount * f.count)
        .sum();
    assert!(
        matter > 0,
        "the same native meal should run: {}",
        result.result
    );
    let wound = session
        .sim
        .harms()
        .iter()
        .filter(|h| h.prey == target && h.ward == Some(Ward::Toxin))
        .map(|h| h.cells)
        .sum();
    assert_eq!(lost(session) - before, wound as usize);
    (matter, wound)
}

#[test]
fn the_native_bite_reads_mapped_defence_and_the_number_of_grown_regions() {
    let authored = founding();
    let toxic = Session::new(authored.generate().unwrap(), Execution::Grouped).unwrap();
    let plain = Session::new(plain(&authored).generate().unwrap(), Execution::Grouped).unwrap();
    for segments in [1, 2, 3] {
        let id = member(&toxic, PLANT, |e| e.soma[1] == segments);
        let (mut a, mut b) = (plain.clone(), toxic.clone());
        let (plain_matter, plain_wound) = bite(&mut a, id);
        let (toxic_matter, toxic_wound) = bite(&mut b, id);
        assert_eq!(plain_matter, toxic_matter);
        assert_eq!(plain_wound, 0);
        assert_eq!(toxic_wound, if segments == 1 { 3 } else { 4 });
        assert_eq!(a.sim.matter(), plain.sim.matter());
        assert_eq!(b.sim.matter(), toxic.sim.matter());
        println!(
            "{segments} repeated region(s) | plain bite {plain_matter}, wound {plain_wound} | toxic bite {toxic_matter}, wound {toxic_wound}"
        );
    }
}

#[derive(Default)]
struct Run {
    eaten: u64,
    toxin_wounds: u64,
    rent: u64,
    born: usize,
    producers: u64,
}

fn run(found: &Founding, ticks: u64) -> Run {
    let mode = Execution::Grouped;
    let mut sim = Session::new(found.generate().unwrap(), mode).unwrap();
    let matter = sim.sim.matter();
    let mut totals = Run::default();
    let mut newborns = BTreeSet::new();
    for tick in 1..=ticks {
        let before = sim.sim.state().population.next_id;
        let result = sim.advance_tick_with_flows().unwrap();
        for f in result.flows {
            if f.by_process(FOOD) && f.from_kind.as_ref().is_some_and(|k| k.lineage == PLANT) {
                totals.eaten += f.amount * f.count;
            }
            if f.by_process("guard:rent-0") && !f.internal() {
                totals.rent += f.amount * f.count;
            }
        }
        totals.toxin_wounds += sim
            .sim
            .harms()
            .iter()
            .filter(|h| h.lineage == PLANT && h.ward == Some(Ward::Toxin))
            .map(|h| u64::from(h.cells))
            .sum::<u64>();
        let state = sim.sim.state();
        for relation in state
            .relations
            .iter()
            .filter(|r| r.kind == "sim:parent" && r.subject >= before)
        {
            let parent = state.population.get(relation.object).unwrap();
            let child = state.population.get(relation.subject).unwrap();
            if child.lineage == PLANT {
                assert_eq!(parent.soma, child.soma);
                let expected = if found.body_maps[PLANT].regions[REGION] == "part:cyanide-leaf" {
                    u32::from(child.soma[1])
                } else {
                    0
                };
                assert_eq!(glands(child), expected);
                newborns.insert(relation.subject);
            }
        }
        assert_eq!(sim.sim.matter(), matter);
        if tick == ticks / 2 {
            let save = serde_json::to_vec(&sim.save()).unwrap();
            let reopened = Session::load_json(&save, mode).unwrap();
            assert_eq!(sim.save(), reopened.save());
            sim = reopened;
        }
    }
    totals.born = newborns.len();
    totals.producers = sim
        .sim
        .state()
        .population
        .groups
        .values()
        .filter(|g| g.entity.alive && g.entity.lineage == PLANT)
        .map(|g| g.count)
        .sum();
    let save = serde_json::to_vec(&sim.save()).unwrap();
    let reopened = Session::load_json(&save, mode).unwrap();
    assert_eq!(sim.save(), reopened.save());
    totals
}

#[test]
fn matched_worlds_advance_reproduce_and_reopen_with_the_body_map() {
    println!(
        "seed,variant,plant matter eaten,toxin wound cells recorded,guard upkeep matter,plant births,living plants at tick 35"
    );
    let (mut plain_births, mut toxic_births) = (0, 0);
    for seed in [9, 17, 31] {
        let authored = Founding { seed, ..founding() };
        let plain = run(&plain(&authored), 35);
        let toxic = run(&authored, 35);
        assert_eq!((plain.rent, plain.toxin_wounds), (0, 0));
        assert!(toxic.eaten > 0 && toxic.rent > 0 && toxic.toxin_wounds > 0);
        plain_births += plain.born;
        toxic_births += toxic.born;
        for (name, r) in [("plain", plain), ("toxic", toxic)] {
            println!(
                "{seed},{name},{},{},{},{},{}",
                r.eaten, r.toxin_wounds, r.rent, r.born, r.producers
            );
        }
    }
    assert!(
        plain_births > 0 && toxic_births > 0,
        "both maps should exercise native inheritance across the matrix"
    );
}

#[test]
fn a_body_map_refuses_missing_bindings_or_a_lineage_it_cannot_found() {
    let mut unmapped = founding();
    let map = unmapped.body_maps.get_mut(PLANT).unwrap();
    map.regions.clear();
    assert!(
        unmapped
            .generate()
            .unwrap_err()
            .contains("no binding for region:defence")
    );

    let mut no_guard = founding();
    no_guard.guard = None;
    assert!(
        no_guard
            .generate()
            .unwrap_err()
            .contains("needs the guard rule")
    );

    let mut missing = founding();
    let map = missing.body_maps.remove(PLANT).unwrap();
    missing.body_maps.insert("lineage:99".into(), map);
    assert!(
        missing
            .generate()
            .unwrap_err()
            .contains("needs a lineage with bodied founders")
    );

    let mut broken = founding();
    let map = broken.body_maps.get_mut(PLANT).unwrap();
    map.grammar.rules.push(Rule::Repeat { tagma: 99, by: 1 });
    assert!(broken.generate().unwrap_err().contains("tagma 99"));
}
