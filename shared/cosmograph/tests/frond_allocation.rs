// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Execution, Founding, Session, anatomy, development, rules::Rule, trophic};

const SOURCE: &str = include_str!("../examples/frond-allocation.toml");
const PLANT: &str = "lineage:0";

#[test]
fn partition_admits_only_attached_equal_budget_forms() {
    let f: Founding = toml::from_str(SOURCE).unwrap();
    let g = f.generate().unwrap();
    let d = g.lineages[PLANT].development.as_ref().unwrap();
    let mut soma = development::lineage_soma(&g.rules, d, 11);
    for n in [1, 3] {
        soma.segments[1] = n;
        let body = development::develop(&g.rules, d, &soma).unwrap();
        let leaves: Vec<_> = body.doc.living().filter(|p| p.situs.unwrap()[0] == 1).collect();
        assert_eq!(leaves.len(), usize::from(n));
        assert_eq!(leaves.iter().map(|p| anatomy::voxels(p.half_extent)).sum::<u128>(), 405);
        for p in &leaves {
            assert_eq!(p.attachment.unwrap().parent, body.doc.root);
            assert_eq!(body.parts[&p.id].cells["function:fix"], 9 / u32::from(n));
        }
        for pair in leaves.windows(2) {
            let left = pair[0]; let right = pair[1];
            assert!(left.attachment.unwrap().offset[2] + left.half_extent[2] < right.attachment.unwrap().offset[2] - right.half_extent[2]);
        }
    }
    soma.segments[1] = 2;
    assert!(development::develop(&g.rules, d, &soma).is_err());
    let mut bad = f.clone();
    let grammar = &mut bad.body_maps.get_mut(PLANT).unwrap().grammar;
    let Rule::Partition(plan) = &mut grammar.rules[0] else {panic!()};
    plan.piece[0] += 1;
    assert!(bad.generate().unwrap_err().contains("conserve organ volume"));
    let mut bad = f;
    bad.body_maps.get_mut(PLANT).unwrap().grammar.seed[1].segment = "kind:block".into();
    assert!(bad.generate().unwrap_err().contains("trait cells"));
}

#[test]
fn authored_partition_survives_founding_and_births_with_inherited_variation() {
    let mut founding: Founding = toml::from_str(SOURCE).unwrap();
    founding.inherited_variation = 100;
    let mut session = Session::new(founding.generate().unwrap(), Execution::Grouped).unwrap();
    let development = session.sim.state().lineages[PLANT].development.as_ref().unwrap();
    assert_eq!(development.grammar.as_ref().unwrap().partition(1).unwrap().pieces, 3);
    session.advance(35).unwrap();
    let state = session.sim.state();
    let mut forms = std::collections::BTreeSet::new();
    for relation in state.relations.iter().filter(|r| r.kind == "sim:parent") {
        let child = state.population.get(relation.subject).unwrap();
        if child.lineage == PLANT {
            let parent = state.population.get(relation.object).unwrap();
            assert_eq!(parent.soma[1], child.soma[1]);
            forms.insert(child.soma[1]);
        }
    }
    assert_eq!(forms.into_iter().collect::<Vec<_>>(), [1, 3]);
}

#[test]
fn exposure_scales_with_funded_tissue_on_each_plate() {
    let g = toml::from_str::<Founding>(SOURCE).unwrap().generate().unwrap();
    let mut plant = g.population.groups.values().find(|g| g.entity.lineage == PLANT && g.entity.soma[1] == 3).unwrap().entity.as_ref().clone();
    let leaves = trophic::canopy_parts(plant.body.as_ref().unwrap());
    for (i, id) in leaves.iter().enumerate() {
        plant.part_mut(id).unwrap().matter.insert("matter:0-0".into(), [0,2,17][i]);
    }
    assert_eq!(trophic::fixing_tissue(&plant, "matter:0-0"), (9,5));
    for k in 0..=9 {
        assert_eq!((97..106).filter(|turn| trophic::fixes_this_turn(k,9,*turn)).count(), k as usize);
    }
}

#[test]
fn exclosure_preserves_founders_and_both_forms_inherit_through_reopen() {
    let f: Founding = toml::from_str(SOURCE).unwrap();
    let stocked = f.generate().unwrap();
    let mut off = f;
    off.unstocked_lineages.insert(1);
    let empty = off.generate().unwrap();
    assert_eq!(stocked.sites, empty.sites);
    assert_eq!(stocked.population.next_id, empty.population.next_id);
    for (id, group) in &stocked.population.groups {
        if group.entity.lineage == "lineage:1" { assert!(empty.population.get(*id).is_none()); }
        else {
            assert_eq!(Some(group.entity.as_ref()), empty.population.get(*id));
            if group.entity.lineage == PLANT {
                assert_eq!(trophic::fixing_tissue(&group.entity, "matter:0-0"), (9, 9));
            }
        }
    }
    for g in [stocked, empty] {
        let mut hashes = vec![];
        for mode in [Execution::Grouped, Execution::Individuals] {
            let mut session = Session::new(g.clone(), mode).unwrap();
            session.advance(11).unwrap();
            let mut reopened = Session::load_json(&serde_json::to_vec(&session.save()).unwrap(), mode).unwrap();
            session.advance(24).unwrap(); reopened.advance(24).unwrap();
            assert_eq!(session.sim.state_hash(), reopened.sim.state_hash());
            hashes.push(session.sim.state_hash());
            let s = session.sim.state();
            let mut births = [0; 2];
            for r in s.relations.iter().filter(|r| r.kind == "sim:parent") {
                let p = s.population.get(r.object).unwrap();
                let child = s.population.get(r.subject).unwrap();
                if child.lineage == PLANT {
                    assert_eq!(p.soma, child.soma);
                    births[usize::from(child.soma[1] == 3)] += 1;
                }
            }
            assert!(births.iter().all(|n| *n > 0));
        }
        assert_eq!(hashes[0], hashes[1]);
    }
}
