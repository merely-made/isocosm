// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Founding, FoundingForm, KingdomDraw};
use crate::{Execution, Session, repertoire::Repertoire};
use std::collections::{BTreeMap, BTreeSet};

fn founding(seed: u64, draw: KingdomDraw) -> Founding {
    let grid = crate::map::Grid {
        shape: "shape:sphere".into(),
        width: 6,
        height: 4,
        side: 256,
        elevation: [4, 26],
        relief: [1, 8],
        sea_per_mille: 300,
    };
    Founding {
        seed,
        sites: grid.width * grid.height,
        population: 120,
        cohort_size: 2,
        ecology: true,
        ecology_rates: crate::ecology::Rates::A1,
        web: crate::ecology::Web::Diets,
        map: Some(crate::map::Layout::Grid(grid)),
        bodies: Some(Default::default()),
        inherit: true,
        repertoire: Some(Repertoire {
            founders: true,
            kingdom_draw: Some(draw),
            ..Default::default()
        }),
        guard: Some(Default::default()),
        categories: Some(Default::default()),
        nutrients: Some(Default::default()),
        mutualism: Some(Default::default()),
        detritus: Some(Default::default()),
        ..Default::default()
    }
}

#[test]
fn kingdom_draw_varies_the_subset_and_richness_and_accepts_exact_counts() {
    let d = KingdomDraw::default();
    let rolls: BTreeSet<_> = (0..32).map(|seed| d.draw(seed).unwrap()).collect();
    let subsets: BTreeSet<_> = rolls
        .iter()
        .map(|r| r.iter().map(|(k, _)| k).collect::<Vec<_>>())
        .collect();
    assert!(subsets.len() > 2);
    assert!(
        rolls
            .iter()
            .any(|r| r.windows(2).any(|pair| pair[0].1 != pair[1].1))
    );
    for r in rolls {
        assert_eq!(
            r.len(),
            r.iter().map(|(k, _)| k).collect::<BTreeSet<_>>().len()
        );
    }
    let fixed = KingdomDraw {
        kingdoms: [5, 5],
        per_kingdom: [20, 20],
        ..d
    };
    assert_eq!(
        fixed.draw(7).unwrap(),
        FoundingForm::ALL.map(|k| (k, 20)).to_vec()
    );
}

#[test]
fn kingdom_draw_founders_use_their_native_scale_and_reopen() {
    let f = founding(
        8307,
        KingdomDraw {
            kingdoms: [5, 5],
            per_kingdom: [20, 20],
            ..Default::default()
        },
    );
    let g = f.generate().unwrap();
    let mut counts = BTreeMap::new();
    for (k, l) in &g.lineages {
        if !k.starts_with("lineage:") {
            continue;
        }
        assert!(l.parent.is_none());
        *counts.entry(l.kingdom.clone()).or_insert(0usize) += 1;
        if l.kingdom == FoundingForm::Microbe.key() {
            assert!(
                g.spreads
                    .identities
                    .contains(&crate::spread::Identity::Surface(k.clone()))
            );
            assert!(
                g.sites
                    .values()
                    .any(|s| s.accounts.get(k).copied().unwrap_or(0) > 0)
            );
        } else if l.kingdom == FoundingForm::WorldClass.key() {
            let marker = k.replace("lineage:", "habitat:body-");
            assert!(g.sites.values().any(|s| s.conditions.contains_key(&marker)));
        } else {
            assert!(l.development.as_ref().unwrap().grammar.is_some());
            assert!(
                g.population
                    .groups
                    .values()
                    .any(|group| group.entity.lineage == *k && group.entity.body.is_some())
            );
        }
    }
    for k in FoundingForm::ALL {
        assert_eq!(counts[k.key()], 20);
    }
    let mut session = Session::new(g, Execution::Grouped).unwrap();
    let initial = session.sim.matter();
    session.advance(2).unwrap();
    assert_eq!(session.sim.matter(), initial);
    let loaded = Session::load_json(
        &serde_json::to_vec(&session.save()).unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
}

#[test]
fn kingdom_draw_world_class_changes_the_generated_geography_at_its_footprint() {
    for seed in [2, 6, 31] {
        let mut f = founding(
            seed,
            KingdomDraw {
                pool: BTreeSet::from([FoundingForm::WorldClass.key().to_owned()]),
                kingdoms: [1, 1],
                per_kingdom: [1, 1],
            },
        );
        let recipe = super::WorldBody {
            reach: 1,
            elevation: 6,
            relief: 2,
            water: -4,
        };
        f.repertoire
            .as_mut()
            .unwrap()
            .world_bodies
            .insert("0".into(), recipe);
        let g = f.generate().unwrap();
        let base = f.map.as_ref().unwrap().lay(seed, f.sites).unwrap();
        let mut affected = 0;
        for (id, s) in &g.sites {
            let before = &base.sites[id].conditions;
            let share = s.conditions.get("habitat:body-0").copied().unwrap_or(0);
            let actual = ["terrain:elevation", "terrain:relief", "terrain:water"]
                .map(|k| s.conditions[k] - before[k]);
            assert_eq!(actual, recipe.contribution(share));
            if share > 0 {
                affected += 1;
            }
        }
        assert!(affected > 1 && affected < g.sites.len());
        assert!(
            g.population
                .groups
                .values()
                .all(|grp| grp.entity.lineage == "world:ground")
        );
    }
}

#[test]
fn generative_founders_grow_different_architectures_with_permitted_organs() {
    use crate::rules::{Rule, unfold};

    let distribution = KingdomDraw {
        pool: [
            FoundingForm::Plant,
            FoundingForm::Animal,
            FoundingForm::Fungus,
        ]
        .map(|form| form.key().to_owned())
        .into(),
        kingdoms: [3, 3],
        per_kingdom: [6, 6],
    };
    let mut found = founding(8307, distribution);
    let original = found.catalogue_body_maps().unwrap();
    assert!(original.values().all(|m| {
        m.grammar
            .rules
            .iter()
            .all(|r| matches!(r, Rule::Repeat { .. }))
    }));
    found.repertoire.as_mut().unwrap().generative_ecology = true;
    let catalogue = crate::catalog::catalog();
    let mut used_foreign = false;
    for cross in [false, true] {
        found.repertoire.as_mut().unwrap().across_kingdoms = cross;
        let maps = found.catalogue_body_maps().unwrap();
        let world = found.generate().unwrap();
        let mut branching = false;
        let mut radial = false;
        for (key, map) in &maps {
            let development = world.lineages[key].development.as_ref().unwrap();
            assert_eq!(
                unfold(development.grammar.as_ref().unwrap()).unwrap(),
                development.recipe.tagmata
            );
            let grown = &development.recipe.tagmata;
            assert!(grown.iter().all(|r| r.segments > 0));
            assert!(grown.iter().map(|r| u32::from(r.segments)).sum::<u32>() <= 25);
            branching |= map
                .grammar
                .rules
                .iter()
                .any(|r| matches!(r, Rule::Branch { .. }))
                && grown.len() > map.grammar.seed.len();
            radial |= map
                .grammar
                .rules
                .iter()
                .any(|r| matches!(r, Rule::Radiate { .. }))
                && grown.len() > map.grammar.seed.len();
            for part in map.regions.values().filter(|key| key.starts_with("part:")) {
                if catalogue.part_by_key(part).unwrap().kingdom != world.lineages[key].kingdom {
                    assert!(cross);
                    used_foreign = true;
                }
            }
        }
        assert!(branching && radial);
    }
    assert!(used_foreign);
}
