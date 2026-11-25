// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::flows::Holder;
use crate::process::IntakePort;
use crate::schema::Entity;

fn has_tuber(e: &Entity) -> bool {
    e.living()
        .any(|(_, part)| part.port.is_some_and(IntakePort::resprouts))
}

fn tuber_founding() -> Genesis {
    let mut founding: Founding =
        toml::from_str(include_str!("../../../examples/optional-gland.toml")).unwrap();
    let key = "part:resprouting-tuber";
    founding.repertoire.as_mut().unwrap().only = [key.to_owned()].into();
    let map = founding.body_maps.get_mut("lineage:0").unwrap();
    map.regions.insert("region:gland".into(), key.into());
    founding.generate().unwrap()
}

#[test]
fn inherited_storage_refills_grazed_tissue_with_its_own_matter() {
    let g = tuber_founding();
    let template = &g.rules.kinds[&kind_key("part:resprouting-tuber")];
    assert!(template.cells[crate::anatomy::STORE] > 0);
    assert!(template.port.unwrap().resprouts());
    assert!(!template.port.unwrap().is_mouth());
    let first_child = g.population.next_id;
    let mut witnessed = 0;
    let mut inherited = false;
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut run = Session::new(g.clone(), mode).unwrap();
        let mut silent = Session::new(g.clone(), mode).unwrap();
        let matter = run.sim.matter();
        let mut returned = 0;
        for tick in 1..=40 {
            let flows = run.advance_tick_with_flows().unwrap().flows;
            for f in flows
                .iter()
                .filter(|f| f.by_process("ecology:feed-1") && f.internal())
            {
                let (Holder::Part(id, donor), Holder::Part(_, leaf)) = (f.from.0, f.to.0) else {
                    continue;
                };
                assert_ne!(donor, leaf);
                assert_eq!(f.from.1, f.to.1);
                let plant = run.sim.state().population.get(id).unwrap();
                assert!(plant.parts[&donor].port.is_some_and(IntakePort::resprouts));
                assert!(plant.parts[&leaf].cells["function:fix"] > 0);
                assert!(f.amount > 0);
                returned += f.amount * f.count;
            }
            for (&first, group) in &run.sim.state().population.groups {
                let plant = &group.entity;
                if plant.lineage == "lineage:0" && plant.alive {
                    assert_eq!(has_tuber(plant), !plant.absent.contains(&(1, 0)));
                    inherited |= first >= first_child && has_tuber(plant);
                }
            }
            assert_eq!(run.sim.matter(), matter);
            if tick == 20 {
                let bytes = serde_json::to_vec(&run.save()).unwrap();
                run = Session::load_json(&bytes, mode).unwrap();
            }
        }
        assert!(returned > 0);
        silent.advance(40).unwrap();
        assert_eq!(run.sim.state_hash(), silent.sim.state_hash());
        if witnessed != 0 {
            assert_eq!(returned, witnessed);
        }
        witnessed = returned;
    }
    assert!(inherited);
}

/// Make one bite on the ordinary grazer's feeding path. The fixing tissue and
/// storage geometry match in every case. Only the working cells or stock vary.
#[test]
fn one_bite_charges_the_store_and_an_empty_or_inactive_store_cannot_pay() {
    let initial = tuber_founding();
    let plant = *initial
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.lineage == "lineage:0" && has_tuber(&group.entity))
        .unwrap()
        .0;
    let grazer = *initial
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.lineage == "lineage:1")
        .unwrap()
        .0;
    let body = initial.population.get(plant).unwrap();
    let store = body
        .living()
        .find(|(_, p)| p.port.is_some_and(IntakePort::resprouts))
        .unwrap()
        .0;
    let leaf = body
        .living()
        .find(|(_, p)| p.cells.get("function:fix").copied().unwrap_or(0) > 0)
        .unwrap()
        .0;
    let own = crate::anatomy::own(&initial.rules, "lineage:0")
        .tissue
        .unwrap();
    for (stock, functional) in [(1, true), (0, true), (1, false)] {
        let mut genesis = initial.clone();
        let body = genesis
            .population
            .groups
            .get_mut(&plant)
            .unwrap()
            .entity_mut();
        for p in body.parts.values_mut() {
            std::sync::Arc::make_mut(p).matter.clear();
        }
        body.part_mut(&leaf).unwrap().matter.insert(own.clone(), 6);
        if stock > 0 {
            body.part_mut(&store)
                .unwrap()
                .matter
                .insert(own.clone(), stock);
        }
        if !functional {
            let extent = body.extent(store);
            let organ = body.part_mut(&store).unwrap();
            organ.cells.remove(crate::anatomy::STORE);
            crate::mosaic::sync(organ, extent);
        }
        let mut run = Session::new(genesis, Execution::Individuals).unwrap();
        let total = run.sim.matter();
        let mut saw_frond = false;
        for _ in 0..6 {
            let old = run.sim.state().population.get(plant).unwrap();
            let held = |e: &Entity, part| e.parts[&part].matter.get(&own).copied().unwrap_or(0);
            let (before_leaf, before_store) = (held(old, leaf), held(old, store));
            let flows = run
                .command_with_flows(Command::Act {
                    actor: grazer,
                    target: Some(plant),
                    process: "ecology:feed-1".into(),
                    cause: None,
                })
                .unwrap()
                .flows;
            let grazed: u64 = flows
                .iter()
                .filter(|f| !f.internal() && f.from == (Holder::Part(plant, leaf), own.clone()))
                .map(|f| f.amount)
                .sum();
            if grazed == 0 {
                continue;
            }
            let replaced: u64 = flows
                .iter()
                .filter(|f| {
                    f.from.0 == Holder::Part(plant, store) && f.to.0 == Holder::Part(plant, leaf)
                })
                .map(|f| f.amount)
                .sum();
            let body = run.sim.state().population.get(plant).unwrap();
            assert_eq!(
                replaced,
                if functional {
                    before_store.min(grazed)
                } else {
                    0
                }
            );
            assert_eq!(held(body, leaf), before_leaf - grazed + replaced);
            assert_eq!(held(body, store), before_store - replaced);
            assert_eq!(run.sim.matter(), total);
            saw_frond = true;
            break;
        }
        assert!(saw_frond, "the grazer must actually bite fixing tissue");
    }
}
