// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use cosmograph::flows::MadeBy;
use std::collections::BTreeSet;

#[test]
fn generated_founders_keep_authored_overrides_and_reroll_unedited_lines() {
    let mut panel = Panel::new(Panel::starter(40)).unwrap();
    assert!(panel.draft.body_maps.is_empty());
    let old = panel.generated_maps.clone();
    let line = panel.lineage.clone();
    panel.map_mut().variance = 2;
    let edited = panel.map().unwrap().clone();
    panel.seed = field(41);
    panel.project().unwrap();
    assert_eq!(panel.map(), Some(&edited));
    assert_eq!(panel.draft.body_maps.len(), 1);
    assert!(
        old.iter()
            .any(|(k, m)| k != &line && panel.generated_maps.get(k) != Some(m))
    );
    let g = panel.session.sim.genesis();
    for (key, map) in &panel.generated_maps {
        let lineage = &g.lineages[key];
        assert!(lineage.development.as_ref().unwrap().grammar.is_some());
        for part in map.regions.values().filter(|p| p.starts_with("part:")) {
            let kind = cosmograph::repertoire::kind_key(part);
            assert!(g.rules.repertoire.as_ref().unwrap().reaches(&lineage.kingdom, &kind));
        }
    }
}

#[test]
fn generated_populations_use_their_organs_and_pass_their_forms_to_offspring() {
    let mut births = 0;
    let mut observed = BTreeSet::new();
    for seed in 40..43 {
        let draft = Panel::starter(seed);
        let maps = draft.catalogue_body_maps().unwrap();
        let mut s = Session::new(draft.generate().unwrap(), Execution::Grouped).unwrap();
        let initial = s.sim.matter();
        let mut actions: BTreeMap<String, u128> = BTreeMap::new();
        let mut born = 0;
        let mut forms = BTreeSet::new();
        for group in s.sim.state().population.groups.values() {
            let e = &group.entity;
            if !maps.contains_key(&e.lineage) {
                continue;
            }
            forms.insert((e.lineage.clone(), e.soma.clone(), e.absent.clone()));
            for (_, part) in e.living() {
                if let Some(port) = cosmograph::mosaic::ports::active(part) {
                    observed.insert(port);
                }
            }
        }
        for _ in 0..36 {
            let first = s.sim.state().population.next_id;
            for flow in s.advance_tick_with_flows().unwrap().flows {
                if let MadeBy::Process(key) = flow.made_by
                    && flow.from.0 != flow.to.0
                {
                    *actions.entry(key).or_default() +=
                        u128::from(flow.count) * u128::from(flow.amount);
                }
            }
            let state = s.sim.state();
            for r in state
                .relations
                .iter()
                .filter(|r| r.kind == "sim:parent" && r.subject >= first)
            {
                let (Some(child), Some(parent)) = (
                    state.population.get(r.subject),
                    state.population.get(r.object),
                ) else {
                    continue;
                };
                if child.lineage == parent.lineage && maps.contains_key(&child.lineage) {
                    assert_eq!(child.soma, parent.soma);
                    assert_eq!(child.absent, parent.absent);
                    born += 1;
                }
            }
            assert_eq!(s.sim.matter(), initial);
        }
        let reopened =
            Session::load_json(&serde_json::to_vec(&s.save()).unwrap(), Execution::Grouped)
                .unwrap();
        assert_eq!(s.sim.state_hash(), reopened.sim.state_hash());
        let organs: BTreeMap<_, _> = maps.iter().map(|(k, m)| (k, &m.regions)).collect();
        let relevant: Vec<_> = actions
            .iter()
            .filter(|(k, _)| {
                k.starts_with("nutrients:tuned")
                    || k.starts_with("guard:rent")
                    || k.starts_with("mutualism:")
                    || k.starts_with("ecology:feed")
            })
            .collect();
        println!(
            "world {seed}, {} founder forms, {born} witnessed births, genes {organs:?}, transfers {relevant:?}",
            forms.len()
        );
        assert!(!relevant.is_empty());
        births += born;
    }
    assert!(observed.len() > 2);
    assert!(births > 0);
}
